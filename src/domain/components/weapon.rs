use crate::{
    domain::components::{
        destructible::MaterialType, hit_effect::HitEffect, weapon_family::WeaponFamily,
        weapon_type::WeaponType,
    },
    engine::{AudioKey, SerializableComponent},
};
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct Weapon {
    pub damage_dice: String,
    pub can_damage: Vec<MaterialType>,
    pub weapon_family: WeaponFamily,
    pub weapon_type: WeaponType,
    pub hit_effects: Vec<HitEffect>,

    // Optional ranged-specific fields
    pub range: Option<usize>,
    pub shoot_audio: Option<AudioKey>,
    pub clip_size: Option<usize>,
    pub current_ammo: Option<usize>,
    pub base_reload_cost: Option<i32>,
    pub reload_audio: Option<AudioKey>,
    pub no_ammo_audio: Option<AudioKey>,
}

impl Weapon {
    pub fn new_melee(
        damage_dice: String,
        can_damage: Vec<MaterialType>,
        weapon_family: WeaponFamily,
    ) -> Self {
        Self {
            damage_dice,
            can_damage,
            weapon_family,
            weapon_type: WeaponType::Melee,
            hit_effects: Vec::new(),
            range: None,
            shoot_audio: None,
            clip_size: None,
            current_ammo: None,
            base_reload_cost: None,
            reload_audio: None,
            no_ammo_audio: None,
        }
    }

    pub fn new_ranged(
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
            can_damage,
            weapon_family,
            weapon_type: WeaponType::Ranged,
            hit_effects: Vec::new(),
            range: Some(range),
            shoot_audio: Some(shoot_audio),
            clip_size,
            current_ammo: clip_size,
            base_reload_cost,
            reload_audio,
            no_ammo_audio,
        }
    }

    pub fn is_melee(&self) -> bool {
        self.weapon_type == WeaponType::Melee
    }

    pub fn is_ranged(&self) -> bool {
        self.weapon_type == WeaponType::Ranged
    }

    // Legacy constructors for backwards compatibility during migration
    pub fn pickaxe() -> Self {
        Self::new_melee(
            "1d4".to_string(),
            vec![MaterialType::Stone, MaterialType::Flesh],
            WeaponFamily::Cudgel,
        )
    }

    pub fn hatchet() -> Self {
        Self::new_melee(
            "1d4".to_string(),
            vec![MaterialType::Wood, MaterialType::Flesh],
            WeaponFamily::Cudgel,
        )
    }

    pub fn sword() -> Self {
        Self::new_melee(
            "1d6+1".to_string(),
            vec![MaterialType::Flesh],
            WeaponFamily::Blade,
        )
    }

    pub fn revolver() -> Self {
        Self::new_ranged(
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
        Self::new_ranged(
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
        let mut shotgun = Self::new_ranged(
            "2d6+1".to_string(),
            8,
            vec![MaterialType::Flesh],
            AudioKey::ShotgunShoot1,
            WeaponFamily::Shotgun,
            Some(2),
            Some(250),
            Some(AudioKey::ShotgunReload),
            Some(AudioKey::ShotgunEmpty),
        );
        shotgun.hit_effects = vec![HitEffect::Knockback(1.0)];
        shotgun
    }
}
