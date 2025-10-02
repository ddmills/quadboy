use bevy_ecs::prelude::*;
use macroquad::prelude::trace;

use crate::{
    cfg::TILE_SIZE_F32,
    common::Palette,
    rendering::{GameCamera, Glyph, GlyphTextureId, Layer, Position, ScreenSize, Text},
    states::CleanupStateExplore,
};

#[derive(Default)]
pub struct Panel {
    pub width: usize,
    pub height: usize,
    pub x: usize,
    pub y: usize,
    pub glyph: Option<Entity>,
    pub border_entities: Vec<Entity>,
}

#[derive(Resource, Default)]
pub struct UiLayout {
    pub top_panel: Panel,
    pub left_panel: Panel,
    pub game_panel: Panel,
    pub bottom_panel: Panel,
}

pub fn update_ui_layout(
    mut ui: ResMut<UiLayout>,
    screen: Res<ScreenSize>,
    mut camera: ResMut<GameCamera>,
) {
    // let left_panel_width = 0;
    let left_panel_width = 10;
    let bottom_panel_height = 5;
    let top_panel_height = 2;

    // Top panel takes full width at top of screen
    ui.top_panel.x = 0;
    ui.top_panel.y = 0;
    ui.top_panel.width = screen.tile_w;
    ui.top_panel.height = top_panel_height;

    // Left panel starts below top panel
    ui.left_panel.x = 0;
    ui.left_panel.y = top_panel_height;
    ui.left_panel.width = left_panel_width;
    ui.left_panel.height = screen.tile_h - top_panel_height;

    // Bottom panel positioned to the right of left panel, at bottom of screen
    ui.bottom_panel.x = left_panel_width;
    ui.bottom_panel.y = screen.tile_h - bottom_panel_height;
    if screen.tile_w > left_panel_width {
        ui.bottom_panel.width = screen.tile_w - left_panel_width;
    }
    ui.bottom_panel.height = bottom_panel_height;

    // Game panel takes remaining space (excluding bottom panel and top panel)
    ui.game_panel.x = ui.left_panel.width;
    ui.game_panel.y = top_panel_height;
    if screen.tile_w > ui.left_panel.width {
        ui.game_panel.width = screen.tile_w - ui.left_panel.width;
    }
    ui.game_panel.height = screen.tile_h - bottom_panel_height - top_panel_height;

    camera.width = ui.game_panel.width as f32 * TILE_SIZE_F32.0;
    camera.height = ui.game_panel.height as f32 * TILE_SIZE_F32.1;
}

pub fn draw_ui_panels(
    mut cmds: Commands,
    mut ui: ResMut<UiLayout>,
    mut q_glyphs: Query<&mut Glyph>,
    mut q_positions: Query<&mut Position>,
) {
    let color = Palette::Clear;

    // Handle top panel
    if let Some(entity) = ui.top_panel.glyph {
        // Check if entity still exists
        if cmds.get_entity(entity).is_ok() {
            // Update existing entity
            if let Ok(mut glyph) = q_glyphs.get_mut(entity) {
                glyph.scale = (ui.top_panel.width as f32, ui.top_panel.height as f32);
            }
            if let Ok(mut position) = q_positions.get_mut(entity) {
                position.x = ui.top_panel.x as f32;
                position.y = ui.top_panel.y as f32;
            }
        } else {
            // Entity was despawned, clear the reference
            ui.top_panel.glyph = None;
        }
    }

    if ui.top_panel.width > 0 && ui.top_panel.height > 0 {
        if ui.top_panel.glyph.is_none() {
            // Create new entity
            let entity = cmds
                .spawn((
                    CleanupStateExplore,
                    Position::new(ui.top_panel.x, ui.top_panel.y, 0),
                    Glyph::new(0, color, color)
                        .bg(color)
                        .scale((ui.top_panel.width as f32, ui.top_panel.height as f32))
                        .layer(Layer::UiPanels),
                ))
                .id();
            ui.top_panel.glyph = Some(entity);
        }
    } else {
        // Panel has no size, remove if it exists
        if let Some(entity) = ui.top_panel.glyph {
            cmds.entity(entity).try_despawn();
            ui.top_panel.glyph = None;
        }
    }

    // Handle left panel
    if let Some(entity) = ui.left_panel.glyph {
        // Check if entity still exists
        if cmds.get_entity(entity).is_ok() {
            // Update existing entity
            if let Ok(mut glyph) = q_glyphs.get_mut(entity) {
                glyph.scale = (ui.left_panel.width as f32, ui.left_panel.height as f32);
            }
            if let Ok(mut position) = q_positions.get_mut(entity) {
                position.x = ui.left_panel.x as f32;
                position.y = ui.left_panel.y as f32;
            }
        } else {
            // Entity was despawned, clear the reference
            ui.left_panel.glyph = None;
        }
    }

    if ui.left_panel.width > 0 && ui.left_panel.height > 0 {
        if ui.left_panel.glyph.is_none() {
            // Create new entity
            let entity = cmds
                .spawn((
                    CleanupStateExplore,
                    Position::new(ui.left_panel.x, ui.left_panel.y, 0),
                    Glyph::new(0, color, color)
                        .bg(color)
                        .scale((ui.left_panel.width as f32, ui.left_panel.height as f32))
                        .layer(Layer::UiPanels),
                ))
                .id();
            ui.left_panel.glyph = Some(entity);
        }
    } else {
        // Panel has no size, remove if it exists
        if let Some(entity) = ui.left_panel.glyph {
            cmds.entity(entity).try_despawn();
            ui.left_panel.glyph = None;
        }
    }

    // Handle bottom panel
    if let Some(entity) = ui.bottom_panel.glyph {
        // Check if entity still exists
        if cmds.get_entity(entity).is_ok() {
            // Update existing entity
            if let Ok(mut glyph) = q_glyphs.get_mut(entity) {
                glyph.scale = (ui.bottom_panel.width as f32, ui.bottom_panel.height as f32);
            }
            if let Ok(mut position) = q_positions.get_mut(entity) {
                position.x = ui.bottom_panel.x as f32;
                position.y = ui.bottom_panel.y as f32;
            }
        } else {
            // Entity was despawned, clear the reference
            ui.bottom_panel.glyph = None;
        }
    }

    if ui.bottom_panel.width > 0 && ui.bottom_panel.height > 0 {
        if ui.bottom_panel.glyph.is_none() {
            // Create new entity
            let entity = cmds
                .spawn((
                    CleanupStateExplore,
                    Position::new(ui.bottom_panel.x, ui.bottom_panel.y, 0),
                    Glyph::new(0, color, color)
                        .bg(color)
                        .scale((ui.bottom_panel.width as f32, ui.bottom_panel.height as f32))
                        .layer(Layer::UiPanels),
                ))
                .id();
            ui.bottom_panel.glyph = Some(entity);
        }
    } else {
        // Panel has no size, remove if it exists
        if let Some(entity) = ui.bottom_panel.glyph {
            cmds.entity(entity).try_despawn();
            ui.bottom_panel.glyph = None;
        }
    }

    // Draw borders for panels (excluding game panel)
    draw_panel_borders(&mut cmds, &mut ui.top_panel, true);
    draw_panel_borders(&mut cmds, &mut ui.left_panel, true);
    draw_panel_borders(&mut cmds, &mut ui.bottom_panel, true);
    draw_panel_borders(&mut cmds, &mut ui.game_panel, false);
}

fn draw_panel_borders(cmds: &mut Commands, panel: &mut Panel, has_borders: bool) {
    if !has_borders || panel.width == 0 || panel.height == 0 {
        // Remove existing borders if they exist
        for entity in panel.border_entities.drain(..) {
            cmds.entity(entity).try_despawn();
        }
        return;
    }

    // Clear existing borders
    for entity in panel.border_entities.drain(..) {
        cmds.entity(entity).try_despawn();
    }

    let border_color = Palette::DarkGreen;

    // Convert panel coordinates to text coordinates (multiply by 2)
    let text_x = panel.x * 2;
    let text_y = panel.y * 2;
    let text_width = panel.width * 2;
    let text_height = panel.height * 2;

    // Draw corners
    let corners = [
        (text_x, text_y, '┌'),                                    // top-left
        (text_x + text_width - 1, text_y, '┐'),                   // top-right
        (text_x, text_y + text_height - 1, '└'),                  // bottom-left
        (text_x + text_width - 1, text_y + text_height - 1, '┘'), // bottom-right
    ];

    for (x, y, ch) in corners {
        let entity = cmds
            .spawn((
                CleanupStateExplore,
                Text::new(&ch.to_string())
                    .fg1(border_color)
                    .layer(Layer::Ui)
                    .texture(GlyphTextureId::BodyFont),
                Position::new_f32(x as f32 * 0.5, y as f32 * 0.5, 0.0),
            ))
            .id();
        panel.border_entities.push(entity);
    }

    // Draw horizontal lines (top and bottom)
    for x in (text_x + 1)..(text_x + text_width - 1) {
        // Top edge
        let entity = cmds
            .spawn((
                CleanupStateExplore,
                Text::new("─")
                    .fg1(border_color)
                    .layer(Layer::Ui)
                    .texture(GlyphTextureId::BodyFont),
                Position::new_f32(x as f32 * 0.5, text_y as f32 * 0.5, 0.0),
            ))
            .id();
        panel.border_entities.push(entity);

        // Bottom edge
        let entity = cmds
            .spawn((
                CleanupStateExplore,
                Text::new("─")
                    .fg1(border_color)
                    .layer(Layer::Ui)
                    .texture(GlyphTextureId::BodyFont),
                Position::new_f32(x as f32 * 0.5, (text_y + text_height - 1) as f32 * 0.5, 0.0),
            ))
            .id();
        panel.border_entities.push(entity);
    }

    // Draw vertical lines (left and right)
    for y in (text_y + 1)..(text_y + text_height - 1) {
        // Left edge
        let entity = cmds
            .spawn((
                CleanupStateExplore,
                Text::new("│")
                    .fg1(border_color)
                    .layer(Layer::Ui)
                    .texture(GlyphTextureId::BodyFont),
                Position::new_f32(text_x as f32 * 0.5, y as f32 * 0.5, 0.0),
            ))
            .id();
        panel.border_entities.push(entity);

        // Right edge
        let entity = cmds
            .spawn((
                CleanupStateExplore,
                Text::new("│")
                    .fg1(border_color)
                    .layer(Layer::Ui)
                    .texture(GlyphTextureId::BodyFont),
                Position::new_f32((text_x + text_width - 1) as f32 * 0.5, y as f32 * 0.5, 0.0),
            ))
            .id();
        panel.border_entities.push(entity);
    }
}
