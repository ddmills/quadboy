use crate::{
    domain::components::destructible::MaterialType,
    engine::{AudioKey, SerializableComponent},
};
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct RangedWeapon {
    pub damage: i32,
    pub range: usize,
    pub can_damage: Vec<MaterialType>,
    pub shoot_audio: AudioKey,
}

impl RangedWeapon {
    pub fn new(
        damage: i32,
        range: usize,
        can_damage: Vec<MaterialType>,
        shoot_audio: AudioKey,
    ) -> Self {
        Self {
            damage,
            range,
            can_damage,
            shoot_audio,
        }
    }

    pub fn revolver() -> Self {
        Self::new(6, 12, vec![MaterialType::Flesh], AudioKey::RevolverShoot1)
    }

    pub fn rifle() -> Self {
        Self::new(8, 16, vec![MaterialType::Flesh], AudioKey::RifleShoot1)
    }

    pub fn shotgun() -> Self {
        Self::new(10, 8, vec![MaterialType::Flesh], AudioKey::ShotgunShoot1)
    }
}
