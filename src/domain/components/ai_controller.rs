use crate::engine::{SerializableComponent, StableId};
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Serialize, Deserialize, Clone, Debug, SerializableComponent)]
pub struct AiController {
    pub template: AiTemplate,
    pub home_position: (usize, usize, usize),
    pub leash_range: usize,
    pub wander_range: usize,
    pub detection_range: usize,
    pub current_target_id: Option<StableId>,
    pub state: AiState,
}

impl AiController {
    pub fn new(template: AiTemplate, home_position: (usize, usize, usize)) -> Self {
        Self {
            template,
            home_position,
            leash_range: 40,
            wander_range: 3,
            detection_range: 6,
            current_target_id: None,
            state: AiState::Idle,
        }
    }

    #[allow(dead_code)]
    pub fn with_ranges(mut self, leash: usize, wander: usize, detection: usize) -> Self {
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
    Waiting,
}
