use crate::engine::SerializableComponent;
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct Label {
    value: String,
}

impl Label {
    pub fn new<S: Into<String>>(value: S) -> Self {
        Self {
            value: value.into(),
        }
    }

    pub fn get(&self) -> &str {
        &self.value
    }
}
