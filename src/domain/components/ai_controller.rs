use crate::engine::SerializableComponent;
use crate::rendering::Position;
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Serialize, Deserialize, Clone, Debug, SerializableComponent)]
pub struct AiController {
    pub template: AiTemplate,
    pub home_position: Position,
    pub leash_range: f32,
    pub wander_range: f32,
    pub detection_range: f32,
    #[serde(skip)]
    pub current_target: Option<Entity>,
    pub state: AiState,
}

impl AiController {
    pub fn new(template: AiTemplate, home_position: Position) -> Self {
        Self {
            template,
            home_position,
            leash_range: 20.0,
            wander_range: 10.0,
            detection_range: 15.0,
            current_target: None,
            state: AiState::Idle,
        }
    }

    pub fn with_ranges(mut self, leash: f32, wander: f32, detection: f32) -> Self {
        self.leash_range = leash;
        self.wander_range = wander;
        self.detection_range = detection;
        self
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum AiTemplate {
    BasicAggressive,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum AiState {
    Idle,
    Wandering,
    Pursuing,
    Fighting,
    Fleeing,
    Returning,
}
