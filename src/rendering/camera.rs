use bevy_ecs::prelude::*;
use macroquad::input::KeyCode;

use crate::{cfg::TILE_SIZE, ecs::Time, engine::KeyInput};

use super::get_render_target_size;

#[derive(Resource, Default)]
pub struct GameCamera {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
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

#[derive(Resource, Default)]
pub struct ScreenSize {
    pub width: u32,
    pub height: u32,
    pub tile_w: usize,
    pub tile_h: usize,
}

pub fn update_screen_size(mut screen: ResMut<ScreenSize>)
{
    let size = get_render_target_size();

    if size.x != screen.width || size.y != screen.height {
        screen.width = size.x;
        screen.height = size.y;
        screen.tile_w = (size.x as usize) / TILE_SIZE.0;
        screen.tile_h = (size.y as usize) / TILE_SIZE.1;
    }
}
