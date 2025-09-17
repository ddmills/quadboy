use bevy_ecs::{prelude::*, system::SystemId};
use macroquad::input::KeyCode;

use crate::{
    domain::{
        AiController, Energy, Label, PursuingPlayer, Stats, systems::ai_utils::distance_from_home,
    },
    engine::{AudioKey, Clock},
    rendering::{Glyph, Layer, Position, ScreenSize},
    ui::{ActivatableBuilder, Dialog, DialogContent, DialogText, DialogTextStyle},
};

pub struct AiDebugDialogBuilder {
    entity: Entity,
    width: f32,
    close_callback: SystemId,
}

impl AiDebugDialogBuilder {
    pub fn new(entity: Entity, close_callback: SystemId) -> Self {
        Self {
            entity,
            width: 32.0,
            close_callback,
        }
    }

    pub fn spawn(
        self,
        cmds: &mut Commands,
        q_labels: &Query<&Label>,
        q_ai_controllers: &Query<&AiController>,
        q_glyphs: &Query<&Glyph>,
        q_pursuing: &Query<&PursuingPlayer>,
        q_energy: &Query<&Energy>,
        q_stats: &Query<&Stats>,
        q_positions: &Query<&Position>,
        clock: &Clock,
        cleanup_component: impl Bundle + Clone,
        screen: &ScreenSize,
    ) -> Entity {
        let entity_name = if let Ok(label) = q_labels.get(self.entity) {
            label.get().to_string()
        } else {
            "Unknown AI".to_string()
        };

        let ai_info = if let Ok(ai) = q_ai_controllers.get(self.entity) {
            let mut info_lines = vec![
                format!("State: {:?}", ai.state),
                format!("Template: {:?}", ai.template),
                format!(
                    "Home: ({}, {}, {})",
                    ai.home_position.x, ai.home_position.y, ai.home_position.z
                ),
                format!("Leash Range: {:.1}", ai.leash_range),
                format!("Wander Range: {:.1}", ai.wander_range),
                format!("Detection Range: {:.1}", ai.detection_range),
            ];

            // Add current position and distance from home
            if let Ok(pos) = q_positions.get(self.entity) {
                let (x, y, z) = pos.world();
                info_lines.push(format!("Current: ({}, {}, {})", x, y, z));
                let home_distance = distance_from_home(pos, &ai.home_position);
                info_lines.push(format!("Distance from Home: {:.1}", home_distance));
            }

            // Add target info
            if let Some(target) = ai.current_target {
                info_lines.push(format!("Target: Entity({:?})", target));
            } else {
                info_lines.push("Target: None".to_string());
            }

            // Add energy info
            if let Ok(energy) = q_energy.get(self.entity) {
                info_lines.push(format!("Energy: {}", energy.value));
            }

            // Add pursuing info if present
            if let Ok(pursuing) = q_pursuing.get(self.entity) {
                info_lines.push("--- Pursuing Info ---".to_string());
                info_lines.push(format!(
                    "Last Seen: ({}, {}, {})",
                    pursuing.last_seen_at.0, pursuing.last_seen_at.1, pursuing.last_seen_at.2
                ));
                info_lines.push(format!("Target Zone: {}", pursuing.target_zone));
                let pursuit_duration = pursuing.pursuit_duration(clock.current_tick());
                info_lines.push(format!("Pursuit Duration: {} ticks", pursuit_duration));

                if pursuing.waiting_to_teleport {
                    info_lines.push("Status: Waiting to teleport".to_string());
                    if let Some(wait_start) = pursuing.wait_started_tick {
                        let wait_elapsed = clock.current_tick().saturating_sub(wait_start);
                        let wait_remaining =
                            pursuing.teleport_wait_duration.saturating_sub(wait_elapsed);
                        info_lines.push(format!(
                            "Teleport Wait: {} / {} ticks",
                            wait_elapsed, pursuing.teleport_wait_duration
                        ));
                        info_lines.push(format!("Time Remaining: {} ticks", wait_remaining));
                    }
                } else if pursuing.searching_at_last_position {
                    info_lines.push("Status: Searching at last known position".to_string());
                    let search_elapsed = pursuing.search_elapsed_time(clock.current_tick());
                    let search_remaining = pursuing.search_duration.saturating_sub(search_elapsed);
                    info_lines.push(format!(
                        "Search Time: {} / {} ticks",
                        search_elapsed, pursuing.search_duration
                    ));
                    info_lines.push(format!("Search Remaining: {} ticks", search_remaining));
                } else {
                    info_lines.push("Status: Actively pursuing".to_string());
                }
            }

            info_lines
        } else {
            vec!["No AI Controller found".to_string()]
        };

        // Calculate dialog height based on content (each line is 0.5 units)
        let content_height = (ai_info.len() as f32 * 0.5) + 3.0; // lines + title + button space
        let total_height = content_height.ceil();

        // Calculate centered position
        let center_x = ((screen.tile_w as f32 - self.width) / 2.0).round();
        let center_y = ((screen.tile_h as f32 - total_height) / 2.0).round();
        let centered_position = Position::new_f32(center_x, center_y, 0.0);

        let dialog_entity = cmds
            .spawn((
                Dialog::new(
                    &format!("AI Debug: {}", entity_name),
                    self.width,
                    total_height,
                ),
                centered_position.clone(),
                cleanup_component.clone(),
            ))
            .id();

        let mut content_y = 1.5; // Start after title
        let mut order = 10;

        // Add each info line individually
        for line in ai_info {
            cmds.spawn((
                DialogText {
                    value: line,
                    style: DialogTextStyle::Normal,
                },
                DialogContent {
                    parent_dialog: dialog_entity,
                    order,
                },
                Position::new_f32(
                    centered_position.x + 1.0,
                    centered_position.y + content_y,
                    centered_position.z,
                ),
                cleanup_component.clone(),
                ChildOf(dialog_entity),
            ));
            content_y += 0.5; // Move down by 0.5 units for each line
            order += 1;
        }

        // Add close button at the bottom
        let button_y = centered_position.y + total_height - 1.5;
        cmds.spawn((
            ActivatableBuilder::new("[{Y|ESC}] Close", self.close_callback)
                .with_audio(AudioKey::ButtonBack1)
                .with_hotkey(KeyCode::Escape)
                .with_focus_order(3000)
                .as_button(Layer::DialogContent),
            DialogContent {
                parent_dialog: dialog_entity,
                order: 100,
            },
            Position::new_f32(
                centered_position.x + (self.width / 2.0) - 3.0, // Center the button
                button_y,
                centered_position.z,
            ),
            cleanup_component,
            ChildOf(dialog_entity),
        ));

        dialog_entity
    }
}
