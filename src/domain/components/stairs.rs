use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use crate::engine::SerializableComponent;

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct StairDown;

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct StairUp;