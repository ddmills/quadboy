use bevy_ecs::prelude::*;
use macroquad::prelude::*;

use crate::{
    cfg::{CAMERA_MODE, TILE_SIZE, TILE_SIZE_F32, ZONE_SIZE_F32},
    domain::Player,
    engine::Time,
    rendering::{
        Position, get_screen_size_texels, world_to_zone_local, zone_center_world,
        zone_local_to_world,
    },
};

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum CameraMode {
    Smooth,
    Snap,
}

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

pub fn update_camera(
    mut camera: ResMut<GameCamera>,
    q_player: Query<&Position, With<Player>>,
    time: Res<Time>,
) {
    let player = q_player.single().unwrap();
    let a = time.overstep_fraction();
    let speed = 0.075;

    let z_pos = zone_center_world(player.zone_idx());

    let player_pos = vec2(player.x + 0.5, player.y + 0.5);
    let center_pos = vec2(z_pos.0, z_pos.1);
    let camera_pos = camera.get_focus();
    let zone_pos = zone_local_to_world(player.zone_idx(), 0, 0);
    let player_local_pos = world_to_zone_local(player.x as usize, player.y as usize);

    let edge_pad = (1., 1.);

    let mut target = player_pos;

    let camera_w = camera.get_width_world();
    let camera_h = camera.get_height_world();

    let camera_radius = (camera_w / 2., camera_h / 2.);
    let camera_radius_buff = (camera_radius.0 - edge_pad.0, camera_radius.1 - edge_pad.1);

    // left edge
    let left_edge = player_local_pos.0 as f32 - camera_radius_buff.0;

    if left_edge < 0. {
        target.x = zone_pos.0 as f32 + camera_radius_buff.0;
    }

    // right edge
    let right_edge = player_local_pos.0 as f32 + camera_radius_buff.0;

    if right_edge > ZONE_SIZE_F32.0 {
        target.x = (zone_pos.0 as f32 + ZONE_SIZE_F32.0) - camera_radius_buff.0;
    }

    // top edge
    let top_edge = player_local_pos.1 as f32 - camera_radius_buff.1;

    if top_edge < 0. {
        target.y = zone_pos.1 as f32 + camera_radius_buff.1;
    }

    // bottom edge
    let bottom_edge = player_local_pos.1 as f32 + camera_radius_buff.1;

    if bottom_edge > ZONE_SIZE_F32.1 {
        target.y = (zone_pos.1 as f32 + ZONE_SIZE_F32.1) - camera_radius_buff.1;
    }

    if ZONE_SIZE_F32.0 < camera_w {
        target.x = center_pos.x;
    }

    if ZONE_SIZE_F32.1 < camera_h {
        target.y = center_pos.y;
    }

    if (camera.get_width_world() as u32).is_multiple_of(2) {
        target.x = target.x.floor();
    }

    if (camera.get_height_world() as u32).is_multiple_of(2) {
        target.y = target.y.floor();
    }

    if CAMERA_MODE == CameraMode::Snap || camera_pos.distance_squared(target) < 0.001 {
        camera.focus_on(target.x, target.y);
    } else {
        let target_fin = camera_pos.lerp(target, a * speed);
        camera.focus_on(target_fin.x, target_fin.y);
    }
}

#[derive(Resource, Default)]
pub struct ScreenSize {
    pub width: u32,
    pub height: u32,
    pub tile_w: usize,
    pub tile_h: usize,
}

pub fn update_screen_size(mut screen: ResMut<ScreenSize>) {
    let size = get_screen_size_texels();

    if size.x != screen.width || size.y != screen.height {
        screen.width = size.x;
        screen.height = size.y;
        screen.tile_w = (size.x as usize) / TILE_SIZE.0;
        screen.tile_h = (size.y as usize) / TILE_SIZE.1;
    }
}
