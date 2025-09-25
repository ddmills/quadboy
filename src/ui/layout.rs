use bevy_ecs::prelude::*;

use crate::{
    cfg::TILE_SIZE_F32,
    common::Palette,
    rendering::{GameCamera, Glyph, Layer, Position, ScreenSize},
    states::CleanupStatePlay,
};

#[derive(Default)]
pub struct Panel {
    pub width: usize,
    pub height: usize,
    pub x: usize,
    pub y: usize,
    pub glyphs: Vec<Entity>,
}

#[derive(Resource, Default)]
pub struct UiLayout {
    pub left_panel: Panel,
    pub game_panel: Panel,
}

pub fn update_ui_layout(
    mut ui: ResMut<UiLayout>,
    screen: Res<ScreenSize>,
    mut camera: ResMut<GameCamera>,
) {
    // let left_panel_width = 0;
    let left_panel_width = 10;

    ui.left_panel.x = 0;
    ui.left_panel.y = 0;
    ui.left_panel.width = left_panel_width;
    ui.left_panel.height = screen.tile_h;

    ui.game_panel.x = ui.left_panel.width;
    ui.game_panel.y = 0;
    if screen.tile_w > ui.left_panel.width {
        ui.game_panel.width = screen.tile_w - ui.left_panel.width;
    }
    ui.game_panel.height = screen.tile_h;

    camera.width = ui.game_panel.width as f32 * TILE_SIZE_F32.0;
    camera.height = ui.game_panel.height as f32 * TILE_SIZE_F32.1;
}

pub fn draw_ui_panels(mut cmds: Commands, mut ui: ResMut<UiLayout>) {
    for e in ui.left_panel.glyphs.iter() {
        cmds.entity(*e).try_despawn();
    }

    ui.left_panel.glyphs = vec![];

    let color = Palette::Clear;

    for x in 0..ui.left_panel.width {
        for y in 0..ui.left_panel.height {
            let e = cmds
                .spawn((
                    CleanupStatePlay,
                    Position::new(x, y, 0),
                    Glyph::new(0, color, color).bg(color).layer(Layer::UiPanels),
                ))
                .id();

            ui.left_panel.glyphs.push(e);
        }
    }
}
