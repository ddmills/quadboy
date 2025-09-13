use crate::{
    domain::components::{destructible::MaterialType, weapon_family::WeaponFamily},
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
    pub weapon_family: WeaponFamily,
}

impl RangedWeapon {
    pub fn new(
        damage: i32,
        range: usize,
        can_damage: Vec<MaterialType>,
        shoot_audio: AudioKey,
        weapon_family: WeaponFamily,
    ) -> Self {
        Self {
            damage,
            range,
            can_damage,
            shoot_audio,
            weapon_family,
        }
    }

    pub fn revolver() -> Self {
        Self::new(6, 12, vec![MaterialType::Flesh], AudioKey::RevolverShoot1, WeaponFamily::Pistol)
    }

    pub fn rifle() -> Self {
        Self::new(8, 16, vec![MaterialType::Flesh], AudioKey::RifleShoot1, WeaponFamily::Rifle)
    }

    pub fn shotgun() -> Self {
        Self::new(10, 8, vec![MaterialType::Flesh], AudioKey::ShotgunShoot1, WeaponFamily::Shotgun)
    }
}
