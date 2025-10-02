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
        attack_verb: &str,
        attack_noun: &str,
    ) -> Self {
        let mut weapon = Weapon::new_ranged(
            damage_dice,
            range,
            can_damage,
            shoot_audio.unwrap_or(AudioKey::RevolverShoot1),
            weapon_family,
            attack_verb.to_string(),
            attack_noun.to_string(),
            None, // No clip for default weapons
            None, // No reload cost
            None, // No reload audio
            None, // No reload complete audio
            None, // No empty audio
        );
        weapon.current_ammo = None; // Override to infinite ammo
        weapon.shoot_audio = shoot_audio; // Restore original audio setting

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
        attack_verb: &str,
        attack_noun: &str,
    ) -> Self {
        // NOTE: Despite the name "with_ammo", default weapons still get infinite ammo
        // This method is kept for API compatibility
        let mut weapon = Weapon::new_ranged(
            damage_dice,
            range,
            can_damage,
            shoot_audio.unwrap_or(AudioKey::RevolverShoot1),
            weapon_family,
            attack_verb.to_string(),
            attack_noun.to_string(),
            None, // No clip for default weapons
            None, // No reload cost
            None, // No reload audio
            None, // No reload complete audio
            no_ammo_audio,
        );
        weapon.current_ammo = None; // Override to infinite ammo
        weapon.shoot_audio = shoot_audio; // Restore original audio setting

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
            "shoots",
            "shot",
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
            "shoots",
            "shot",
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
