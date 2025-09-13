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