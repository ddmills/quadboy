use crate::{
    domain::components::{
        destructible::MaterialType, hit_effect::HitEffect, weapon::Weapon,
        weapon_family::WeaponFamily, weapon_type::WeaponType,
    },
    engine::{AudioKey, SerializableComponent},
    rendering::ParticleEffectId,
};
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct DefaultRangedAttack {
    pub weapon: Weapon,
}

impl DefaultRangedAttack {
    pub fn new(
        damage_dice: String,
        can_damage: Vec<MaterialType>,
        attack_name: &str,
        weapon_family: WeaponFamily,
        range: usize,
        shoot_audio: Option<AudioKey>,
    ) -> Self {
        let weapon = Weapon {
            damage_dice: damage_dice.clone(),
            can_damage: can_damage.clone(),
            weapon_family,
            weapon_type: WeaponType::Ranged,
            hit_effects: Vec::new(),
            particle_effect_id: None,
            melee_audio: None,
            range: Some(range),
            shoot_audio,
            clip_size: None,
            current_ammo: None, // None = infinite ammo for default weapons
            base_reload_cost: None,
            reload_audio: None,
            reload_complete_audio: None,
            no_ammo_audio: None,
        };

        Self { weapon }
    }

    pub fn with_ammo(
        damage_dice: String,
        can_damage: Vec<MaterialType>,
        attack_name: &str,
        weapon_family: WeaponFamily,
        range: usize,
        shoot_audio: Option<AudioKey>,
        ammo: usize,
        no_ammo_audio: Option<AudioKey>,
    ) -> Self {
        // NOTE: Despite the name "with_ammo", default weapons still get infinite ammo
        // This method is kept for API compatibility
        let weapon = Weapon {
            damage_dice: damage_dice.clone(),
            can_damage: can_damage.clone(),
            weapon_family,
            weapon_type: WeaponType::Ranged,
            hit_effects: Vec::new(),
            particle_effect_id: None,
            melee_audio: None,
            range: Some(range),
            shoot_audio,
            clip_size: None,
            current_ammo: None, // None = infinite ammo for default weapons
            base_reload_cost: None,
            reload_audio: None,
            reload_complete_audio: None,
            no_ammo_audio,
        };

        Self { weapon }
    }

    pub fn revolver() -> Self {
        let mut revolver = Self::with_ammo(
            "1d8+2".to_string(),
            vec![MaterialType::Flesh],
            "Revolver Shot",
            WeaponFamily::Pistol,
            12,
            Some(AudioKey::RevolverShoot1),
            6,
            Some(AudioKey::RevolverEmpty),
        );
        revolver.weapon.particle_effect_id = Some(ParticleEffectId::default_pistol());
        revolver
    }

    pub fn rifle() -> Self {
        let mut rifle = Self::with_ammo(
            "1d10+3".to_string(),
            vec![MaterialType::Flesh],
            "Rifle Shot",
            WeaponFamily::Rifle,
            16,
            Some(AudioKey::RifleShoot2),
            4,
            Some(AudioKey::RifleEmpty),
        );
        rifle.weapon.particle_effect_id = Some(ParticleEffectId::default_rifle());
        rifle
    }

    pub fn has_ammo(&self) -> bool {
        match self.weapon.current_ammo {
            Some(ammo) => ammo > 0,
            None => true, // Unlimited ammo for default weapons
        }
    }

    pub fn consume_ammo(&mut self) {
        if let Some(current) = self.weapon.current_ammo {
            self.weapon.current_ammo = Some(current.saturating_sub(1));
        }
    }
}
