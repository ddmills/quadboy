use crate::{
    domain::components::{destructible::MaterialType, weapon_family::WeaponFamily},
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
        }
    }

    pub fn fists() -> Self {
        Self::new(2, vec![MaterialType::Flesh], "Fists", WeaponFamily::Unarmed)
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
