use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{domain::StatType, engine::SerializableComponent};

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct Consumable {
    pub effect: ConsumableEffect,
    pub consume_on_use: bool, // Whether item is destroyed after use
}

impl Consumable {
    pub fn new(effect: ConsumableEffect, consume_on_use: bool) -> Self {
        Self {
            effect,
            consume_on_use,
        }
    }

    pub fn heal(amount: i32) -> Self {
        Self::new(ConsumableEffect::Heal(amount), true)
    }

    pub fn restore_energy(amount: i32) -> Self {
        Self::new(ConsumableEffect::RestoreEnergy(amount), true)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub enum ConsumableEffect {
    Heal(i32),                // Restore HP
    RestoreEnergy(i32),       // Restore energy/stamina
    HealAndEnergy(i32, i32),  // Restore both HP and energy (hp, energy)
    Poison(i32, u32),         // Deal damage over time (damage, duration)
    Buff(StatType, i32, u32), // Temporary stat boost (stat, amount, duration)
    Cure,                     // Remove negative effects
                              // Can be extended with more effects as needed
}
