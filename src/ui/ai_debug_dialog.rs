use bevy_ecs::{prelude::*, system::SystemId};
use macroquad::input::KeyCode;

use crate::{
    domain::{AiController, Energy, Label},
    engine::{AudioKey, Clock},
    rendering::{Layer, Position, ScreenSize},
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
        q_energy: &Query<&Energy>,
        q_positions: &Query<&Position>,
        clock: &Clock,
        cleanup_component: impl Bundle + Clone,
        screen: &ScreenSize,
    ) -> Entity {
        let entity_name = if let Ok(label) = q_labels.get(self.entity) {
            label.get().to_string()
        } else {
            "Unknown".to_string()
        };

        let ai_info = if let Ok(ai) = q_ai_controllers.get(self.entity) {
            let mut info_lines = vec![
                format!("State: {:?}", ai.state),
                format!("Template: {:?}", ai.template),
                format!(
                    "Home: ({}, {}, {})",
                    ai.home_position.0, ai.home_position.1, ai.home_position.2
                ),
                format!("Leash Range: {:.1}", ai.leash_range),
                format!("Wander Range: {:.1}", ai.wander_range),
                format!("Detection Range: {:.1}", ai.detection_range),
            ];

            // Add current position and distance from home
            if let Ok(pos) = q_positions.get(self.entity) {
                let (x, y, z) = pos.world();
                info_lines.push(format!("Current: ({}, {}, {})", x, y, z));
            }

            // Add target info
            if let Some(target_stable_id) = ai.current_target_id {
                info_lines.push(format!("Target: StableId({:?})", target_stable_id));
            } else {
                info_lines.push("Target: None".to_string());
            }

            // Add energy info
            if let Ok(energy) = q_energy.get(self.entity) {
                info_lines.push(format!("Energy: {}", energy.value));
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
