use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::engine::SerializableComponent;

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct BumpAttack {
    pub direction: (f32, f32),
    pub duration_remaining: f32,
    pub total_duration: f32,
}

impl BumpAttack {
    pub fn attacked(direction: (f32, f32)) -> Self {
        Self {
            direction,
            duration_remaining: 0.15,
            total_duration: 0.15,
        }
    }
}
