use crate::engine::SerializableComponent;
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Serialize, Deserialize, Clone, Debug, SerializableComponent, Default)]
#[allow(dead_code)]
pub enum AiBehavior {
    #[default]
    Wander,
    BearAi {
        aggressive: bool,
        detection_range: f32,
    },
}
