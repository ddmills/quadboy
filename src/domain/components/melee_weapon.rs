use crate::{
    domain::components::{destructible::MaterialType, weapon_family::WeaponFamily},
    engine::SerializableComponent,
};
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct MeleeWeapon {
    pub damage_dice: String,
    pub can_damage: Vec<MaterialType>,
    pub weapon_family: WeaponFamily,
}

impl MeleeWeapon {
    pub fn new(
        damage_dice: String,
        can_damage: Vec<MaterialType>,
        weapon_family: WeaponFamily,
    ) -> Self {
        Self {
            damage_dice,
            can_damage,
            weapon_family,
        }
    }

    pub fn pickaxe() -> Self {
        Self::new(
            "1d4".to_string(),
            vec![MaterialType::Stone],
            WeaponFamily::Cudgel,
        )
    }

    pub fn hatchet() -> Self {
        Self::new(
            "1d4".to_string(),
            vec![MaterialType::Wood, MaterialType::Flesh],
            WeaponFamily::Cudgel,
        )
    }

    pub fn sword() -> Self {
        Self::new(
            "1d6+1".to_string(),
            vec![MaterialType::Flesh],
            WeaponFamily::Blade,
        )
    }
}
