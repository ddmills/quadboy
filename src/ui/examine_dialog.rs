use bevy_ecs::{prelude::*, system::SystemId};
use macroquad::input::KeyCode;

use crate::{
    domain::{Description, Label},
    engine::{AudioKey, StableId},
    rendering::{Glyph, Layer, Position, ScreenSize, text_content_length, wrap_text},
    ui::{ActivatableBuilder, Dialog, DialogContent, DialogIcon, DialogText, DialogTextStyle},
};

pub struct ExamineDialogBuilder {
    entity: Entity,
    width: f32,
    close_callback: SystemId,
    relationship_text: Option<String>,
}

impl ExamineDialogBuilder {
    pub fn new(entity: Entity, close_callback: SystemId) -> Self {
        Self {
            entity,
            width: 24.0,
            close_callback,
            relationship_text: None,
        }
    }

    pub fn with_relationship_text(mut self, relationship_text: Option<String>) -> Self {
        self.relationship_text = relationship_text;
        self
    }

    pub fn spawn(
        self,
        cmds: &mut Commands,
        q_labels: &Query<&Label>,
        q_descriptions: &Query<&Description>,
        q_glyphs: &Query<&Glyph>,
        q_stable_ids: &Query<&StableId>,
        cleanup_component: impl Bundle + Clone,
        screen: &ScreenSize,
    ) -> Entity {
        let entity_name = if let Ok(label) = q_labels.get(self.entity) {
            label.get().to_string()
        } else {
            "Unknown".to_string()
        };

        // Wrap the entity name for long titles
        let available_width = ((self.width as usize).saturating_sub(2)) * 2; // Account for 0.5-width text chars
        let title_lines = wrap_text(&entity_name, available_width.max(20));
        let title_height = title_lines.len() as f32 * 0.5; // 0.5 units per line

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

        let relationship_height = if self.relationship_text.is_some() {
            0.5 // 0.5 height for text
        } else {
            0.0
        };

        let description_gap = if self.relationship_text.is_some() && !description_lines.is_empty() {
            0.5 // Extra gap before description when relationship text is present
        } else {
            0.0
        };

        let gap_after_title = 0.5;
        let gap_before_button = if !description_lines.is_empty() || self.relationship_text.is_some()
        {
            0.5
        } else {
            0.5
        };

        let total_height = (2.0
            + title_height
            + gap_after_title
            + relationship_height
            + description_gap
            + description_height
            + gap_before_button
            + 2.0)
            .ceil(); // Icon + title + gap + relationship + gap + description + gap + button, rounded up

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

        // Add centered entity name (wrapped if necessary)
        for (i, line) in title_lines.iter().enumerate() {
            let line_visual_length = text_content_length(line);
            let centered_x =
                centered_position.x + (self.width / 2.0) - (line_visual_length as f32 * 0.25); // 0.25 = 0.5 width / 2 for centering

            cmds.spawn((
                DialogText {
                    value: line.clone(),
                    style: DialogTextStyle::Title,
                },
                DialogContent {
                    parent_dialog: dialog_entity,
                    order: order + i,
                },
                Position::new_f32(
                    centered_x,
                    centered_position.y + content_y + (i as f32 * 0.5), // 0.5 units per line
                    centered_position.z,
                ),
                cleanup_component.clone(),
                ChildOf(dialog_entity),
            ));
        }

        content_y += title_height; // Add space for all title lines
        content_y += 0.5; // Gap after title
        order += title_lines.len();

        // Add relationship text directly under the name if provided
        if let Some(relationship_text) = &self.relationship_text {
            let relationship_visual_length = text_content_length(relationship_text);
            let relationship_x = centered_position.x + (self.width / 2.0)
                - (relationship_visual_length as f32 * 0.25);

            cmds.spawn((
                DialogText {
                    value: relationship_text.clone(),
                    style: DialogTextStyle::Normal,
                },
                DialogContent {
                    parent_dialog: dialog_entity,
                    order,
                },
                Position::new_f32(
                    relationship_x,
                    centered_position.y + content_y,
                    centered_position.z,
                ),
                cleanup_component.clone(),
                ChildOf(dialog_entity),
            ));

            content_y += 0.5; // Space for relationship text
            order += 1;

            // Add gap before description if description exists
            if !description_lines.is_empty() {
                content_y += 0.5; // Extra gap before description
            }
        }

        // Add description if available
        if !description_lines.is_empty() {
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

        // Add StableId in bottom-right corner if available
        if let Ok(stable_id) = q_stable_ids.get(self.entity) {
            let id_text = format!("{{b|{}}}", stable_id.0);
            let id_visual_length = text_content_length(&id_text);

            cmds.spawn((
                DialogText {
                    value: id_text,
                    style: DialogTextStyle::Normal,
                },
                DialogContent {
                    parent_dialog: dialog_entity,
                    order: 21,
                },
                Position::new_f32(
                    centered_position.x + self.width - (id_visual_length as f32 * 0.5) - 1.0, // Right edge minus text width
                    centered_position.y + total_height - 1.0, // Bottom edge
                    centered_position.z,
                ),
                cleanup_component.clone(),
                ChildOf(dialog_entity),
            ));
        }

        dialog_entity
    }
}

pub fn spawn_examine_dialog(
    world: &mut World,
    entity: Entity,
    player_entity: Entity,
    close_callback: SystemId,
) {
    let cmd = crate::ui::SpawnExamineDialogCommand::new(entity, player_entity, close_callback);
    cmd.apply(world);
}
