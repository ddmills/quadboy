use crate::engine::SerializableComponent;
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct Health {
    pub current: i32,
    pub max: i32,
}

impl Health {
    pub fn new(max: i32) -> Self {
        Self { current: max, max }
    }

    pub fn take_damage(&mut self, damage: i32) {
        self.current = (self.current - damage).max(0);
    }

    pub fn heal(&mut self, amount: i32) {
        self.current = (self.current + amount).min(self.max);
    }

    pub fn is_dead(&self) -> bool {
        self.current <= 0
    }
}
