use std::collections::HashMap;

use crate::engine::SerializableComponent;
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use super::Attributes;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StatType {
    Fortitude,  // Constitution + modifiers (affects max HP)
    Speed,      // Dexterity + modifiers (affects movement energy cost)
    Armor,      // Only affected by modifiers (no base attribute)
    ArmorRegen, // Intelligence + modifiers (affects armor regeneration rate)
    Rifle,      // Dexterity + modifiers (rifle weapon proficiency)
    Shotgun,    // Strength + modifiers (shotgun weapon proficiency)
    Pistol,     // Strength + modifiers (pistol weapon proficiency)
    Blade,      // Dexterity + modifiers (blade weapon proficiency)
    Cudgel,     // Strength + modifiers (cudgel weapon proficiency)
    Unarmed,    // Strength + modifiers (unarmed combat proficiency)
}

impl StatType {
    pub fn get_base_value(&self, attributes: &Attributes) -> i32 {
        match self {
            StatType::Fortitude => attributes.constitution as i32,
            StatType::Speed => attributes.dexterity as i32,
            StatType::Armor => 0, // No base attribute, only modifiers
            StatType::ArmorRegen => attributes.intelligence as i32,
            StatType::Rifle => attributes.dexterity as i32,
            StatType::Shotgun => attributes.strength as i32,
            StatType::Pistol => attributes.strength as i32,
            StatType::Blade => attributes.dexterity as i32,
            StatType::Cudgel => attributes.strength as i32,
            StatType::Unarmed => attributes.strength as i32,
        }
    }

    pub fn all() -> &'static [StatType] {
        &[
            StatType::Fortitude,
            StatType::Speed,
            StatType::Armor,
            StatType::ArmorRegen,
            StatType::Rifle,
            StatType::Shotgun,
            StatType::Pistol,
            StatType::Blade,
            StatType::Cudgel,
            StatType::Unarmed,
        ]
    }
}

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct Stats {
    pub values: HashMap<StatType, i32>, // Cached calculated values
}

impl Stats {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn get_stat(&self, stat_type: StatType) -> i32 {
        *self.values.get(&stat_type).unwrap_or(&0)
    }
}

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct StatModifiers {
    pub modifiers: HashMap<StatType, Vec<StatModifier>>,
}

impl StatModifiers {
    pub fn new() -> Self {
        Self {
            modifiers: HashMap::new(),
        }
    }

    pub fn add_modifier(&mut self, stat_type: StatType, modifier: StatModifier) {
        self.modifiers
            .entry(stat_type)
            .or_insert_with(Vec::new)
            .push(modifier);
    }

    pub fn remove_equipment_modifiers(&mut self, item_id: u64) {
        for modifiers in self.modifiers.values_mut() {
            modifiers.retain(
                |m| !matches!(m.source, ModifierSource::Equipment { item_id: id } if id == item_id),
            );
        }
    }

    pub fn get_total_for_stat(&self, stat_type: StatType) -> i32 {
        self.modifiers
            .get(&stat_type)
            .map(|mods| mods.iter().map(|m| m.value).sum())
            .unwrap_or(0)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StatModifier {
    pub value: i32,
    pub source: ModifierSource,
}

impl StatModifier {
    pub fn equipment(value: i32, item_id: u64) -> Self {
        Self {
            value,
            source: ModifierSource::Equipment { item_id },
        }
    }

    pub fn intrinsic(value: i32, name: String) -> Self {
        Self {
            value,
            source: ModifierSource::Intrinsic { name },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModifierSource {
    Equipment { item_id: u64 },
    Intrinsic { name: String },
}
