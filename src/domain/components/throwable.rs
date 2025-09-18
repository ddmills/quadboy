use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::engine::SerializableComponent;

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct Throwable {
    pub base_range: usize,
    pub particle_char: char,
    pub throwable_fg1: u32,
}

impl Throwable {
    pub fn new(base_range: usize, particle_char: char, throwable_fg1: u32) -> Self {
        Self {
            base_range,
            particle_char,
            throwable_fg1,
        }
    }

    pub fn calculate_throw_range(&self, strength: u32) -> usize {
        self.base_range + strength as usize
    }
}

impl Default for Throwable {
    fn default() -> Self {
        Self::new(5, '?', 0xFFFFFF)
    }
}
