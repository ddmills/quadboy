use std::collections::HashMap;

use crate::engine::SerializableComponent;
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use super::Attributes;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AttributeGroup {
    Strength,
    Dexterity,
    Constitution,
    Intelligence,
    Special,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StatType {
    Fortitude,    // affects max HP
    Speed,        // affects movement energy cost
    Armor,        // no base attribute
    ArmorRegen,   // affects armor regeneration rate
    Rifle,        // rifle weapon proficiency
    Shotgun,      // shotgun weapon proficiency
    Pistol,       // pistol weapon proficiency
    Blade,        // blade weapon proficiency
    Cudgel,       // cudgel weapon proficiency
    Unarmed,      // unarmed combat proficiency
    Dodge,        // dodge/evasion ability
    Knockback,    // affects knockback distance
    ReloadSpeed,  // affects reload energy cost
    PoisonDamage, // bonus damage to poison effects
    BleedDamage,  // bonus damage to bleed effects
    BurnDamage,   // bonus damage to burn effects
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
            StatType::Dodge => attributes.dexterity as i32,
            StatType::Knockback => attributes.strength as i32,
            StatType::ReloadSpeed => attributes.dexterity as i32,
            StatType::PoisonDamage => 3,
            StatType::BleedDamage => 3,
            StatType::BurnDamage => 3,
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
            StatType::Dodge,
            StatType::Knockback,
            StatType::ReloadSpeed,
            StatType::PoisonDamage,
            StatType::BleedDamage,
            StatType::BurnDamage,
        ]
    }

    pub fn verb(&self) -> &'static str {
        match self {
            StatType::Fortitude => "Fortitude",
            StatType::Speed => "Swiftness",
            StatType::Armor => "Protection",
            StatType::ArmorRegen => "Restoration",
            StatType::Rifle => "Marksmanship",
            StatType::Shotgun => "Devastation",
            StatType::Pistol => "Precision",
            StatType::Blade => "Sharpness",
            StatType::Cudgel => "Smashing",
            StatType::Unarmed => "Striking",
            StatType::Dodge => "Evasion",
            StatType::Knockback => "Impact",
            StatType::ReloadSpeed => "Reloading",
            StatType::PoisonDamage => "Toxicity",
            StatType::BleedDamage => "Laceration",
            StatType::BurnDamage => "Combustion",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            StatType::Fortitude => "Maximum health points capacity",
            StatType::Speed => "Reduces movement energy cost",
            StatType::Armor => "Damage absorbed before health loss",
            StatType::ArmorRegen => "Rate of armor regeneration per tick",
            StatType::Rifle => "Rifle weapon proficiency and accuracy",
            StatType::Shotgun => "Shotgun weapon proficiency and damage",
            StatType::Pistol => "Pistol weapon proficiency and accuracy",
            StatType::Blade => "Blade weapon proficiency and damage",
            StatType::Cudgel => "Cudgel weapon proficiency and damage",
            StatType::Unarmed => "Unarmed combat proficiency and damage",
            StatType::Dodge => "Chance to evade incoming attacks",
            StatType::Knockback => "Distance enemies are pushed on hit",
            StatType::ReloadSpeed => "Reduces weapon reload energy cost",
            StatType::PoisonDamage => "Bonus damage to poison effects per tick",
            StatType::BleedDamage => "Bonus damage to bleeding effects per tick",
            StatType::BurnDamage => "Bonus damage to burning effects per tick",
        }
    }

    pub fn get_attribute_group(&self) -> AttributeGroup {
        match self {
            StatType::Shotgun
            | StatType::Pistol
            | StatType::Cudgel
            | StatType::Unarmed
            | StatType::Knockback => AttributeGroup::Strength,
            StatType::Speed
            | StatType::Rifle
            | StatType::Blade
            | StatType::Dodge
            | StatType::ReloadSpeed => AttributeGroup::Dexterity,
            StatType::Fortitude => AttributeGroup::Constitution,
            StatType::ArmorRegen => AttributeGroup::Intelligence,
            StatType::Armor
            | StatType::PoisonDamage
            | StatType::BleedDamage
            | StatType::BurnDamage => AttributeGroup::Special,
        }
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
        self.modifiers.entry(stat_type).or_default().push(modifier);
    }

    pub fn remove_equipment_modifiers(&mut self, item_id: u64) {
        for modifiers in self.modifiers.values_mut() {
            modifiers.retain(
                |m| !matches!(m.source, ModifierSource::Equipment { item_id: id } if id == item_id),
            );
        }
    }

    pub fn remove_condition_modifiers(&mut self, condition_id: &str) {
        for modifiers in self.modifiers.values_mut() {
            modifiers.retain(
                |m| !matches!(&m.source, ModifierSource::Condition { condition_id: id } if id == condition_id),
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

    pub fn condition(value: i32, condition_id: String) -> Self {
        Self {
            value,
            source: ModifierSource::Condition { condition_id },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModifierSource {
    Equipment { item_id: u64 },
    Intrinsic { name: String },
    Condition { condition_id: String },
}
