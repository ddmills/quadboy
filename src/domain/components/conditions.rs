use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::engine::{SerializableComponent, StableId};

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent, Default)]
pub struct ActiveConditions {
    pub conditions: Vec<Condition>,
}

impl ActiveConditions {
    pub fn new() -> Self {
        Self {
            conditions: Vec::new(),
        }
    }

    pub fn add_condition(&mut self, condition: Condition) {
        self.conditions.push(condition);
    }

    pub fn remove_condition(&mut self, condition_type: &ConditionType) {
        self.conditions
            .retain(|c| &c.condition_type != condition_type);
    }

    pub fn has_condition(&self, condition_type: &ConditionType) -> bool {
        self.conditions
            .iter()
            .any(|c| &c.condition_type == condition_type)
    }

    pub fn get_conditions_of_type(&self, condition_type: &ConditionType) -> Vec<&Condition> {
        self.conditions
            .iter()
            .filter(|c| &c.condition_type == condition_type)
            .collect()
    }

    pub fn is_empty(&self) -> bool {
        self.conditions.is_empty()
    }

    pub fn clear(&mut self) {
        self.conditions.clear();
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Condition {
    pub condition_type: ConditionType,
    pub duration_remaining: u32,
    pub intensity: f32,
    pub source: ConditionSource,
    #[serde(skip)]
    pub accumulated_effect: f32,
    #[serde(skip)]
    pub particle_spawner_entity: Option<Entity>,
}

impl Condition {
    pub fn new(
        condition_type: ConditionType,
        duration_ticks: u32,
        intensity: f32,
        source: ConditionSource,
    ) -> Self {
        Self {
            condition_type,
            duration_remaining: duration_ticks,
            intensity,
            source,
            accumulated_effect: 0.0,
            particle_spawner_entity: None,
        }
    }

    pub fn tick(&mut self, delta_ticks: u32) -> bool {
        if self.duration_remaining > delta_ticks {
            self.duration_remaining -= delta_ticks;
            false
        } else {
            self.duration_remaining = 0;
            true
        }
    }

    pub fn is_expired(&self) -> bool {
        self.duration_remaining == 0
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ConditionType {
    // Damage over time effects
    Poisoned {
        damage_per_tick: i32,
        tick_interval: u32,
    },
    Bleeding {
        damage_per_tick: i32,
        can_stack: bool,
    },
    Burning {
        damage_per_tick: i32,
        spread_chance: f32,
    },

    // AI behavior modifiers
    Feared {
        flee_from: StableId,
        min_distance: f32,
    },
    Taunted {
        move_toward: StableId,
        force_target: bool,
    },
    Confused {
        random_chance: f32,
    },
}

impl ConditionType {
    pub fn get_base_duration_ticks(&self) -> u32 {
        match self {
            ConditionType::Poisoned { .. } => 1000,
            ConditionType::Bleeding { .. } => 800,
            ConditionType::Burning { .. } => 600,
            ConditionType::Feared { .. } => 600,
            ConditionType::Taunted { .. } => 400,
            ConditionType::Confused { .. } => 500,
        }
    }

    pub fn can_stack(&self) -> bool {
        match self {
            ConditionType::Poisoned { .. } => false,
            ConditionType::Bleeding { can_stack, .. } => *can_stack,
            ConditionType::Burning { .. } => false,
            ConditionType::Feared { .. } => false,
            ConditionType::Taunted { .. } => false,
            ConditionType::Confused { .. } => false,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ConditionSource {
    Entity(StableId),
    Environment,
    Item(StableId),
    Spell {
        caster: StableId,
        spell_name: String,
    },
    Unknown,
}

impl ConditionSource {
    pub fn entity(stable_id: StableId) -> Self {
        Self::Entity(stable_id)
    }

    pub fn environment() -> Self {
        Self::Environment
    }

    pub fn item(stable_id: StableId) -> Self {
        Self::Item(stable_id)
    }

    pub fn spell(caster: StableId, spell_name: String) -> Self {
        Self::Spell { caster, spell_name }
    }

    pub fn get_source_id(&self) -> Option<StableId> {
        match self {
            ConditionSource::Entity(id) => Some(*id),
            ConditionSource::Item(id) => Some(*id),
            ConditionSource::Spell { caster, .. } => Some(*caster),
            _ => None,
        }
    }
}
