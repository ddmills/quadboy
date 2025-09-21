use crate::engine::SerializableComponent;
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

/// Marks entities that never move after initial placement
/// These entities are placed in zone caches once and never updated
#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct StaticEntity;

/// Marks entities that can move and need zone cache updates
/// These entities are tracked for position changes
#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct DynamicEntity;
