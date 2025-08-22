use bevy_ecs::prelude::*;

#[derive(Resource, Default)]
pub struct Clock {
    tick: u32,
}

impl Clock {
    pub fn new() -> Self {
        Self { tick: 0 }
    }

    pub fn increment_tick(&mut self, amount: u32) {
        self.tick += amount;
    }

    pub fn current_tick(&self) -> u32 {
        self.tick
    }

    pub fn set_tick(&mut self, value: u32) {
        self.tick = value;
    }
}
