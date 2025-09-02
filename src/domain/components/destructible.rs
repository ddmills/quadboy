use crate::engine::{AudioCollection, SerializableComponent};
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MaterialType {
    Stone, // Rocks, boulders, stone walls - requires pickaxe
    Wood,  // Trees, wooden doors - requires hatchet/axe
    Flesh, // Living creatures - damaged by weapons
}

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct Destructible {
    pub durability: i32,
    pub max_durability: i32,
    pub material_type: MaterialType,
}

impl MaterialType {
    pub fn hit_audio_collection(self) -> Option<AudioCollection> {
        match self {
            MaterialType::Stone => Some(AudioCollection::Mining),
            MaterialType::Wood => Some(AudioCollection::Chopping),
            MaterialType::Flesh => None,
        }
    }

    pub fn destroy_audio_collection(self) -> Option<AudioCollection> {
        match self {
            MaterialType::Stone => Some(AudioCollection::RockCrumble),
            MaterialType::Wood => Some(AudioCollection::Vegetation),
            MaterialType::Flesh => None,
        }
    }
}

impl Destructible {
    pub fn new(durability: i32, material_type: MaterialType) -> Self {
        Self {
            durability,
            max_durability: durability,
            material_type,
        }
    }

    pub fn take_damage(&mut self, damage: i32) {
        self.durability = (self.durability - damage).max(0);
    }

    pub fn is_destroyed(&self) -> bool {
        self.durability <= 0
    }
}
