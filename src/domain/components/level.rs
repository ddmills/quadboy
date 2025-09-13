use crate::{domain::GameFormulas, engine::SerializableComponent};
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct Level {
    pub current_level: u32,
    pub current_xp: u32,
    pub xp_to_next_level: u32,
}

impl Level {
    pub fn new(level: u32) -> Self {
        Self {
            current_level: level,
            current_xp: 0,
            xp_to_next_level: GameFormulas::xp_required_for_next_level(level),
        }
    }

    /// Add XP and handle level ups. Returns true if leveled up.
    pub fn add_xp(&mut self, xp: u32) -> bool {
        self.current_xp += xp;
        let mut leveled_up = false;

        while self.current_xp >= self.xp_to_next_level {
            self.current_xp -= self.xp_to_next_level;
            self.current_level += 1;
            leveled_up = true;

            self.xp_to_next_level = GameFormulas::xp_required_for_next_level(self.current_level);

            if self.current_level >= GameFormulas::MAX_LEVEL {
                self.current_xp = 0;
                break;
            }
        }

        leveled_up
    }

    pub fn xp_progress_percentage(&self) -> f32 {
        if self.xp_to_next_level == 0 {
            return 1.0;
        }
        self.current_xp as f32 / self.xp_to_next_level as f32
    }
}
