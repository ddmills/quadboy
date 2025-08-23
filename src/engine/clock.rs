use bevy_ecs::prelude::*;

#[derive(Resource, Default)]
pub struct Clock {
    tick: u32,
    tick_delta: u32,
}

impl Clock {
    pub fn new() -> Self {
        Self {
            tick: 0,
            tick_delta: 1,
        }
    }

    pub fn increment_tick(&mut self, amount: u32) {
        self.tick += amount;
        self.tick_delta += amount;
    }

    pub fn current_tick(&self) -> u32 {
        self.tick
    }

    pub fn tick_delta(&self) -> u32 {
        self.tick_delta
    }

    pub fn clear_tick_delta(&mut self) {
        self.tick_delta = 0;
    }

    pub fn set_tick(&mut self, value: u32) {
        self.tick = value;
    }

    pub fn current_turn(&self) -> u32 {
        self.tick / 1000
    }

    pub fn sub_turn(&self) -> u32 {
        self.tick % 1000
    }
}
