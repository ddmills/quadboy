use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::engine::{AudioKey, SerializableComponent};

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct ExplosiveProperties {
    pub fuse_duration: i32,
    pub radius: usize,
    pub base_damage: i32,
    pub falloff_rate: f32,
    pub destroys_terrain: bool,
    pub explosion_audio: Option<AudioKey>,
}

impl ExplosiveProperties {
    pub fn new(
        fuse_duration: i32,
        radius: usize,
        base_damage: i32,
        falloff_rate: f32,
        explosion_audio: Option<AudioKey>,
    ) -> Self {
        Self {
            fuse_duration,
            radius,
            base_damage,
            falloff_rate,
            destroys_terrain: true,
            explosion_audio,
        }
    }

    pub fn dynamite() -> Self {
        Self::new(600, 3, 50, 0.3, Some(AudioKey::Explosion1)) // 3 turns, 3 radius, 50 damage, 30% falloff per tile
    }

    pub fn calculate_damage_at_distance(&self, distance: f32) -> i32 {
        if distance <= 0.0 {
            return self.base_damage;
        }

        let damage_multiplier = (1.0 - (distance * self.falloff_rate)).max(0.0);
        (self.base_damage as f32 * damage_multiplier) as i32
    }
}
