use bevy_ecs::{prelude::*, system::SystemId};
use macroquad::input::KeyCode;

use crate::{
    domain::{Description, Label},
    engine::AudioKey,
    rendering::{Glyph, Layer, Position, ScreenSize, text_content_length, wrap_text},
    ui::{ActivatableBuilder, Dialog, DialogContent, DialogIcon, DialogText, DialogTextStyle},
};

pub struct ExamineDialogBuilder {
    entity: Entity,
    position: Position,
    width: f32,
    height: f32,
    close_callback: SystemId,
}

impl ExamineDialogBuilder {
    pub fn new(entity: Entity, close_callback: SystemId) -> Self {
        Self {
            entity,
            position: Position::new_f32(0.0, 0.0, 0.0), // Will be centered by dialog system
            width: 24.0,
            height: 12.0, // Will be adjusted based on description
            close_callback,
        }
    }

    pub fn with_position(mut self, x: f32, y: f32) -> Self {
        self.position = Position::new_f32(x, y, 0.0);
        self
    }

    pub fn with_size(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn spawn(
        self,
        cmds: &mut Commands,
        q_labels: &Query<&Label>,
        q_descriptions: &Query<&Description>,
        q_glyphs: &Query<&Glyph>,
        cleanup_component: impl Bundle + Clone,
        screen: &ScreenSize,
    ) -> Entity {
        let entity_name = if let Ok(label) = q_labels.get(self.entity) {
            label.get().to_string()
        } else {
            "Unknown".to_string()
        };

        // Calculate height based on actual text wrapping
        let description_lines = if let Ok(description) = q_descriptions.get(self.entity) {
            let available_width = ((self.width as usize).saturating_sub(2)) * 2; // Account for 0.5-width text chars
            wrap_text(description.get(), available_width.max(20)) // Minimum width adjusted for 0.5-width chars
        } else {
            vec![]
        };

        let description_height = if description_lines.is_empty() {
            0.0
        } else {
            (description_lines.len() as f32 * 0.5) + 0.5 // 0.5 units per line + spacing
        };

        let total_height = (4.0 + description_height + 2.0).ceil(); // Icon + name + description + button, rounded up

        // Calculate centered position before creating dialog and children
        let center_x = ((screen.tile_w as f32 - self.width) / 2.0).round();
        let center_y = ((screen.tile_h as f32 - total_height) / 2.0).round();
        let centered_position = Position::new_f32(center_x, center_y, 0.0);

        let dialog_entity = cmds
            .spawn((
                Dialog::new("", self.width, total_height),
                centered_position.clone(),
                cleanup_component.clone(),
            ))
            .id();

        let mut content_y = 1.0;
        let mut order = 10;

        // Add centered glyph
        if let Ok(glyph) = q_glyphs.get(self.entity) {
            cmds.spawn((
                DialogIcon {
                    glyph_idx: glyph.idx,
                    scale: 2.0,
                    fg1: glyph.fg1,
                    fg2: glyph.fg2,
                    texture_id: glyph.texture_id,
                },
                DialogContent {
                    parent_dialog: dialog_entity,
                    order,
                },
                Position::new_f32(
                    centered_position.x + (self.width / 2.0) - 1.0,
                    centered_position.y + content_y,
                    centered_position.z,
                ),
                cleanup_component.clone(),
                ChildOf(dialog_entity),
            ));
        }

        content_y += 2.0; // Space for glyph
        order += 1;

        // Add centered entity name
        cmds.spawn((
            DialogText {
                value: entity_name.clone(),
                style: DialogTextStyle::Title,
            },
            DialogContent {
                parent_dialog: dialog_entity,
                order,
            },
            Position::new_f32(
                centered_position.x + (self.width / 2.0)
                    - (text_content_length(&entity_name) as f32 * 0.25),
                centered_position.y + content_y,
                centered_position.z,
            ),
            cleanup_component.clone(),
            ChildOf(dialog_entity),
        ));

        content_y += 1.0; // Space for name
        order += 1;

        // Add description if available
        if !description_lines.is_empty() {
            // No additional spacing - move up by 0.5 from previous value

            for (i, line) in description_lines.iter().enumerate() {
                let line_visual_length = text_content_length(line);
                let centered_x =
                    centered_position.x + (self.width / 2.0) - (line_visual_length as f32 * 0.25);

                cmds.spawn((
                    DialogText {
                        value: line.clone(),
                        style: DialogTextStyle::Normal,
                    },
                    DialogContent {
                        parent_dialog: dialog_entity,
                        order: order + i,
                    },
                    Position::new_f32(
                        centered_x,
                        centered_position.y + content_y + (i as f32 * 0.5),
                        centered_position.z,
                    ),
                    cleanup_component.clone(),
                    ChildOf(dialog_entity),
                ));
            }
        }

        // Add close button at the bottom
        let button_y = centered_position.y + total_height - 2.0;
        cmds.spawn((
            ActivatableBuilder::new("[{Y|ESC}] Close", self.close_callback)
                .with_audio(AudioKey::ButtonBack1)
                .with_hotkey(KeyCode::Escape)
                .with_focus_order(3000)
                .as_button(Layer::DialogContent),
            DialogContent {
                parent_dialog: dialog_entity,
                order: 20,
            },
            Position::new_f32(
                centered_position.x + (self.width / 2.0) - 3.0, // Center the button
                button_y,
                centered_position.z,
            ),
            cleanup_component.clone(),
            ChildOf(dialog_entity),
        ));

        dialog_entity
    }
}

pub fn spawn_examine_dialog(
    cmds: &mut Commands,
    entity: Entity,
    close_callback: SystemId,
    q_labels: &Query<&Label>,
    q_descriptions: &Query<&Description>,
    q_glyphs: &Query<&Glyph>,
    cleanup_component: impl Bundle + Clone,
    screen: &ScreenSize,
) -> Entity {
    ExamineDialogBuilder::new(entity, close_callback).spawn(
        cmds,
        q_labels,
        q_descriptions,
        q_glyphs,
        cleanup_component,
        screen,
    )
}
