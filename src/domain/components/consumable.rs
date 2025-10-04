use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::engine::SerializableComponent;

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct Consumable {
    pub effect: ConsumableEffect,
    pub consume_on_use: bool,
}

impl Consumable {
    pub fn new(effect: ConsumableEffect, consume_on_use: bool) -> Self {
        Self {
            effect,
            consume_on_use,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub enum ConsumableEffect {
    Heal(i32),
    RestoreArmor(i32),
    Poison(i32, u32),
    Buff(String, i32, u32),
    Cure,
}
