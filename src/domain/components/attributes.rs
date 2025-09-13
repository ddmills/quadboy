use crate::engine::SerializableComponent;
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct Attributes {
    pub strength: u32,
    pub dexterity: u32,
    pub constitution: u32,
    pub intelligence: u32,
}

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct AttributePoints {
    pub available: u32,
    pub spent: u32,
}

impl AttributePoints {
    pub fn new(level: u32) -> Self {
        Self {
            available: 5 + level,
            spent: 0,
        }
    }

    pub fn total_points(&self) -> u32 {
        self.available + self.spent
    }

    pub fn can_increase(&self) -> bool {
        self.available > 0
    }

    pub fn can_decrease(&self) -> bool {
        self.spent > 0
    }

    pub fn increase_attribute(&mut self) -> bool {
        if self.available > 0 {
            self.available -= 1;
            self.spent += 1;
            true
        } else {
            false
        }
    }

    pub fn decrease_attribute(&mut self) -> bool {
        if self.spent > 0 {
            self.available += 1;
            self.spent -= 1;
            true
        } else {
            false
        }
    }

    pub fn reset_all(&mut self) {
        self.available = self.total_points();
        self.spent = 0;
    }
}

impl Attributes {
    pub fn new(strength: u32, dexterity: u32, constitution: u32, intelligence: u32) -> Self {
        Self {
            strength,
            dexterity,
            constitution,
            intelligence,
        }
    }

    pub fn get_strength(&self) -> u32 {
        self.strength
    }

    pub fn get_dexterity(&self) -> u32 {
        self.dexterity
    }

    pub fn get_constitution(&self) -> u32 {
        self.constitution
    }

    pub fn get_intelligence(&self) -> u32 {
        self.intelligence
    }
}
