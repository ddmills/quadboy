use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{domain::ConditionType, engine::SerializableComponent};

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct ConditionBlink {
    pub conditions: Vec<ConditionBlinkData>,
    pub current_condition_index: usize,
    pub cycle_timer: f32,
    pub cycle_duration: f32, // Time per condition color (e.g., 0.3s)
    pub blink_timer: f32,
    pub blink_rate: f32, // Blinks per second (e.g., 3.0)
    pub blink_on: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ConditionBlinkData {
    pub condition_type: ConditionType,
    pub color: u32,
}

impl ConditionBlink {
    pub fn new() -> Self {
        Self {
            conditions: Vec::new(),
            current_condition_index: 0,
            cycle_timer: 0.0,
            cycle_duration: 0.3, // 300ms per condition
            blink_timer: 0.0,
            blink_rate: 3.0, // 3 blinks per second
            blink_on: true,
        }
    }

    pub fn add_condition(&mut self, condition_type: ConditionType, color: u32) {
        // Check if condition already exists (for non-stacking conditions)
        if let Some(existing) = self
            .conditions
            .iter_mut()
            .find(|c| c.condition_type == condition_type)
        {
            existing.color = color;
            return;
        }

        self.conditions.push(ConditionBlinkData {
            condition_type,
            color,
        });
    }

    pub fn remove_condition(&mut self, condition_type: &ConditionType) {
        self.conditions
            .retain(|c| &c.condition_type != condition_type);

        // Adjust current index if we removed the currently displayed condition
        if self.current_condition_index >= self.conditions.len() && !self.conditions.is_empty() {
            self.current_condition_index = 0;
        }
    }

    pub fn get_current_color(&self) -> Option<u32> {
        if self.conditions.is_empty() {
            return None;
        }

        let index = self.current_condition_index % self.conditions.len();
        Some(self.conditions[index].color)
    }

    pub fn is_empty(&self) -> bool {
        self.conditions.is_empty()
    }

    pub fn update_timers(&mut self, dt: f32) {
        // Update blink timer
        self.blink_timer += dt;
        let blink_interval = 1.0 / self.blink_rate / 2.0; // divide by 2 for on/off cycle

        if self.blink_timer >= blink_interval {
            self.blink_on = !self.blink_on;
            self.blink_timer = 0.0;
        }

        // Update cycle timer (only if multiple conditions)
        if self.conditions.len() > 1 {
            self.cycle_timer += dt;
            if self.cycle_timer >= self.cycle_duration {
                self.current_condition_index =
                    (self.current_condition_index + 1) % self.conditions.len();
                self.cycle_timer = 0.0;
            }
        }
    }
}

impl Default for ConditionBlink {
    fn default() -> Self {
        Self::new()
    }
}
