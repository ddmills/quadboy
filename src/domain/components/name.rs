use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use crate::engine::SerializableComponent;

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct Name {
    value: String,
}

impl Name {
    pub fn new<S: Into<String>>(value: S) -> Self {
        Self { value: value.into() }
    }

    pub fn get(&self) -> &str {
        &self.value
    }
}