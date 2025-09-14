use crate::engine::{AudioKey, SerializableComponent};
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Component, SerializableComponent,
)]
pub enum CreatureType {
    Bear,
    Bandit,
    Coyote,
    Rattlesnake,
    Bat,
}

impl CreatureType {
    pub fn death_audio_key(self) -> AudioKey {
        match self {
            CreatureType::Bear => AudioKey::Growl1,
            CreatureType::Bandit => AudioKey::Pain1,
            CreatureType::Coyote => AudioKey::Bark1,
            CreatureType::Rattlesnake => AudioKey::Hiss1,
            CreatureType::Bat => AudioKey::Hiss1,
        }
    }
}
