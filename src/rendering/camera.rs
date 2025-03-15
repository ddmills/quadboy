use bevy_ecs::prelude::*;
use macroquad::input::KeyCode;

use crate::{ecs::Time, engine::KeyInput};

#[derive(Resource, Default)]
pub struct GameCamera {
    pub x: f32,
    pub y: f32,
}

pub fn update_camera(mut camera: ResMut<GameCamera>, keys: Res<KeyInput>, time: Res<Time>) {
    let speed = 25.;

    if keys.is_down(KeyCode::A) {
        camera.x -= speed * time.dt;
    }

    if keys.is_down(KeyCode::D) {
        camera.x += speed * time.dt;
    }

    if keys.is_down(KeyCode::W) {
        camera.y -= speed * time.dt;
    }

    if keys.is_down(KeyCode::S) {
        camera.y += speed * time.dt;
    }
}
