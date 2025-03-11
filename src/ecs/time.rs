use bevy_ecs::prelude::*;
use macroquad::prelude::*;

use crate::{cfg::BODY_FONT_SIZE_F32, common::{cp437_idx, MacroquadColorable, Palette}, rendering::{get_render_offset, Renderable, TilesetId, TRANSPARENT}};

#[derive(Resource, Default)]
pub struct Time {
    pub dt: f32,
    pub fps: i32,
    pub start: f64,
}

pub fn update_time(mut time: ResMut<Time>) {
    time.dt = get_frame_time();
    time.fps = get_fps();
    time.start = get_time();
}

pub fn render_fps(time: Res<Time>) {
    // let offset = get_render_offset();

    // draw_text(time.fps.to_string().as_str(), 16.0 + offset.x, 24.0 + offset.y, 16.0, GOLD);

    let binding = time.fps.to_string();
    let t = binding.as_str();

    for (i, c) in t.chars().enumerate() {
        // renderer.draw(Renderable {
        //     idx: cp437_idx(c).unwrap_or(0),
        //     fg1: Palette::LightGreen.to_macroquad_color(),
        //     fg2: TRANSPARENT,
        //     bg: TRANSPARENT,
        //     outline: TRANSPARENT,
        //     tileset_id: TilesetId::BodyFont,
        //     x: i as f32 * BODY_FONT_SIZE_F32.0,
        //     y: 0.,
        //     w: BODY_FONT_SIZE_F32.0,
        //     h: BODY_FONT_SIZE_F32.1,
        // });
    }
}
