use bevy_ecs::prelude::*;
use macroquad::prelude::*;

use crate::rendering::Text;

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

#[derive(Component)]
pub struct FpsDisplay;

pub fn render_fps(time: Res<Time>, mut q_fps: Query<&mut Text, With<FpsDisplay>>) {
    let binding = time.fps.to_string();
    let t = binding.as_str();

    for mut text in q_fps.iter_mut() {
        text.value = t.into();
    }
}
