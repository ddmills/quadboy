use bevy_ecs::{
    component::Component,
    event::EventReader,
    prelude::*,
    query::With,
    schedule::IntoScheduleConfigs,
    system::{Commands, Local, Query, Res, ResMut, SystemId},
};
use macroquad::{input::KeyCode, prelude::trace};

use crate::{
    common::Palette,
    domain::{GameSettings, SaveGameCommand, SaveGameResult},
    engine::{Plugin, Time},
    rendering::{Glyph, Layer, Position, ScreenSize, Text},
    states::{cleanup_system, AppState, CurrentAppState, CurrentGameState, GameStatePlugin},
    ui::{setup_fullscreen_backgrounds, FullScreenBackground, List, ListItemData},
};

use super::GameState;

#[derive(Resource)]
struct PauseCallbacks {
    continue_game: SystemId,
    save_game: SystemId,
    quit_to_menu: SystemId,
}

pub struct PauseStatePlugin;

impl Plugin for PauseStatePlugin {
    fn build(&self, app: &mut crate::engine::App) {
        GameStatePlugin::new(GameState::Pause)
            .on_enter(app, (setup_callbacks, on_enter_pause, setup_pause_background, setup_fullscreen_backgrounds).chain())
            .on_update(app, (update_save_status_display, handle_save_result))
            .on_update(
                app,
                setup_fullscreen_backgrounds.run_if(resource_changed::<ScreenSize>),
            )
            .on_leave(
                app,
                (
                    on_leave_pause,
                    cleanup_system::<CleanupStatePause>,
                    remove_pause_callbacks,
                )
                    .chain(),
            );
    }
}

fn on_leave_pause() {
    trace!("LeaveGameState::<Pause>");
}

#[derive(Component)]
pub struct CleanupStatePause;

#[derive(Component)]
pub struct SaveStatusDisplay;

fn setup_callbacks(world: &mut World) {
    let callbacks = PauseCallbacks {
        continue_game: world.register_system(continue_game),
        save_game: world.register_system(save_game_callback),
        quit_to_menu: world.register_system(quit_to_menu),
    };

    world.insert_resource(callbacks);
}

fn continue_game(mut game_state: ResMut<CurrentGameState>) {
    game_state.next = GameState::Explore;
}

fn save_game_callback(
    mut cmds: Commands,
    settings: Res<GameSettings>,
    mut q_save_status: Query<&mut Text, With<SaveStatusDisplay>>,
) {
    if !settings.enable_saves {
        if let Ok(mut text) = q_save_status.single_mut() {
            text.value = "Save disabled in settings".to_string();
        }
        return;
    }

    cmds.queue(SaveGameCommand);

    if let Ok(mut text) = q_save_status.single_mut() {
        text.value = format!("Saving game to '{}'...", settings.save_name);
    }
}

fn quit_to_menu(mut app_state: ResMut<CurrentAppState>, mut game_state: ResMut<CurrentGameState>) {
    game_state.next = GameState::None;
    app_state.next = AppState::MainMenu;
}

fn remove_pause_callbacks(mut cmds: Commands) {
    cmds.remove_resource::<PauseCallbacks>();
}

fn on_enter_pause(mut cmds: Commands, callbacks: Res<PauseCallbacks>) {
    cmds.spawn((
        Text::new("PAUSED").bg(Palette::Black),
        Position::new_f32(4., 2., 0.),
        CleanupStatePause,
    ));

    cmds.spawn((
        List::new(vec![
            ListItemData::new("({Y|ESC}) CONTINUE", callbacks.continue_game)
                .with_hotkey(KeyCode::Escape),
            ListItemData::new("({Y|S}) SAVE GAME", callbacks.save_game).with_hotkey(KeyCode::S),
            ListItemData::new("({R|Q}) QUIT TO MAIN MENU", callbacks.quit_to_menu)
                .with_hotkey(KeyCode::Q),
        ])
        .with_focus_order(1000),
        Position::new_f32(4., 3.5, 0.),
        CleanupStatePause,
    ));

    cmds.spawn((
        Text::new("").bg(Palette::Black),
        Position::new_f32(4., 5., 0.),
        CleanupStatePause,
        SaveStatusDisplay,
    ));
}

fn update_save_status_display(
    mut q_save_status: Query<&mut Text, With<SaveStatusDisplay>>,
    mut save_message_timer: Local<Option<f64>>,
    time: Res<Time>,
) {
    if let Ok(text) = q_save_status.single()
        && !text.value.is_empty()
        && save_message_timer.is_none()
    {
        *save_message_timer = Some(time.elapsed + 3.0); // Clear after 3 seconds
    }

    if let Some(clear_time) = *save_message_timer
        && time.elapsed >= clear_time
    {
        if let Ok(mut text) = q_save_status.single_mut() {
            text.value.clear();
        }
        *save_message_timer = None;
    }
}

fn handle_save_result(
    mut e_save_result: EventReader<SaveGameResult>,
    mut q_save_status: Query<&mut Text, With<SaveStatusDisplay>>,
) {
    for result in e_save_result.read() {
        if let Ok(mut text) = q_save_status.single_mut() {
            if result.success {
                text.value = "Game saved.".to_owned();
            } else {
                text.value = "Error saving game.".to_owned();
            }
        }
    }
}

fn setup_pause_background(mut cmds: Commands, screen: Res<ScreenSize>) {
    let color = Palette::Clear;
    cmds.spawn((
        FullScreenBackground,
        CleanupStatePause,
        Position::new(0, 0, 0),
        Glyph::new(6, color, color)
            .bg(color)
            .scale((screen.tile_w as f32, screen.tile_h as f32))
            .layer(Layer::UiPanels),
    ));
}
