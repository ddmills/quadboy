use crate::{
    domain::{Level, StatType, Stats},
    engine::{SerializableComponent, StableId},
};
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct Health {
    pub current: i32,
    pub current_armor: i32,
    pub last_damage_tick: u32,
    pub armor_regen_progress: u32,
    pub last_damage_source: Option<StableId>,
    // max HP is now computed from Level and Fortitude stat
    // max armor is computed from Armor stat
}

impl Health {
    /// Create Health with specific current values (for loading saved games)
    pub fn new_with_current(current: i32) -> Self {
        Self {
            current,
            current_armor: 0, // Will be set by armor system
            last_damage_tick: 0,
            armor_regen_progress: 0,
            last_damage_source: None,
        }
    }

    /// Create Health with specific current and armor values (for loading saved games)
    pub fn new_with_current_and_armor(current: i32, current_armor: i32) -> Self {
        Self {
            current,
            current_armor,
            last_damage_tick: 0,
            armor_regen_progress: 0,
            last_damage_source: None,
        }
    }

    /// Create Health for new entities (will be clamped to proper max on first update)
    pub fn new_full() -> Self {
        Self {
            current: i32::MAX,
            current_armor: i32::MAX, // Will be clamped by armor system
            last_damage_tick: 0,
            armor_regen_progress: 0,
            last_damage_source: None,
        }
    }

    /// Legacy constructor for NPCs without Level/Stats - creates static health
    pub fn new(max: i32) -> Self {
        Self {
            current: max,
            current_armor: 0, // NPCs don't have armor by default
            last_damage_tick: 0,
            armor_regen_progress: 0,
            last_damage_source: None,
        }
    }

    /// Calculate max HP using the formula: (Level * 2) + (Fortitude * 2) + 5
    pub fn get_max_hp(level: &Level, stats: &Stats) -> i32 {
        let fortitude = stats.get_stat(StatType::Fortitude);
        (level.current_level as i32 * 2) + (fortitude * 2) + 5
    }

    /// Get current and max HP as a tuple
    pub fn get_current_max(&self, level: &Level, stats: &Stats) -> (i32, i32) {
        let max_hp = Self::get_max_hp(level, stats);
        (self.current, max_hp)
    }

    /// Get HP as a percentage (0.0 to 1.0)
    pub fn get_percentage(&self, level: &Level, stats: &Stats) -> f32 {
        let max_hp = Self::get_max_hp(level, stats);
        if max_hp <= 0 {
            0.0
        } else {
            (self.current as f32 / max_hp as f32).clamp(0.0, 1.0)
        }
    }

    /// Get current and max armor as a tuple
    pub fn get_current_max_armor(&self, stats: &Stats) -> (i32, i32) {
        let max_armor = stats.get_stat(StatType::Armor);
        (self.current_armor, max_armor)
    }

    /// Get armor as a percentage (0.0 to 1.0)
    pub fn get_armor_percentage(&self, stats: &Stats) -> f32 {
        let max_armor = stats.get_stat(StatType::Armor);
        if max_armor <= 0 {
            0.0
        } else {
            (self.current_armor as f32 / max_armor as f32).clamp(0.0, 1.0)
        }
    }

    /// Clamp current armor to the calculated maximum
    pub fn clamp_armor_to_max(&mut self, max_armor: i32) {
        self.current_armor = self.current_armor.min(max_armor).max(0);
    }

    /// Restore armor up to max
    pub fn restore_armor(&mut self, amount: i32, stats: &Stats) {
        let max_armor = stats.get_stat(StatType::Armor);
        self.current_armor = (self.current_armor + amount).min(max_armor);
    }

    /// Clamp current HP to the calculated maximum
    pub fn clamp_to_max(&mut self, max_hp: i32) {
        self.current = self.current.min(max_hp).max(0);
    }

    pub fn take_damage(&mut self, damage: i32, current_tick: u32) {
        self.take_damage_from_source(damage, current_tick, None);
    }

    pub fn take_damage_from_source(
        &mut self,
        damage: i32,
        current_tick: u32,
        source: Option<StableId>,
    ) {
        if damage > 0 {
            self.last_damage_tick = current_tick;
            self.armor_regen_progress = 0;
            self.last_damage_source = source;
        }

        if self.current_armor > 0 {
            // Armor absorbs damage first
            let armor_absorbed = damage.min(self.current_armor);
            self.current_armor -= armor_absorbed;
            let remaining_damage = damage - armor_absorbed;

            // Any remaining damage goes to health
            if remaining_damage > 0 {
                self.current = (self.current - remaining_damage).max(0);
            }
        } else {
            // No armor, damage goes directly to health
            self.current = (self.current - damage).max(0);
        }
    }

    pub fn is_dead(&self) -> bool {
        self.current <= 0
    }

    /// For entities with Level/Stats, use this to check if dead
    pub fn is_dead_dynamic(&self, level: &Level, stats: &Stats) -> bool {
        self.current <= 0
    }

    /// Heal up to max HP
    pub fn heal(&mut self, amount: i32, level: &Level, stats: &Stats) {
        let max_hp = Self::get_max_hp(level, stats);
        self.current = (self.current + amount).min(max_hp);
    }
}
