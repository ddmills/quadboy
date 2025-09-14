use crate::{
    domain::components::{destructible::MaterialType, weapon_family::WeaponFamily},
    engine::{AudioKey, SerializableComponent},
};
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct RangedWeapon {
    pub damage_dice: String,
    pub range: usize,
    pub can_damage: Vec<MaterialType>,
    pub shoot_audio: AudioKey,
    pub weapon_family: WeaponFamily,
    pub clip_size: Option<usize>,
    pub current_ammo: Option<usize>,
    pub base_reload_cost: Option<i32>,
    pub reload_audio: Option<AudioKey>,
    pub no_ammo_audio: Option<AudioKey>,
}

impl RangedWeapon {
    pub fn new(
        damage_dice: String,
        range: usize,
        can_damage: Vec<MaterialType>,
        shoot_audio: AudioKey,
        weapon_family: WeaponFamily,
        clip_size: Option<usize>,
        base_reload_cost: Option<i32>,
        reload_audio: Option<AudioKey>,
        no_ammo_audio: Option<AudioKey>,
    ) -> Self {
        Self {
            damage_dice,
            range,
            can_damage,
            shoot_audio,
            weapon_family,
            clip_size,
            current_ammo: clip_size,
            base_reload_cost,
            reload_audio,
            no_ammo_audio,
        }
    }

    pub fn revolver() -> Self {
        Self::new(
            "1d8+2".to_string(),
            12,
            vec![MaterialType::Flesh],
            AudioKey::RevolverShoot1,
            WeaponFamily::Pistol,
            Some(6),
            Some(150),
            Some(AudioKey::RevolverReload),
            Some(AudioKey::RevolverEmpty),
        )
    }

    pub fn rifle() -> Self {
        Self::new(
            "1d10+3".to_string(),
            16,
            vec![MaterialType::Flesh],
            AudioKey::RifleShoot1,
            WeaponFamily::Rifle,
            Some(8),
            Some(200),
            Some(AudioKey::RifleReload),
            Some(AudioKey::RifleEmpty),
        )
    }

    pub fn shotgun() -> Self {
        Self::new(
            "2d6+1".to_string(),
            8,
            vec![MaterialType::Flesh],
            AudioKey::ShotgunShoot1,
            WeaponFamily::Shotgun,
            Some(2),
            Some(250),
            Some(AudioKey::ShotgunReload),
            Some(AudioKey::ShotgunEmpty),
        )
    }
}
