use crate::{
    common::Rand,
    domain::{HitEffect, MaterialType, WeaponFamily, WeaponType},
    engine::SerializableComponent,
};
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WeaponApplicationType {
    MeleeOnly,
    RangedOnly,
    Both,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WeaponModifierType {
    Prefix,
    Suffix,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaponModifier {
    pub name: String,
    pub modifier_type: WeaponModifierType,
    pub applicable_to: WeaponApplicationType,
    pub damage_bonus: i32,
    pub energy_cost_modifier: i32,
    pub range_modifier: i32,
    pub hit_effects: Vec<HitEffect>,
    pub special_damage: Vec<(MaterialType, i32)>, // Bonus vs specific materials
    pub description: String,
}

impl WeaponModifier {
    pub fn new(
        name: &str,
        modifier_type: WeaponModifierType,
        applicable_to: WeaponApplicationType,
        description: &str,
    ) -> Self {
        Self {
            name: name.to_string(),
            modifier_type,
            applicable_to,
            damage_bonus: 0,
            energy_cost_modifier: 0,
            range_modifier: 0,
            hit_effects: Vec::new(),
            special_damage: Vec::new(),
            description: description.to_string(),
        }
    }

    pub fn with_damage_bonus(mut self, bonus: i32) -> Self {
        self.damage_bonus = bonus;
        self
    }

    pub fn with_energy_cost_modifier(mut self, modifier: i32) -> Self {
        self.energy_cost_modifier = modifier;
        self
    }

    pub fn with_range_modifier(mut self, modifier: i32) -> Self {
        self.range_modifier = modifier;
        self
    }

    pub fn with_hit_effect(mut self, effect: HitEffect) -> Self {
        self.hit_effects.push(effect);
        self
    }

    pub fn with_special_damage(mut self, material: MaterialType, damage: i32) -> Self {
        self.special_damage.push((material, damage));
        self
    }

    pub fn applies_to_weapon_type(&self, weapon_type: WeaponType) -> bool {
        match self.applicable_to {
            WeaponApplicationType::MeleeOnly => weapon_type == WeaponType::Melee,
            WeaponApplicationType::RangedOnly => weapon_type == WeaponType::Ranged,
            WeaponApplicationType::Both => true,
        }
    }
}

/// Get all available gun prefixes
pub fn get_gun_prefixes() -> Vec<WeaponModifier> {
    vec![
        WeaponModifier::new(
            "Hair-Trigger",
            WeaponModifierType::Prefix,
            WeaponApplicationType::RangedOnly,
            "This gun's trigger has been filed down for a lightning-fast draw.",
        )
        .with_energy_cost_modifier(-30),
        WeaponModifier::new(
            "Modified",
            WeaponModifierType::Prefix,
            WeaponApplicationType::RangedOnly,
            "Custom work has improved this weapon's stopping power.",
        )
        .with_damage_bonus(2),
        WeaponModifier::new(
            "Sawed-Off",
            WeaponModifierType::Prefix,
            WeaponApplicationType::RangedOnly,
            "The barrel has been shortened for maximum impact at close range.",
        )
        .with_hit_effect(HitEffect::Knockback {
            strength: 1.5,
            chance: 0.5,
        })
        .with_range_modifier(-2),
        WeaponModifier::new(
            "Long-Barrel",
            WeaponModifierType::Prefix,
            WeaponApplicationType::RangedOnly,
            "Extended barrel provides superior accuracy and range.",
        )
        .with_damage_bonus(1)
        .with_range_modifier(4),
        WeaponModifier::new(
            "Rusty",
            WeaponModifierType::Prefix,
            WeaponApplicationType::RangedOnly,
            "Age and neglect have left this weapon prone to causing infection.",
        )
        .with_hit_effect(HitEffect::Bleeding {
            damage_per_tick: 1,
            duration_ticks: 400,
            chance: 0.15,
            can_stack: false,
        }),
        WeaponModifier::new(
            "Hot-Loaded",
            WeaponModifierType::Prefix,
            WeaponApplicationType::RangedOnly,
            "Overcharged rounds leave burning wounds.",
        )
        .with_hit_effect(HitEffect::Burning {
            damage_per_tick: 3,
            duration_ticks: 300,
            chance: 0.2,
        }),
        WeaponModifier::new(
            "Custom",
            WeaponModifierType::Prefix,
            WeaponApplicationType::RangedOnly,
            "Fine craftsmanship and custom parts make this weapon exceptional.",
        )
        .with_damage_bonus(1)
        .with_energy_cost_modifier(-20),
        WeaponModifier::new(
            "Worn",
            WeaponModifierType::Prefix,
            WeaponApplicationType::RangedOnly,
            "Years of use have worn this iron smooth as silk.",
        )
        .with_damage_bonus(-1)
        .with_energy_cost_modifier(-40),
        WeaponModifier::new(
            "Heavy-Bore",
            WeaponModifierType::Prefix,
            WeaponApplicationType::RangedOnly,
            "Massive bore diameter delivers devastating impact.",
        )
        .with_damage_bonus(3)
        .with_hit_effect(HitEffect::Knockback {
            strength: 1.0,
            chance: 0.4,
        }),
    ]
}

/// Get all available gun suffixes
pub fn get_gun_suffixes() -> Vec<WeaponModifier> {
    vec![
        WeaponModifier::new(
            "of the Lawman",
            WeaponModifierType::Suffix,
            WeaponApplicationType::RangedOnly,
            "Carried by those who bring order to chaos.",
        )
        .with_damage_bonus(1), // Placeholder for accuracy bonus when implemented
        WeaponModifier::new(
            "of the Outlaw",
            WeaponModifierType::Suffix,
            WeaponApplicationType::RangedOnly,
            "This weapon strikes fear into the hearts of enemies.",
        )
        .with_hit_effect(HitEffect::Stun {
            duration_ticks: 150,
            chance: 0.2,
        }),
        WeaponModifier::new(
            "of Thunder",
            WeaponModifierType::Suffix,
            WeaponApplicationType::RangedOnly,
            "Each shot rings out like a thunderclap.",
        )
        .with_hit_effect(HitEffect::Knockback {
            strength: 1.0,
            chance: 0.5,
        }),
        WeaponModifier::new(
            "of Lead Poisoning",
            WeaponModifierType::Suffix,
            WeaponApplicationType::RangedOnly,
            "Dirty bullets leave lasting damage.",
        )
        .with_hit_effect(HitEffect::Poison {
            damage_per_tick: 1,
            duration_ticks: 800,
            chance: 0.15,
        }),
        WeaponModifier::new(
            "of the Quick Draw",
            WeaponModifierType::Suffix,
            WeaponApplicationType::RangedOnly,
            "Lightning-fast action for rapid fire.",
        )
        .with_energy_cost_modifier(-40),
        WeaponModifier::new(
            "of Stopping Power",
            WeaponModifierType::Suffix,
            WeaponApplicationType::RangedOnly,
            "Hits like a mule kick.",
        )
        .with_hit_effect(HitEffect::Stun {
            duration_ticks: 200,
            chance: 0.3,
        }),
        WeaponModifier::new(
            "of the Marksman",
            WeaponModifierType::Suffix,
            WeaponApplicationType::RangedOnly,
            "Precision engineering for deadly accuracy.",
        )
        .with_damage_bonus(2), // Placeholder for crit bonus when implemented
    ]
}

/// Get all available melee prefixes
pub fn get_melee_prefixes() -> Vec<WeaponModifier> {
    vec![
        WeaponModifier::new(
            "Sharp",
            WeaponModifierType::Prefix,
            WeaponApplicationType::MeleeOnly,
            "A keen edge that cuts deep and true.",
        )
        .with_damage_bonus(1)
        .with_hit_effect(HitEffect::Bleeding {
            damage_per_tick: 1,
            duration_ticks: 500,
            chance: 0.25,
            can_stack: false,
        }),
        WeaponModifier::new(
            "Jagged",
            WeaponModifierType::Prefix,
            WeaponApplicationType::MeleeOnly,
            "Crude serrations tear flesh with every strike.",
        )
        .with_hit_effect(HitEffect::Bleeding {
            damage_per_tick: 2,
            duration_ticks: 300,
            chance: 0.4,
            can_stack: false,
        }),
        WeaponModifier::new(
            "Serrated",
            WeaponModifierType::Prefix,
            WeaponApplicationType::MeleeOnly,
            "Saw-like teeth cause terrible wounds.",
        )
        .with_hit_effect(HitEffect::Bleeding {
            damage_per_tick: 1,
            duration_ticks: 400,
            chance: 0.3,
            can_stack: true,
        }),
        WeaponModifier::new(
            "Rusted",
            WeaponModifierType::Prefix,
            WeaponApplicationType::MeleeOnly,
            "Corrosion and grime cause festering wounds.",
        )
        .with_hit_effect(HitEffect::Poison {
            damage_per_tick: 2,
            duration_ticks: 600,
            chance: 0.2,
        }),
        WeaponModifier::new(
            "Heavy",
            WeaponModifierType::Prefix,
            WeaponApplicationType::MeleeOnly,
            "Brutal weight behind every swing.",
        )
        .with_damage_bonus(2)
        .with_hit_effect(HitEffect::Knockback {
            strength: 1.0,
            chance: 0.4,
        }),
        WeaponModifier::new(
            "Barbed",
            WeaponModifierType::Prefix,
            WeaponApplicationType::MeleeOnly,
            "Wicked hooks tear flesh on withdrawal.",
        )
        .with_hit_effect(HitEffect::Bleeding {
            damage_per_tick: 2,
            duration_ticks: 600,
            chance: 0.5,
            can_stack: false,
        }),
        WeaponModifier::new(
            "Crude",
            WeaponModifierType::Prefix,
            WeaponApplicationType::MeleeOnly,
            "Rough construction causes grievous wounds.",
        )
        .with_damage_bonus(-1)
        .with_hit_effect(HitEffect::Bleeding {
            damage_per_tick: 2,
            duration_ticks: 400,
            chance: 0.6,
            can_stack: false,
        }),
        WeaponModifier::new(
            "Balanced",
            WeaponModifierType::Prefix,
            WeaponApplicationType::MeleeOnly,
            "Perfect weight distribution allows swift strikes.",
        )
        .with_energy_cost_modifier(-20),
        WeaponModifier::new(
            "Brutal",
            WeaponModifierType::Prefix,
            WeaponApplicationType::MeleeOnly,
            "Designed for maximum carnage.",
        )
        .with_damage_bonus(3)
        .with_hit_effect(HitEffect::Stun {
            duration_ticks: 150,
            chance: 0.2,
        }),
    ]
}

/// Get all available melee suffixes
pub fn get_melee_suffixes() -> Vec<WeaponModifier> {
    vec![
        WeaponModifier::new(
            "of Wounding",
            WeaponModifierType::Suffix,
            WeaponApplicationType::MeleeOnly,
            "Every strike leaves lasting damage.",
        )
        .with_hit_effect(HitEffect::Bleeding {
            damage_per_tick: 1,
            duration_ticks: 600,
            chance: 0.35,
            can_stack: false,
        }),
        WeaponModifier::new(
            "of Maiming",
            WeaponModifierType::Suffix,
            WeaponApplicationType::MeleeOnly,
            "Crippling blows that slow the enemy.",
        )
        .with_hit_effect(HitEffect::Slow {
            speed_reduction: 0.3,
            duration_ticks: 500,
            chance: 0.25,
        }),
        WeaponModifier::new(
            "of Butchery",
            WeaponModifierType::Suffix,
            WeaponApplicationType::MeleeOnly,
            "Designed for cutting flesh.",
        )
        .with_special_damage(MaterialType::Flesh, 2),
        WeaponModifier::new(
            "of the Prospector",
            WeaponModifierType::Suffix,
            WeaponApplicationType::MeleeOnly,
            "Hardened steel breaks through stone.",
        )
        .with_special_damage(MaterialType::Stone, 3),
        WeaponModifier::new(
            "of the Lumberjack",
            WeaponModifierType::Suffix,
            WeaponApplicationType::MeleeOnly,
            "Fells trees and men with equal ease.",
        )
        .with_special_damage(MaterialType::Wood, 3),
        WeaponModifier::new(
            "of Stunning Blows",
            WeaponModifierType::Suffix,
            WeaponApplicationType::MeleeOnly,
            "Strikes that leave enemies dazed.",
        )
        .with_hit_effect(HitEffect::Stun {
            duration_ticks: 200,
            chance: 0.2,
        }),
        WeaponModifier::new(
            "of Swift Strikes",
            WeaponModifierType::Suffix,
            WeaponApplicationType::MeleeOnly,
            "Lightning-fast attacks catch foes off guard.",
        )
        .with_energy_cost_modifier(-30),
    ]
}

/// Pick a random prefix for the given weapon type
pub fn pick_random_prefix(weapon_type: WeaponType, rand: &mut Rand) -> Option<WeaponModifier> {
    let prefixes = match weapon_type {
        WeaponType::Melee => get_melee_prefixes(),
        WeaponType::Ranged => get_gun_prefixes(),
    };

    if prefixes.is_empty() {
        return None;
    }

    let index = (rand.random() * prefixes.len() as f32) as usize;
    prefixes.into_iter().nth(index)
}

/// Pick a random suffix for the given weapon type
pub fn pick_random_suffix(weapon_type: WeaponType, rand: &mut Rand) -> Option<WeaponModifier> {
    let suffixes = match weapon_type {
        WeaponType::Melee => get_melee_suffixes(),
        WeaponType::Ranged => get_gun_suffixes(),
    };

    if suffixes.is_empty() {
        return None;
    }

    let index = (rand.random() * suffixes.len() as f32) as usize;
    suffixes.into_iter().nth(index)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Component, SerializableComponent)]
pub enum ItemRarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

impl ItemRarity {
    /// Get the wild west themed name for this rarity
    pub fn get_display_name(&self) -> &'static str {
        match self {
            ItemRarity::Common => "Standard Issue",
            ItemRarity::Uncommon => "Trail-Worn",
            ItemRarity::Rare => "Frontier Special",
            ItemRarity::Epic => "Legendary Outlaw's",
            ItemRarity::Legendary => "Famous Gunslinger's",
        }
    }

    /// Get the maximum number of modifiers for this rarity
    pub fn max_modifiers(&self) -> usize {
        match self {
            ItemRarity::Common => 0,
            ItemRarity::Uncommon => 1,
            ItemRarity::Rare => 2,
            ItemRarity::Epic => 2,
            ItemRarity::Legendary => 2,
        }
    }

    /// Get the minimum number of modifiers for this rarity
    pub fn min_modifiers(&self) -> usize {
        match self {
            ItemRarity::Common => 0,
            ItemRarity::Uncommon => 1,
            ItemRarity::Rare => 1,
            ItemRarity::Epic => 2,
            ItemRarity::Legendary => 2,
        }
    }

    /// Roll for a random rarity with weighted chances
    pub fn roll_random(rand: &mut Rand) -> Self {
        let roll = rand.random();

        if roll < 0.60 {
            ItemRarity::Common
        } else if roll < 0.85 {
            ItemRarity::Uncommon
        } else if roll < 0.95 {
            ItemRarity::Rare
        } else if roll < 0.995 {
            ItemRarity::Epic
        } else {
            ItemRarity::Legendary
        }
    }
}

/// Famous legendary weapon names
pub fn get_legendary_gun_names() -> Vec<&'static str> {
    vec![
        "Ol' Bessie",
        "The Judge",
        "Widowmaker",
        "Peacekeeper",
        "The Last Word",
        "Dead Eye",
        "Six Feet Under",
        "Hellbringer",
        "Iron Justice",
        "The Equalizer",
    ]
}

pub fn get_legendary_melee_names() -> Vec<&'static str> {
    vec![
        "Bone Cleaver",
        "Scalp Hunter",
        "The Prospector's Friend",
        "Railroad Spike",
        "Cherokee Rose",
        "Mountain Man's Fury",
        "Widow's Blade",
        "Blood Drinker",
        "Thunder Strike",
        "Devil's Tooth",
    ]
}

/// Pick a random legendary name for the weapon type
pub fn pick_legendary_name(weapon_type: WeaponType, rand: &mut Rand) -> Option<String> {
    let names = match weapon_type {
        WeaponType::Melee => get_legendary_melee_names(),
        WeaponType::Ranged => get_legendary_gun_names(),
    };

    if names.is_empty() {
        return None;
    }

    let index = (rand.random() * names.len() as f32) as usize;
    names.get(index).map(|&name| name.to_string())
}
