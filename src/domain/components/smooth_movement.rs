use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::engine::SerializableComponent;

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct SmoothMovement {
    pub start_position: (f32, f32),
    pub end_position: (f32, f32),
    pub duration_remaining: f32,
    pub total_duration: f32,
}

impl SmoothMovement {
    pub fn new(start_position: (f32, f32), end_position: (f32, f32)) -> Self {
        Self {
            start_position,
            end_position,
            duration_remaining: 0.1,
            total_duration: 0.1,
        }
    }
}