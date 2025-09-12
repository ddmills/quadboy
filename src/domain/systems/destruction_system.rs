use bevy_ecs::prelude::*;

use crate::domain::MaterialType;

#[derive(Debug, Clone, Copy)]
pub enum DestructionCause {
    Attack,
}

#[derive(Event)]
pub struct EntityDestroyedEvent {
    pub entity: Entity,
    pub position: (usize, usize, usize),
    pub material_type: Option<MaterialType>,
}

impl EntityDestroyedEvent {
    pub fn new(entity: Entity, position: (usize, usize, usize), _cause: DestructionCause) -> Self {
        Self {
            entity,
            position,
            material_type: None,
        }
    }

    pub fn with_material_type(
        entity: Entity,
        position: (usize, usize, usize),
        _cause: DestructionCause,
        material_type: MaterialType,
    ) -> Self {
        Self {
            entity,
            position,
            material_type: Some(material_type),
        }
    }
}
