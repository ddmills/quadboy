use bevy_ecs::prelude::*;
use macroquad::prelude::*;

use crate::rendering::get_render_offset;

#[derive(Resource, Default)]
pub struct Time {
    pub dt: f32,
    pub fps: i32,
}

pub fn update_time(mut time: ResMut<Time>) {
    time.dt = get_frame_time();
    time.fps = get_fps();
}

pub fn render_fps(time: Res<Time>) {
    let offset = get_render_offset();

    draw_text(time.fps.to_string().as_str(), 16.0 + offset.x, 32.0 + offset.y, 16.0, GOLD);
}
