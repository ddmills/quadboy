use bevy_ecs::prelude::*;
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};

use crate::rendering::{world_to_zone_idx, world_to_zone_local};

use crate::engine::SerializableComponent;

#[derive(Component, Serialize, Deserialize, Clone, Copy, SerializableComponent, Debug)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    #[serde(skip)]
    prev_zone_idx: usize,
}

impl Position {
    pub fn new_world(world_pos: (usize, usize, usize)) -> Self {
        Self {
            x: world_pos.0 as f32,
            y: world_pos.1 as f32,
            z: world_pos.2 as f32,
            prev_zone_idx: 9999999,
        }
    }

    pub fn new(x: usize, y: usize, z: usize) -> Self {
        Self {
            x: x as f32,
            y: y as f32,
            z: z as f32,
            prev_zone_idx: 9999999,
        }
    }

    pub fn new_f32(x: f32, y: f32, z: f32) -> Self {
        Self {
            x,
            y,
            z,
            prev_zone_idx: 9999999,
        }
    }

    #[inline]
    pub fn zone_idx(&self) -> usize {
        world_to_zone_idx(self.x as usize, self.y as usize, self.z as usize)
    }

    #[inline]
    pub fn world(&self) -> (usize, usize, usize) {
        (self.x as usize, self.y as usize, self.z as usize)
    }

    #[inline]
    pub fn zone_local(&self) -> (usize, usize) {
        world_to_zone_local(self.x as usize, self.y as usize)
    }

    #[inline]
    pub fn prev_zone_idx(&self) -> usize {
        self.prev_zone_idx
    }

    #[inline]
    pub fn set_prev_zone_idx(&mut self, idx: usize) {
        self.prev_zone_idx = idx;
    }
}
