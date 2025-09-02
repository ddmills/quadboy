use crate::{domain::components::destructible::MaterialType, engine::SerializableComponent};
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct MeleeWeapon {
    pub damage: i32,
    pub can_damage: Vec<MaterialType>,
}

impl MeleeWeapon {
    pub fn new(damage: i32, can_damage: Vec<MaterialType>) -> Self {
        Self { damage, can_damage }
    }

    pub fn pickaxe() -> Self {
        Self::new(2, vec![MaterialType::Stone])
    }

    pub fn hatchet() -> Self {
        Self::new(2, vec![MaterialType::Wood])
    }

    pub fn sword() -> Self {
        Self::new(3, vec![MaterialType::Flesh])
    }

    pub fn can_damage_material(&self, material: MaterialType) -> bool {
        self.can_damage.contains(&material)
    }
}
