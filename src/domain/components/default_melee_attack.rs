use crate::{
    domain::components::{
        destructible::MaterialType, hit_effect::HitEffect, weapon_family::WeaponFamily,
    },
    engine::SerializableComponent,
};
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct DefaultMeleeAttack {
    pub damage: i32,
    pub can_damage: Vec<MaterialType>,
    pub attack_name: String,
    pub weapon_family: WeaponFamily,
    pub hit_effects: Vec<HitEffect>,
}

impl DefaultMeleeAttack {
    pub fn new(
        damage: i32,
        can_damage: Vec<MaterialType>,
        attack_name: &str,
        weapon_family: WeaponFamily,
    ) -> Self {
        Self {
            damage,
            can_damage,
            attack_name: attack_name.to_string(),
            weapon_family,
            hit_effects: Vec::new(),
        }
    }

    pub fn with_hit_effects(
        damage: i32,
        can_damage: Vec<MaterialType>,
        attack_name: &str,
        weapon_family: WeaponFamily,
        hit_effects: Vec<HitEffect>,
    ) -> Self {
        Self {
            damage,
            can_damage,
            attack_name: attack_name.to_string(),
            weapon_family,
            hit_effects,
        }
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
            vec![HitEffect::Knockback(0.5)], // Strength / 2
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
        Self::new(
            3,
            vec![MaterialType::Flesh],
            "Venomous Bite",
            WeaponFamily::Unarmed,
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
