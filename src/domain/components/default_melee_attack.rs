use crate::{domain::components::destructible::MaterialType, engine::SerializableComponent};
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct DefaultMeleeAttack {
    pub damage: i32,
    pub can_damage: Vec<MaterialType>,
    pub attack_name: String,
}

impl DefaultMeleeAttack {
    pub fn new(damage: i32, can_damage: Vec<MaterialType>, attack_name: &str) -> Self {
        Self {
            damage,
            can_damage,
            attack_name: attack_name.to_string(),
        }
    }

    pub fn fists() -> Self {
        Self::new(2, vec![MaterialType::Flesh], "Fists")
    }

    pub fn claw_swipe() -> Self {
        Self::new(
            4,
            vec![MaterialType::Flesh, MaterialType::Wood],
            "Claw Swipe",
        )
    }

    pub fn venomous_bite() -> Self {
        Self::new(3, vec![MaterialType::Flesh], "Venomous Bite")
    }

    pub fn bite() -> Self {
        Self::new(3, vec![MaterialType::Flesh], "Bite")
    }

    pub fn wing_buffet() -> Self {
        Self::new(1, vec![MaterialType::Flesh], "Wing Buffet")
    }

    pub fn electric_touch() -> Self {
        Self::new(2, vec![MaterialType::Flesh], "Electric Touch")
    }
}
