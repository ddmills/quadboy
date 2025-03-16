use bevy_ecs::prelude::*;
use macroquad::input::KeyCode;

use crate::{cfg::TILE_SIZE, ecs::Time, engine::KeyInput};

use super::get_render_target_size;

#[derive(Resource, Default)]
pub struct GameCamera {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

pub fn update_camera(mut camera: ResMut<GameCamera>, keys: Res<KeyInput>, time: Res<Time>) {
    let speed = 25.;

    camera.w = 100.;
    camera.h = 100.;

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
    pub width: f32,
    pub height: f32,
    pub tile_w: usize,
    pub tile_h: usize,
}

pub fn update_screen_size(mut screen: ResMut<ScreenSize>)
{
    let size = get_render_target_size();

    screen.width = size.x as f32;
    screen.height = size.y as f32;
    screen.tile_h = (size.x as usize) / TILE_SIZE.0;
    screen.tile_w = (size.y as usize) / TILE_SIZE.1;
}