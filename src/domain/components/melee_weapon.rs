use crate::{domain::components::{destructible::MaterialType, weapon_family::WeaponFamily}, engine::SerializableComponent};
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct MeleeWeapon {
    pub damage: i32,
    pub can_damage: Vec<MaterialType>,
    pub weapon_family: WeaponFamily,
}

impl MeleeWeapon {
    pub fn new(damage: i32, can_damage: Vec<MaterialType>, weapon_family: WeaponFamily) -> Self {
        Self { damage, can_damage, weapon_family }
    }

    pub fn pickaxe() -> Self {
        Self::new(2, vec![MaterialType::Stone], WeaponFamily::Cudgel)
    }

    pub fn hatchet() -> Self {
        Self::new(2, vec![MaterialType::Wood, MaterialType::Flesh], WeaponFamily::Cudgel)
    }

    pub fn sword() -> Self {
        Self::new(4, vec![MaterialType::Flesh], WeaponFamily::Blade)
    }
}
