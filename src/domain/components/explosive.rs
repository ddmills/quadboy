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
}
