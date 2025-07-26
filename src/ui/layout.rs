use bevy_ecs::prelude::*;
use macroquad::{color::WHITE, math::uvec2, texture::{draw_texture_ex, DrawTextureParams}};

use crate::{cfg::{TEXEL_SIZE_F32, TILE_SIZE, TILE_SIZE_F32}, rendering::{GameCamera, RenderTargets, ScreenSize}};

#[derive(Default)]
pub struct Panel {
    pub width: usize,
    pub height: usize,
    pub x: usize,
    pub y: usize,
}

#[derive(Resource)]
pub struct UiLayout {
    pub left_panel: Panel,
    pub game_panel: Panel,
}

impl FromWorld for UiLayout {
    fn from_world(world: &mut World) -> Self {
        Self {
            left_panel: Panel::default(),
            game_panel: Panel::default(),
        }
    }
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

pub fn render_layout(ren: Res<RenderTargets>, ui: Res<UiLayout>, screen: Res<ScreenSize>)
{
    let target_size = uvec2(screen.width, screen.height);

    // draw final texture as double size
    let dest_size = target_size.as_vec2() * TEXEL_SIZE_F32;

    draw_texture_ex(
        &ren.world.texture,
        (ui.game_panel.x * TILE_SIZE.0) as f32 * TEXEL_SIZE_F32,
        (ui.game_panel.y * TILE_SIZE.1) as f32 * TEXEL_SIZE_F32,
        WHITE,
        DrawTextureParams {
            dest_size: Some(dest_size),
            flip_y: true,
            ..Default::default()
        },
    );

    draw_texture_ex(
        &ren.screen.texture,
        0.,
        0.,
        WHITE,
        DrawTextureParams {
            dest_size: Some(dest_size),
            flip_y: true,
            ..Default::default()
        },
    );
}
