use bevy_ecs::prelude::*;
use macroquad::prelude::*;

use crate::rendering::{world_to_zone_idx};

#[derive(Component)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Position {
    pub fn new(x: usize, y: usize, z: usize) -> Self {
        Self {
            x: x as f32,
            y: y as f32,
            z: z as f32,
        }
    }

    pub fn new_f32(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    #[inline]
    pub fn zone_idx(&self) -> usize {
        world_to_zone_idx(self.x as usize, self.y as usize, self.z as usize)
    }

    #[inline]
    pub fn world(&self) -> (usize, usize, usize)
    {
        (self.x as usize, self.y as usize, self.z as usize)
    }
}
