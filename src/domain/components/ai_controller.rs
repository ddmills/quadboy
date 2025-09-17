use crate::engine::SerializableComponent;
use crate::rendering::Position;
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct AiBehaviorModifiers {
    pub aggression: f32,
    pub caution: f32,
    pub speed_multiplier: f32,
    pub energy_efficiency: f32,
    pub group_behavior: bool,
    pub territorial: bool,
    pub noise_sensitivity: f32,
    pub light_sensitivity: f32,
}

impl Default for AiBehaviorModifiers {
    fn default() -> Self {
        Self {
            aggression: 1.0,
            caution: 1.0,
            speed_multiplier: 1.0,
            energy_efficiency: 1.0,
            group_behavior: false,
            territorial: false,
            noise_sensitivity: 1.0,
            light_sensitivity: 1.0,
        }
    }
}

impl AiBehaviorModifiers {
    pub fn aggressive() -> Self {
        Self {
            aggression: 1.5,
            caution: 0.5,
            speed_multiplier: 1.2,
            territorial: true,
            ..Default::default()
        }
    }

    pub fn cautious() -> Self {
        Self {
            aggression: 0.5,
            caution: 1.8,
            speed_multiplier: 0.8,
            noise_sensitivity: 1.5,
            light_sensitivity: 1.3,
            ..Default::default()
        }
    }

    pub fn pack_hunter() -> Self {
        Self {
            aggression: 1.2,
            group_behavior: true,
            speed_multiplier: 1.1,
            ..Default::default()
        }
    }

    pub fn efficient() -> Self {
        Self {
            energy_efficiency: 1.3,
            speed_multiplier: 0.9,
            caution: 1.2,
            ..Default::default()
        }
    }
}

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
    #[serde(skip)]
    pub cached_home_path: Option<Vec<(usize, usize, usize)>>,
    #[serde(skip)]
    pub cached_pursuit_path: Option<Vec<(usize, usize, usize)>>,
    pub behavior_modifiers: AiBehaviorModifiers,
}

impl AiController {
    pub fn new(template: AiTemplate, home_position: Position) -> Self {
        Self {
            template,
            home_position,
            leash_range: 40.0,
            wander_range: 10.0,
            detection_range: 15.0,
            current_target: None,
            state: AiState::Idle,
            cached_home_path: None,
            cached_pursuit_path: None,
            behavior_modifiers: AiBehaviorModifiers::default(),
        }
    }

    pub fn with_ranges(mut self, leash: f32, wander: f32, detection: f32) -> Self {
        self.leash_range = leash;
        self.wander_range = wander;
        self.detection_range = detection;
        self
    }

    pub fn with_behavior_modifiers(mut self, modifiers: AiBehaviorModifiers) -> Self {
        self.behavior_modifiers = modifiers;
        self
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum AiTemplate {
    BasicAggressive,
    Timid,
    Scavenger,
    Ambush { strike_range: f32 },
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

impl AiState {
    /// Returns true if this AI state requires active turn processing
    pub fn is_active(&self) -> bool {
        matches!(
            self,
            AiState::Pursuing | AiState::Fighting | AiState::Returning
        )
    }
}
