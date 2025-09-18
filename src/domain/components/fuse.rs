use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::engine::{SerializableComponent, TICKS_PER_MINUTE};

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct Fuse {
    pub remaining_ticks: i32,
    pub explosion_radius: usize,
    pub explosion_damage: i32,
    pub lit_at_tick: u32,
}

impl Fuse {
    pub fn new(duration: i32, radius: usize, damage: i32, current_tick: u32) -> Self {
        Self {
            remaining_ticks: duration,
            explosion_radius: radius,
            explosion_damage: damage,
            lit_at_tick: current_tick,
        }
    }

    pub fn tick_down(&mut self, amount: i32) {
        self.remaining_ticks -= amount;
    }

    pub fn is_expired(&self) -> bool {
        self.remaining_ticks <= 0
    }

    pub fn get_countdown_display(&self) -> String {
        format!(
            "{{U|({}m)}}",
            self.remaining_ticks.max(0) as u32 / TICKS_PER_MINUTE
        )
    }
}
