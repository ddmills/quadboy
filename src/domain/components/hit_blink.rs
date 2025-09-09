use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{common::Palette, engine::SerializableComponent};

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct HitBlink {
    pub color: u32,
    pub duration_remaining: f32,
}

impl HitBlink {
    pub fn attacked() -> Self {
        Self {
            color: Palette::White.into(),
            duration_remaining: 0.05,
        }
    }
}