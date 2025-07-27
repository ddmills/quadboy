use bevy_ecs::prelude::*;
use macroquad::prelude::*;

use crate::{cfg::{TILE_SIZE, TILE_SIZE_F32}, domain::Player, ecs::Time, rendering::{zone_center_world, Position}};

use super::get_render_target_size;

#[derive(Resource, Default)]
pub struct GameCamera {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl GameCamera {
    pub fn focus_on(&mut self, x: f32, y: f32) {
        self.x = x - (self.get_width_world() / 2.);
        self.y = y - (self.get_height_world() / 2.);
    }

    pub fn get_focus(&mut self) -> Vec2 {
        vec2(
            self.x + self.get_width_world() / 2.,
            self.y + self.get_height_world() / 2.,
        )
    }

    pub fn get_width_world(&self) -> f32 {
        self.width / TILE_SIZE_F32.0
    }

    pub fn get_height_world(&self) -> f32 {
        self.height / TILE_SIZE_F32.1
    }
}

pub fn update_camera(mut camera: ResMut<GameCamera>, q_player: Query<&Position, With<Player>>, time: Res<Time>) {
    let player = q_player.single().unwrap();
    let a = time.overstep_fraction();
    let speed = 0.08;

    let z_pos = zone_center_world(player.zone_idx());

    let player_pos = vec2(z_pos.0, z_pos.1);
    let camera_pos = camera.get_focus();

    let target = camera_pos.lerp(player_pos, a * speed);

    camera.focus_on(target.x, target.y);
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
