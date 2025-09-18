use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{common::Palette, engine::SerializableComponent};

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct HitBlink {
    pub color: u32,
    pub duration_remaining: f32,
    pub blink_rate: Option<f32>, // cycles per second, None for one-time flash
    pub blink_on: bool,          // current blink state for continuous blinking
    pub time_since_last_toggle: f32, // time accumulator for blink timing
}

impl HitBlink {
    pub fn attacked() -> Self {
        Self {
            color: Palette::White.into(),
            duration_remaining: 0.05,
            blink_rate: None,
            blink_on: true,
            time_since_last_toggle: 0.0,
        }
    }

    pub fn blinking(color: u32, blink_rate: f32) -> Self {
        Self {
            color,
            duration_remaining: f32::INFINITY, // blink indefinitely
            blink_rate: Some(blink_rate),
            blink_on: true,
            time_since_last_toggle: 0.0,
        }
    }
}
