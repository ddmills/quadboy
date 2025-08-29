use crate::engine::SerializableComponent;
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct Vision {
    pub range: usize,
    pub underground_range: usize,
}

impl Vision {
    pub fn new(range: usize) -> Self {
        Self {
            range,
            underground_range: range / 2,
        }
    }

    pub fn with_underground_range(range: usize, underground_range: usize) -> Self {
        Self {
            range,
            underground_range,
        }
    }
}

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct VisionBlocker;

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct IsVisible;

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct IsExplored;

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct ApplyVisibilityEffects;

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct HideWhenNotVisible;
