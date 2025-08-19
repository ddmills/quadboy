use std::alloc::Layout;

use bevy_ecs::prelude::*;
use macroquad::input::mouse_position;
use macroquad::prelude::*;

use crate::{cfg::{MAP_SIZE, TEXEL_SIZE_F32, TILE_SIZE, TILE_SIZE_F32, ZONE_SIZE}, rendering::{GameCamera, ScreenSize}, ui::UiLayout};

#[derive(Resource, Default)]
pub struct Mouse {
    pub px: (usize, usize),
    pub local: (f32, f32),
    pub world: (f32, f32),
    pub uv: (f32, f32),
}

fn crt_curve_uv(uv: (f32, f32)) -> (f32, f32) {
    let curvature = Vec2::new(9.0, 9.0);
    let input = Vec2::new(uv.0, uv.1);
    let mut out = input * 2.0 - 1.0;
    let offset = out.yx().abs() / curvature;
    out = out + out * offset * offset;
    out = out * 0.5 + 0.5;
    (
        out.x.clamp(0., 1.),
        out.y.clamp(0., 1.),
    )
}

pub fn update_mouse(
    mut cursor: ResMut<Mouse>,
    screen: Res<ScreenSize>,
    camera: Res<GameCamera>,
    layout: Res<UiLayout>,
) {
    let target_size = (
        (screen.tile_w * TILE_SIZE.0) as u32,
        (screen.tile_h * TILE_SIZE.1) as u32,
    );

    let screen_offset = (
        screen.width - target_size.0,
        screen.height - target_size.1,
    );

    let mouse = mouse_position();

    let px_flat = (
        ((mouse.0 - screen_offset.0 as f32) / TEXEL_SIZE_F32).clamp(0., target_size.0 as f32) as usize,
        ((mouse.1 - screen_offset.1 as f32) / TEXEL_SIZE_F32).clamp(0., target_size.1 as f32) as usize,
    );

    let flat_uv = (
        (px_flat.0 as f32 / target_size.0 as f32).clamp(0., 1.),
        (px_flat.1 as f32 / target_size.1 as f32).clamp(0., 1.),
    );
    let uv = crt_curve_uv(flat_uv);

    let px = (
        (uv.0 * target_size.0 as f32) as usize,
        (uv.1 * target_size.1 as f32) as usize,
    );

    let local = (
        (px.0 as f32 / TILE_SIZE_F32.0).clamp(0., screen.tile_w as f32),
        (px.1 as f32 / TILE_SIZE_F32.1).clamp(0., screen.tile_w as f32),
    );

    let world = (
        (camera.x + local.0 - layout.game_panel.x as f32).clamp(0., (MAP_SIZE.0 * ZONE_SIZE.0 - 1) as f32 + 0.99),
        (camera.y + local.1 - layout.game_panel.y as f32).clamp(0., (MAP_SIZE.1 * ZONE_SIZE.1 - 1) as f32 + 0.99),
    );

    cursor.px = px;
    cursor.local = local;
    cursor.uv = uv;
    cursor.world = world;
}
