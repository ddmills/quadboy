use crate::{
    domain::components::{
        destructible::MaterialType, hit_effect::HitEffect, weapon::Weapon,
        weapon_family::WeaponFamily, weapon_type::WeaponType,
    },
    engine::SerializableComponent,
};
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct DefaultMeleeAttack {
    pub weapon: Weapon,
}

impl DefaultMeleeAttack {
    pub fn new(
        damage: i32,
        can_damage: Vec<MaterialType>,
        attack_name: &str,
        weapon_family: WeaponFamily,
    ) -> Self {
        let weapon = Weapon {
            damage_dice: damage.to_string(),
            can_damage: can_damage.clone(),
            weapon_family,
            weapon_type: WeaponType::Melee,
            hit_effects: Vec::new(),
            particle_effect_id: None,
            range: None,
            shoot_audio: None,
            clip_size: None,
            current_ammo: None,
            base_reload_cost: None,
            reload_audio: None,
            reload_complete_audio: None,
            no_ammo_audio: None,
        };

        Self { weapon }
    }

    pub fn with_hit_effects(
        damage: i32,
        can_damage: Vec<MaterialType>,
        attack_name: &str,
        weapon_family: WeaponFamily,
        hit_effects: Vec<HitEffect>,
    ) -> Self {
        let weapon = Weapon {
            damage_dice: damage.to_string(),
            can_damage: can_damage.clone(),
            weapon_family,
            weapon_type: WeaponType::Melee,
            hit_effects: hit_effects.clone(),
            particle_effect_id: None,
            range: None,
            shoot_audio: None,
            clip_size: None,
            current_ammo: None,
            base_reload_cost: None,
            reload_audio: None,
            reload_complete_audio: None,
            no_ammo_audio: None,
        };

        Self { weapon }
    }

    pub fn fists() -> Self {
        Self::new(2, vec![MaterialType::Flesh], "Fists", WeaponFamily::Unarmed)
    }

    pub fn fists_with_knockback() -> Self {
        Self::with_hit_effects(
            2,
            vec![MaterialType::Flesh],
            "Fists",
            WeaponFamily::Unarmed,
            vec![HitEffect::Knockback {
                strength: 0.5,
                chance: 1.0,
            }], // Always knockback at half strength
        )
    }

    pub fn claw_swipe() -> Self {
        Self::new(
            4,
            vec![MaterialType::Flesh, MaterialType::Wood],
            "Claw Swipe",
            WeaponFamily::Unarmed,
        )
    }

    pub fn venomous_bite() -> Self {
        Self::with_hit_effects(
            3,
            vec![MaterialType::Flesh],
            "Venomous Bite",
            WeaponFamily::Unarmed,
            vec![HitEffect::Poison {
                damage_per_tick: 2,
                duration_ticks: 1000,
                chance: 1.0, // Always apply poison
            }],
        )
    }
    pub fn fire_fists() -> Self {
        Self::with_hit_effects(
            3,
            vec![MaterialType::Flesh],
            "Fire Fists",
            WeaponFamily::Unarmed,
            vec![HitEffect::Bleeding {
                damage_per_tick: 2,
                duration_ticks: 1000,
                chance: 1.0, // Always apply poison
                can_stack: false,
            }],
        )
    }

    pub fn bite() -> Self {
        Self::new(3, vec![MaterialType::Flesh], "Bite", WeaponFamily::Unarmed)
    }

    pub fn wing_buffet() -> Self {
        Self::new(
            1,
            vec![MaterialType::Flesh],
            "Wing Buffet",
            WeaponFamily::Unarmed,
        )
    }

    pub fn electric_touch() -> Self {
        Self::new(
            2,
            vec![MaterialType::Flesh],
            "Electric Touch",
            WeaponFamily::Unarmed,
        )
    }
}
