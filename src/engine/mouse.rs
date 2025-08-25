use bevy_ecs::prelude::*;
use macroquad::input::mouse_position;
use macroquad::prelude::*;

use crate::{
    cfg::{MAP_SIZE, TEXEL_SIZE_F32, TILE_SIZE, TILE_SIZE_F32, ZONE_SIZE},
    domain::GameSettings,
    rendering::{GameCamera, ScreenSize},
    ui::UiLayout,
};

#[derive(Resource, Default)]
pub struct Mouse {
    pub px: (usize, usize),
    pub ui: (f32, f32),
    pub world: (f32, f32),
    pub uv: (f32, f32),
}

fn crt_curve_uv(uv: (f32, f32), crt_curvature: &crate::rendering::CrtCurvature) -> (f32, f32) {
    if !crt_curvature.is_enabled() {
        return uv;
    }

    let curve_values = crt_curvature.get_values();
    let curvature = Vec2::new(curve_values.0, curve_values.1);
    let input = Vec2::new(uv.0, uv.1);
    let mut out = input * 2.0 - 1.0;
    let offset = out.yx().abs() / curvature;
    out = out + out * offset * offset;
    out = out * 0.5 + 0.5;
    (out.x.clamp(0., 1.), out.y.clamp(0., 1.))
}

pub fn update_mouse(
    mut cursor: ResMut<Mouse>,
    screen: Res<ScreenSize>,
    camera: Res<GameCamera>,
    layout: Res<UiLayout>,
    settings: Res<GameSettings>,
) {
    let target_size = (
        (screen.tile_w * TILE_SIZE.0) as u32,
        (screen.tile_h * TILE_SIZE.1) as u32,
    );

    let screen_offset = (screen.width - target_size.0, screen.height - target_size.1);

    let mouse = mouse_position();

    let px_flat = (
        ((mouse.0 - screen_offset.0 as f32) / TEXEL_SIZE_F32).clamp(0., target_size.0 as f32)
            as usize,
        ((mouse.1 - screen_offset.1 as f32) / TEXEL_SIZE_F32).clamp(0., target_size.1 as f32)
            as usize,
    );

    let flat_uv = (
        (px_flat.0 as f32 / target_size.0 as f32).clamp(0., 1.),
        (px_flat.1 as f32 / target_size.1 as f32).clamp(0., 1.),
    );
    let uv = crt_curve_uv(flat_uv, &settings.crt_curvature);

    let px = (
        (uv.0 * target_size.0 as f32) as usize,
        (uv.1 * target_size.1 as f32) as usize,
    );

    let ui = (
        (px.0 as f32 / TILE_SIZE_F32.0).clamp(0., screen.tile_w as f32),
        (px.1 as f32 / TILE_SIZE_F32.1).clamp(0., screen.tile_w as f32),
    );

    let world = (
        (camera.x + ui.0 - layout.game_panel.x as f32)
            .clamp(0., (MAP_SIZE.0 * ZONE_SIZE.0 - 1) as f32 + 0.99),
        (camera.y + ui.1 - layout.game_panel.y as f32)
            .clamp(0., (MAP_SIZE.1 * ZONE_SIZE.1 - 1) as f32 + 0.99),
    );

    cursor.px = px;
    cursor.ui = ui;
    cursor.uv = uv;
    cursor.world = world;
}
