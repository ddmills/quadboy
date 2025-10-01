use bevy_ecs::prelude::*;
use macroquad::input::KeyCode;
use quadboy_macros::profiled_system;

use crate::{
    common::Palette,
    domain::systems::game_log_system::GameLog,
    engine::{KeyInput, Mouse},
    rendering::{Layer, Position, ScreenSize, Text, Visibility},
    states::CleanupStateExplore,
    ui::UiLayout,
};

#[derive(Component)]
pub struct EventLogUi {
    pub visible_lines: usize,
    pub max_width: f32,
    pub scroll_offset: usize,
    pub auto_scroll: bool,
    pub show_timestamps: bool,
    pub fade_old_messages: bool,
    pub last_message_count: usize,
}

impl Default for EventLogUi {
    fn default() -> Self {
        Self {
            visible_lines: 6,
            max_width: 40.0,
            scroll_offset: 0,
            auto_scroll: true,
            show_timestamps: false,
            fade_old_messages: true,
            last_message_count: 0,
        }
    }
}

impl EventLogUi {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_lines(mut self, lines: usize) -> Self {
        self.visible_lines = lines;
        self
    }

    pub fn with_width(mut self, width: f32) -> Self {
        self.max_width = width;
        self
    }

    pub fn with_timestamps(mut self, show: bool) -> Self {
        self.show_timestamps = show;
        self
    }
}

#[derive(Component)]
pub struct EventLogLine {
    pub line_index: usize,
    pub parent_log: Entity,
}

#[derive(Component)]
pub struct EventLogScrollUpIndicator {
    pub parent_log: Entity,
}

#[derive(Component)]
pub struct EventLogScrollDownIndicator {
    pub parent_log: Entity,
}

#[derive(Component)]
pub struct EventLogBackground {
    pub parent_log: Entity,
}

/// Spawn the event log UI when entering the explore state
pub fn spawn_event_log_ui(cmds: &mut Commands) {
    let log_ui = cmds
        .spawn((
            EventLogUi::new(),
            Position::new_f32(1.0, 0.0, 0.0), // Will be updated to bottom of screen
            CleanupStateExplore,
        ))
        .id();

    // Spawn background panel
    cmds.spawn((
        Text::new("").bg(Palette::Black).layer(Layer::Ui),
        Position::new_f32(0.0, 0.0, 0.0),
        EventLogBackground { parent_log: log_ui },
        Visibility::Hidden, // Start hidden, show when messages exist
        CleanupStateExplore,
    ));

    // Spawn scroll indicators
    cmds.spawn((
        Text::new("▲").fg1(Palette::Yellow).layer(Layer::Ui),
        Position::new_f32(0.0, 0.0, 0.0),
        EventLogScrollUpIndicator { parent_log: log_ui },
        Visibility::Hidden,
        CleanupStateExplore,
    ));

    cmds.spawn((
        Text::new("▼").fg1(Palette::Yellow).layer(Layer::Ui),
        Position::new_f32(0.0, 0.0, 0.0),
        EventLogScrollDownIndicator { parent_log: log_ui },
        Visibility::Hidden,
        CleanupStateExplore,
    ));

    // Spawn text lines for the log
    for i in 0..6 {
        cmds.spawn((
            Text::new("").layer(Layer::Ui),
            Position::new_f32(0.0, 0.0, 0.0),
            EventLogLine {
                line_index: i,
                parent_log: log_ui,
            },
            Visibility::Hidden,
            CleanupStateExplore,
        ));
    }
}

/// Update the event log UI positioning based on screen size
#[profiled_system]
pub fn update_event_log_positioning(
    screen: Res<ScreenSize>,
    ui: Res<UiLayout>,
    mut q_log_ui: Query<&mut Position, With<EventLogUi>>,
    mut q_background: Query<(&EventLogBackground, &mut Position), Without<EventLogUi>>,
    mut q_lines: Query<
        (&EventLogLine, &mut Position),
        (Without<EventLogUi>, Without<EventLogBackground>),
    >,
    mut q_scroll_up: Query<
        (&EventLogScrollUpIndicator, &mut Position),
        (
            Without<EventLogUi>,
            Without<EventLogBackground>,
            Without<EventLogLine>,
        ),
    >,
    mut q_scroll_down: Query<
        (&EventLogScrollDownIndicator, &mut Position),
        (
            Without<EventLogUi>,
            Without<EventLogBackground>,
            Without<EventLogLine>,
            Without<EventLogScrollUpIndicator>,
        ),
    >,
) {
    // Position log at bottom-left of screen, aligned with bottom panel
    // Account for text height (0.5) and 6 visible lines = 3.0 total height
    let log_y = screen.tile_h as f32 - 3.5;
    let left_panel_offset = ui.left_panel.width as f32;

    for mut log_pos in q_log_ui.iter_mut() {
        log_pos.y = log_y;
    }

    // Update background position
    for (_bg, mut bg_pos) in q_background.iter_mut() {
        bg_pos.x = left_panel_offset + 0.5;
        bg_pos.y = log_y - 0.25;
    }

    // Update line positions
    for (line, mut line_pos) in q_lines.iter_mut() {
        line_pos.x = left_panel_offset + 1.5;
        line_pos.y = log_y + (line.line_index as f32 * 0.5);
    }

    // Update scroll indicator positions
    for (_indicator, mut pos) in q_scroll_up.iter_mut() {
        pos.x = left_panel_offset + 0.5;
        pos.y = log_y;
    }

    for (_indicator, mut pos) in q_scroll_down.iter_mut() {
        pos.x = left_panel_offset + 0.5;
        pos.y = log_y + 2.5;
    }
}

/// Update the event log display with messages from GameLog
#[profiled_system]
pub fn update_event_log_display(
    game_log: Res<GameLog>,
    mut q_log_ui: Query<&mut EventLogUi>,
    mut q_lines: Query<(&EventLogLine, &mut Text, &mut Visibility)>,
    mut q_background: Query<&mut Visibility, (With<EventLogBackground>, Without<EventLogLine>)>,
    mut q_scroll_up: Query<
        &mut Visibility,
        (
            With<EventLogScrollUpIndicator>,
            Without<EventLogLine>,
            Without<EventLogBackground>,
        ),
    >,
    mut q_scroll_down: Query<
        &mut Visibility,
        (
            With<EventLogScrollDownIndicator>,
            Without<EventLogLine>,
            Without<EventLogBackground>,
            Without<EventLogScrollUpIndicator>,
        ),
    >,
) {
    let messages = game_log.get_messages();
    let message_count = messages.len();

    if message_count == 0 {
        // Hide everything if no messages
        for (_, mut text, mut vis) in q_lines.iter_mut() {
            text.value = String::new();
            *vis = Visibility::Hidden;
        }

        for mut vis in q_background.iter_mut() {
            *vis = Visibility::Hidden;
        }

        for mut vis in q_scroll_up.iter_mut() {
            *vis = Visibility::Hidden;
        }

        for mut vis in q_scroll_down.iter_mut() {
            *vis = Visibility::Hidden;
        }

        return;
    }

    let Ok(mut log_ui) = q_log_ui.single_mut() else {
        return;
    };

    // Check if new messages arrived and auto-scroll if enabled
    if log_ui.auto_scroll && message_count > log_ui.last_message_count {
        log_ui.scroll_offset = message_count.saturating_sub(log_ui.visible_lines);
    }
    log_ui.last_message_count = message_count;

    // Show background
    for mut vis in q_background.iter_mut() {
        *vis = Visibility::Visible;
    }

    // Update scroll indicators
    let can_scroll_up = log_ui.scroll_offset > 0;
    let can_scroll_down = log_ui.scroll_offset + log_ui.visible_lines < message_count;

    for mut vis in q_scroll_up.iter_mut() {
        *vis = if can_scroll_up {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    for mut vis in q_scroll_down.iter_mut() {
        *vis = if can_scroll_down {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    // Update line contents
    let messages_vec: Vec<_> = messages.iter().collect();

    for (line, mut text, mut vis) in q_lines.iter_mut() {
        if line.line_index < log_ui.visible_lines {
            let message_index = log_ui.scroll_offset + line.line_index;

            if message_index < message_count {
                let message = &messages_vec[message_index];

                // Format the message with optional timestamp
                text.value = if log_ui.show_timestamps {
                    format!("[{}] {}", message.tick, message.text)
                } else {
                    message.text.clone()
                };

                // Apply fade effect for older messages if enabled
                if log_ui.fade_old_messages {
                    let age = message_count - message_index - 1;
                    if age > 2 {
                        text.fg1 = Some(Palette::DarkGray.into());
                    } else {
                        text.fg1 = Some(Palette::White.into());
                    }
                }

                *vis = Visibility::Visible;
            } else {
                text.value = String::new();
                *vis = Visibility::Hidden;
            }
        } else {
            text.value = String::new();
            *vis = Visibility::Hidden;
        }
    }
}

/// Handle scrolling of the event log
#[profiled_system]
pub fn event_log_scroll_system(
    mut q_log_ui: Query<&mut EventLogUi>,
    game_log: Res<GameLog>,
    keys: Res<KeyInput>,
    mouse: Res<Mouse>,
    screen: Res<ScreenSize>,
    ui: Res<UiLayout>,
) {
    let Ok(mut log_ui) = q_log_ui.single_mut() else {
        return;
    };

    let message_count = game_log.get_messages().len();
    if message_count == 0 {
        return;
    }

    let max_scroll = message_count.saturating_sub(log_ui.visible_lines);

    // Handle keyboard controls
    if keys.is_pressed(KeyCode::PageUp) {
        log_ui.scroll_offset = log_ui.scroll_offset.saturating_sub(3);
        log_ui.auto_scroll = false;
    }

    if keys.is_pressed(KeyCode::PageDown) {
        log_ui.scroll_offset = (log_ui.scroll_offset + 3).min(max_scroll);
        log_ui.auto_scroll = false;
    }

    if keys.is_pressed(KeyCode::Home) {
        log_ui.auto_scroll = !log_ui.auto_scroll;
        if log_ui.auto_scroll {
            log_ui.scroll_offset = max_scroll;
        }
    }

    if keys.is_pressed(KeyCode::End) {
        log_ui.scroll_offset = max_scroll;
        log_ui.auto_scroll = true;
    }

    // Handle mouse wheel scrolling
    if mouse.wheel_delta.1.abs() > 0.01 {
        let log_y = screen.tile_h as f32 - 3.0;
        let log_height = log_ui.visible_lines as f32 * 0.5;
        let left_panel_offset = ui.left_panel.width as f32;

        // Check if mouse is over the log area
        let mouse_over_log = mouse.ui.0 >= left_panel_offset + 1.5
            && mouse.ui.0 <= 41.0
            && mouse.ui.1 >= log_y
            && mouse.ui.1 <= log_y + log_height;

        if mouse_over_log {
            let scroll_speed = 3;

            if mouse.wheel_delta.1 > 0.0 {
                // Scroll up (show older messages)
                log_ui.scroll_offset = log_ui.scroll_offset.saturating_sub(scroll_speed);
                log_ui.auto_scroll = false;
            } else {
                // Scroll down (show newer messages)
                log_ui.scroll_offset = (log_ui.scroll_offset + scroll_speed).min(max_scroll);

                // Re-enable auto-scroll if we've scrolled to the bottom
                if log_ui.scroll_offset == max_scroll {
                    log_ui.auto_scroll = true;
                }
            }
        }
    }
}

/// Handle visibility and new message notifications
#[profiled_system]
pub fn event_log_visibility_system(game_log: Res<GameLog>, mut q_log_ui: Query<&mut EventLogUi>) {
    let Ok(mut log_ui) = q_log_ui.single_mut() else {
        return;
    };

    // Mark messages as read after they've been displayed
    if game_log.has_new_messages() {
        // We could add a brief highlight or flash effect here
        // For now, we'll just ensure auto-scroll works correctly
    }
}
