use crate::engine::SerializableComponent;
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct LightSource {
    pub intensity: f32,
    pub color: u32,
    pub range: i32,
    pub is_enabled: bool,
    pub flicker: f32,
}

#[derive(Component)]
pub struct LightBlocker;

#[derive(Component)]
pub struct IgnoreLighting;

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct Lightable {
    pub action_label: String,
}

impl Lightable {
    pub fn new() -> Self {
        Self {
            action_label: "Extinguish".to_string(),
        }
    }

    pub fn update_label(&mut self, is_lit: bool) {
        self.action_label = if is_lit {
            "Extinguish".to_string()
        } else {
            "Light".to_string()
        };
    }
}

impl LightSource {
    pub fn new(intensity: f32, color: u32, range: i32) -> Self {
        Self {
            intensity,
            color,
            range,
            is_enabled: true,
            flicker: 0.0,
        }
    }

    pub fn with_flicker(mut self, flicker: f32) -> Self {
        self.flicker = flicker;
        self
    }

    pub fn campfire() -> Self {
        Self::new(0.9, 0xFFBB55, 6).with_flicker(0.8)
    }

    pub fn lantern() -> Self {
        Self::new(0.9, 0xFFCB90, 10).with_flicker(0.4)
    }

    pub fn mushroom() -> Self {
        Self::new(0.4, 0x93C7E5, 4)
    }
}
