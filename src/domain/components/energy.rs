use crate::engine::SerializableComponent;
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct Energy {
    pub value: i32,
}

impl Energy {
    pub fn new(value: i32) -> Self {
        Self { value }
    }

    pub fn has_energy(&self) -> bool {
        self.value >= 0
    }

    pub fn add_energy(&mut self, amount: u32) {
        self.value += amount as i32;

        // this shouldn't really happen
        if self.value > 0 {
            self.value = 0;
        }
    }

    pub fn consume_energy(&mut self, amount: i32) {
        self.value -= amount;
    }
}
