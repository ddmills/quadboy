use bevy_ecs::prelude::*;

use crate::{cfg::TILE_SIZE_F32, rendering::{GameCamera, ScreenSize}};

#[derive(Default)]
pub struct Panel {
    pub width: usize,
    pub height: usize,
    pub x: usize,
    pub y: usize,
}

#[derive(Resource, Default)]
pub struct UiLayout {
    pub left_panel: Panel,
    pub game_panel: Panel,
}

pub fn update_ui_layout(mut ui: ResMut<UiLayout>, screen: Res<ScreenSize>, mut camera: ResMut<GameCamera>) {
    // let left_panel_width = 8;
    let left_panel_width = 0;

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
