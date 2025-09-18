use crate::engine::{AudioKey, SerializableComponent};
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
    pub light_audio: Option<AudioKey>,
    pub extinguish_audio: Option<AudioKey>,
}

impl Lightable {
    pub fn new() -> Self {
        Self {
            action_label: "Extinguish".to_string(),
            light_audio: None,
            extinguish_audio: None,
        }
    }

    pub fn with_light_audio(mut self, audio: AudioKey) -> Self {
        self.light_audio = Some(audio);
        self
    }

    pub fn with_extinguish_audio(mut self, audio: AudioKey) -> Self {
        self.extinguish_audio = Some(audio);
        self
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
        Self::new(0.9, 0xFF7A2E, 4).with_flicker(0.8)
    }

    pub fn lantern() -> Self {
        Self::new(0.9, 0xFFC690, 6).with_flicker(0.4)
    }

    pub fn mushroom() -> Self {
        Self::new(0.5, 0x7FA7E2, 4)
    }
}
