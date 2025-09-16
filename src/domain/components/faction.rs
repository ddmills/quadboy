use crate::engine::SerializableComponent;
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FactionId {
    Player,
    Bandits,
    Wildlife,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FactionModifier {
    Charmed { duration_ticks: u32 },
    Enraged { duration_ticks: u32 },
    Feared { duration_ticks: u32 },
}

impl FactionModifier {
    pub fn tick(&mut self) -> bool {
        match self {
            FactionModifier::Charmed { duration_ticks } => {
                if *duration_ticks > 0 {
                    *duration_ticks -= 1;
                    *duration_ticks > 0
                } else {
                    false
                }
            }
            FactionModifier::Enraged { duration_ticks } => {
                if *duration_ticks > 0 {
                    *duration_ticks -= 1;
                    *duration_ticks > 0
                } else {
                    false
                }
            }
            FactionModifier::Feared { duration_ticks } => {
                if *duration_ticks > 0 {
                    *duration_ticks -= 1;
                    *duration_ticks > 0
                } else {
                    false
                }
            }
        }
    }
}

#[derive(Component, Serialize, Deserialize, Clone, Debug, SerializableComponent)]
pub struct FactionMember {
    pub faction_id: FactionId,
    pub modifiers: HashMap<String, FactionModifier>,
}

impl FactionMember {
    pub fn new(faction_id: FactionId) -> Self {
        Self {
            faction_id,
            modifiers: HashMap::new(),
        }
    }

    pub fn add_modifier(&mut self, name: String, modifier: FactionModifier) {
        self.modifiers.insert(name, modifier);
    }

    pub fn remove_modifier(&mut self, name: &str) {
        self.modifiers.remove(name);
    }

    pub fn has_modifier(&self, name: &str) -> bool {
        self.modifiers.contains_key(name)
    }

    pub fn get_modifier(&self, name: &str) -> Option<&FactionModifier> {
        self.modifiers.get(name)
    }

    pub fn tick_modifiers(&mut self) {
        self.modifiers.retain(|_, modifier| modifier.tick());
    }
}
