use crate::engine::SerializableComponent;
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct Label {
    value: String,
    cached: String,
    dirty: bool,
}

impl Label {
    pub fn new<S: Into<String>>(value: S) -> Self {
        let value_str = value.into();
        Self {
            cached: value_str.clone(),
            value: value_str,
            dirty: false,
        }
    }

    pub fn get(&self) -> &str {
        &self.cached
    }

    pub fn get_base(&self) -> &str {
        &self.value
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn update_cache(&mut self, new_cached: String) {
        if self.cached != new_cached {
            self.cached = new_cached;
        }
        self.dirty = false;
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }
}
