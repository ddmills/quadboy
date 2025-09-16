use bevy_ecs::prelude::*;

use crate::{
    common::Palette,
    rendering::{Glyph, GlyphTextureId, Layer, Position, ScreenSize, Text},
};

/// Resource to track if any dialog is currently open
#[derive(Resource, Default)]
pub struct DialogState {
    pub is_open: bool,
}

/// Main dialog component with title and dimensions
#[derive(Component)]
pub struct Dialog {
    pub title: String,
    pub width: f32,
    pub height: f32,
}

impl Dialog {
    pub fn new(title: &str, width: f32, height: f32) -> Self {
        Self {
            title: title.to_string(),
            width,
            height,
        }
    }
}

/// Marker component for dialog background elements that render on UiPanels layer
#[derive(Component)]
pub struct DialogBackground;

/// Marker component for dialog border elements
#[derive(Component)]
pub struct DialogBorder;

/// Component that marks content belonging to a specific dialog
#[derive(Component, Clone)]
pub struct DialogContent {
    pub parent_dialog: Entity,
    pub order: usize,
}

/// Text content within a dialog
#[derive(Component)]
pub struct DialogText {
    pub value: String,
    pub style: DialogTextStyle,
}

#[derive(Clone)]
pub enum DialogTextStyle {
    Normal,
    Title,
    Property,
    Description,
}

/// Icon/glyph display within a dialog
#[derive(Component)]
pub struct DialogIcon {
    pub glyph_idx: usize,
    pub scale: f32,
    pub fg1: Option<u32>,
    pub fg2: Option<u32>,
    pub texture_id: GlyphTextureId,
}

/// Key-value property display (e.g., "Weight: 3.0 kg")
#[derive(Component)]
pub struct DialogProperty {
    pub label: String,
    pub value: String,
}

/// Visual divider/separator
#[derive(Component)]
pub struct DialogDivider;

pub fn setup_dialogs(
    mut cmds: Commands,
    mut q_dialogs: Query<(Entity, &Dialog, &Position), Changed<Dialog>>,
    mut dialog_state: ResMut<DialogState>,
) {
    for (dialog_entity, dialog, dialog_pos) in q_dialogs.iter_mut() {
        dialog_state.is_open = true;

        cmds.spawn((
            DialogBackground,
            DialogContent {
                parent_dialog: dialog_entity,
                order: 0,
            },
            Glyph::idx(6) // Solid fill
                .scale((dialog.width, dialog.height))
                .layer(Layer::DialogPanels)
                .bg(Palette::Clear),
            Position::new_f32(dialog_pos.x, dialog_pos.y, dialog_pos.z),
            ChildOf(dialog_entity),
        ));

        // Create border (on UiPanels layer, renders after background)
        create_dialog_border(
            &mut cmds,
            dialog_entity,
            dialog_pos,
            dialog.width,
            dialog.height,
        );

        // Create title bar background
        if !dialog.title.is_empty() {
            cmds.spawn((
                DialogBackground,
                DialogContent {
                    parent_dialog: dialog_entity,
                    order: 1,
                },
                Glyph::idx(6)
                    .scale((dialog.width - 2.0, 1.0))
                    .layer(Layer::DialogPanels)
                    .bg(Palette::Clear),
                Position::new_f32(dialog_pos.x + 1.0, dialog_pos.y + 1.0, dialog_pos.z),
                ChildOf(dialog_entity),
            ));

            // Create title text (on Ui layer, renders on top)
            cmds.spawn((
                DialogText {
                    value: dialog.title.clone(),
                    style: DialogTextStyle::Title,
                },
                DialogContent {
                    parent_dialog: dialog_entity,
                    order: 2,
                },
                Text::new(&dialog.title)
                    .fg1(Palette::White)
                    .layer(Layer::DialogContent),
                Position::new_f32(dialog_pos.x + 2.0, dialog_pos.y + 1.0, dialog_pos.z),
                ChildOf(dialog_entity),
            ));
        }
    }
}

/// Create border glyphs for the dialog
fn create_dialog_border(
    cmds: &mut Commands,
    dialog_entity: Entity,
    dialog_pos: &Position,
    width: f32,
    height: f32,
) {
    let border_fg1 = Palette::Gray;
    let border_fg2 = Palette::DarkOrange;

    // Top and bottom horizontal borders
    for x in 0..(width as usize) {
        // Top border
        let glyph_idx = if x == 0 {
            218 // Top-left corner ┌
        } else if x == (width as usize) - 1 {
            // 229 // Top-right corner ┐
            220 // Top-right corner ┐
        } else {
            // 196 // Horizontal line ─
            219 // Horizontal line ─
        };

        cmds.spawn((
            DialogBorder,
            DialogContent {
                parent_dialog: dialog_entity,
                order: 0,
            },
            Glyph::idx(glyph_idx)
                .layer(Layer::DialogPanels)
                .fg1(border_fg1)
                .fg2(border_fg2),
            Position::new_f32(dialog_pos.x + x as f32, dialog_pos.y, dialog_pos.z),
            ChildOf(dialog_entity),
        ));

        // Bottom border
        let glyph_idx = if x == 0 {
            // 192 // Bottom-left corner └
            250 // Bottom-left corner └
        } else if x == (width as usize) - 1 {
            // 217 // Bottom-right corner ┘
            252 // Bottom-right corner ┘
        } else {
            // 196 // Horizontal line ─
            251 // Horizontal line ─
        };

        cmds.spawn((
            DialogBorder,
            DialogContent {
                parent_dialog: dialog_entity,
                order: 0,
            },
            Glyph::idx(glyph_idx)
                .layer(Layer::DialogPanels)
                .fg1(border_fg1)
                .fg2(border_fg2),
            Position::new_f32(
                dialog_pos.x + x as f32,
                dialog_pos.y + height - 1.0,
                dialog_pos.z,
            ),
            ChildOf(dialog_entity),
        ));
    }

    // Left and right vertical borders
    for y in 1..((height as usize) - 1) {
        // Left border
        cmds.spawn((
            DialogBorder,
            DialogContent {
                parent_dialog: dialog_entity,
                order: 0,
            },
            Glyph::idx(234) // Vertical line │
                .layer(Layer::DialogPanels)
                .fg1(border_fg1)
                .fg2(border_fg2),
            Position::new_f32(dialog_pos.x, dialog_pos.y + y as f32, dialog_pos.z),
            ChildOf(dialog_entity),
        ));

        // Right border
        cmds.spawn((
            DialogBorder,
            DialogContent {
                parent_dialog: dialog_entity,
                order: 0,
            },
            Glyph::idx(236) // Vertical line │
                .layer(Layer::DialogPanels)
                .fg1(border_fg1)
                .fg2(border_fg2),
            Position::new_f32(
                dialog_pos.x + width - 1.0,
                dialog_pos.y + y as f32,
                dialog_pos.z,
            ),
            ChildOf(dialog_entity),
        ));
    }
}

pub fn render_dialog_content(
    mut cmds: Commands,
    q_text: Query<(Entity, &DialogText, &Position), (With<DialogContent>, Without<Text>)>,
    q_icons: Query<(Entity, &DialogIcon, &Position), (With<DialogContent>, Without<Glyph>)>,
    q_properties: Query<(Entity, &DialogProperty, &Position), (With<DialogContent>, Without<Text>)>,
    q_dividers: Query<(Entity, &DialogDivider, &Position), (With<DialogContent>, Without<Glyph>)>,
) {
    // Render DialogText components as Text
    for (entity, dialog_text, _pos) in q_text.iter() {
        let color = match dialog_text.style {
            DialogTextStyle::Title => Palette::White,
            DialogTextStyle::Normal => Palette::White,
            DialogTextStyle::Property => Palette::Gray,
            DialogTextStyle::Description => Palette::White,
        };

        if let Ok(mut entity_cmds) = cmds.get_entity(entity) {
            entity_cmds.insert(
                Text::new(&dialog_text.value)
                    .fg1(color)
                    .layer(Layer::DialogContent),
            );
        }
    }

    // Render DialogIcon components as Glyph
    for (entity, dialog_icon, _pos) in q_icons.iter() {
        let mut glyph = Glyph::idx(dialog_icon.glyph_idx)
            .scale((dialog_icon.scale, dialog_icon.scale))
            .layer(Layer::DialogContent)
            .texture(dialog_icon.texture_id)
            .fg1_opt(dialog_icon.fg1);

        if let Some(fg2) = dialog_icon.fg2 {
            glyph = glyph.fg2(fg2);
        }

        if let Ok(mut entity_cmds) = cmds.get_entity(entity) {
            entity_cmds.insert(glyph);
        }
    }

    // Render DialogProperty components as Text
    for (entity, property, _pos) in q_properties.iter() {
        let display_text = format!("{}: {}", property.label, property.value);
        if let Ok(mut entity_cmds) = cmds.get_entity(entity) {
            entity_cmds.insert(
                Text::new(&display_text)
                    .fg1(Palette::White)
                    .layer(Layer::DialogContent),
            );
        }
    }

    // Render DialogDivider components as horizontal lines
    for (entity, _divider, _pos) in q_dividers.iter() {
        if let Ok(mut entity_cmds) = cmds.get_entity(entity) {
            entity_cmds.insert(
                Glyph::idx(196) // Horizontal line ─
                    .scale((20.0, 1.0))
                    .layer(Layer::DialogPanels)
                    .fg1(Palette::Gray),
            );
        }
    }
}

pub fn center_dialogs_on_screen_change(
    screen: Res<ScreenSize>,
    mut q_dialogs: Query<(Entity, &Dialog, &mut Position)>,
    mut q_dialog_children: Query<&mut Position, (With<DialogContent>, Without<Dialog>)>,
    children: Query<&Children>,
) {
    for (dialog_entity, dialog, mut position) in q_dialogs.iter_mut() {
        let center_x = ((screen.tile_w as f32 - dialog.width) / 2.0).round();
        let center_y = ((screen.tile_h as f32 - dialog.height) / 2.0).round();

        // Calculate position deltat
        let delta_x = center_x - position.x;
        let delta_y = center_y - position.y;

        // Update dialog position
        position.x = center_x;
        position.y = center_y;

        // Update all child positions by the same delta
        if let Ok(dialog_children) = children.get(dialog_entity) {
            for child_entity in dialog_children.iter() {
                if let Ok(mut child_position) = q_dialog_children.get_mut(child_entity) {
                    child_position.x += delta_x;
                    child_position.y += delta_y;
                }
            }
        }
    }
}
