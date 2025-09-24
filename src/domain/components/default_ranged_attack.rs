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
    // Keep old fields for backward compatibility during transition
    #[deprecated = "Use weapon.damage_dice instead"]
    pub damage_dice: String,
    #[deprecated = "Use weapon.can_damage instead"]
    pub can_damage: Vec<MaterialType>,
    #[deprecated = "Use weapon name instead"]
    pub attack_name: String,
    #[deprecated = "Use weapon.weapon_family instead"]
    pub weapon_family: WeaponFamily,
    #[deprecated = "Use weapon.hit_effects instead"]
    pub hit_effects: Vec<HitEffect>,
    #[deprecated = "Use weapon.range instead"]
    pub range: usize,
    #[deprecated = "Use weapon.shoot_audio instead"]
    pub shoot_audio: Option<AudioKey>,
    #[deprecated = "Use weapon.clip_size instead"]
    pub ammo: Option<usize>,
    #[deprecated = "Use weapon.current_ammo instead"]
    pub current_ammo: Option<usize>,
    #[deprecated = "Use weapon.no_ammo_audio instead"]
    pub no_ammo_audio: Option<AudioKey>,
    #[deprecated = "Use weapon.particle_effect_id instead"]
    pub particle_effect_id: Option<ParticleEffectId>,
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
            range: Some(range),
            shoot_audio,
            clip_size: None,
            current_ammo: None, // None = infinite ammo for default weapons
            base_reload_cost: None,
            reload_audio: None,
            reload_complete_audio: None,
            no_ammo_audio: None,
        };

        Self {
            weapon,
            damage_dice,
            can_damage,
            attack_name: attack_name.to_string(),
            weapon_family,
            hit_effects: Vec::new(),
            range,
            shoot_audio,
            ammo: None,
            current_ammo: None,
            no_ammo_audio: None,
            particle_effect_id: None,
        }
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
            range: Some(range),
            shoot_audio,
            clip_size: None,
            current_ammo: None, // None = infinite ammo for default weapons
            base_reload_cost: None,
            reload_audio: None,
            reload_complete_audio: None,
            no_ammo_audio,
        };

        Self {
            weapon,
            damage_dice,
            can_damage,
            attack_name: attack_name.to_string(),
            weapon_family,
            hit_effects: Vec::new(),
            range,
            shoot_audio,
            ammo: Some(ammo),
            current_ammo: Some(ammo),
            no_ammo_audio,
            particle_effect_id: None,
        }
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
        // Set particle effect on both old field and new weapon
        revolver.particle_effect_id = Some(ParticleEffectId::default_pistol());
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
        // Set particle effect on both old field and new weapon
        rifle.particle_effect_id = Some(ParticleEffectId::default_rifle());
        rifle.weapon.particle_effect_id = Some(ParticleEffectId::default_rifle());
        rifle
    }

    pub fn has_ammo(&self) -> bool {
        // Check the weapon first, fallback to old field for compatibility
        match self.weapon.current_ammo.or(self.current_ammo) {
            Some(ammo) => ammo > 0,
            None => true, // Unlimited ammo for default weapons
        }
    }

    pub fn consume_ammo(&mut self) {
        // Update both weapon and old field for compatibility
        if let Some(current) = self.weapon.current_ammo {
            self.weapon.current_ammo = Some(current.saturating_sub(1));
        }
        if let Some(current) = self.current_ammo {
            self.current_ammo = Some(current.saturating_sub(1));
        }
    }
}
