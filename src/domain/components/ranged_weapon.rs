use crate::{domain::components::destructible::MaterialType, engine::SerializableComponent};
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct RangedWeapon {
    pub damage: i32,
    pub range: i32,
    pub can_damage: Vec<MaterialType>,
}

impl RangedWeapon {
    pub fn new(damage: i32, range: i32, can_damage: Vec<MaterialType>) -> Self {
        Self {
            damage,
            range,
            can_damage,
        }
    }

    pub fn revolver() -> Self {
        Self::new(6, 8, vec![MaterialType::Flesh])
    }

    pub fn rifle() -> Self {
        Self::new(8, 12, vec![MaterialType::Flesh])
    }

    pub fn shotgun() -> Self {
        Self::new(10, 6, vec![MaterialType::Flesh])
    }
}