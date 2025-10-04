use bevy_ecs::prelude::*;

use crate::domain::MaterialType;

#[derive(Debug, Clone, Copy)]
pub enum DestructionCause {
    Attack { attacker: Entity },
    Environmental,
    Scripted,
    Consumed,
}

#[derive(Event)]
pub struct EntityDestroyedEvent {
    pub entity: Entity,
    pub position: (usize, usize, usize),
    pub cause: DestructionCause,
    pub material_type: Option<MaterialType>,
}

impl EntityDestroyedEvent {
    pub fn new(entity: Entity, position: (usize, usize, usize), cause: DestructionCause) -> Self {
        Self {
            entity,
            position,
            cause,
            material_type: None,
        }
    }

    pub fn with_material_type(
        entity: Entity,
        position: (usize, usize, usize),
        cause: DestructionCause,
        material_type: MaterialType,
    ) -> Self {
        Self {
            entity,
            position,
            cause,
            material_type: Some(material_type),
        }
    }

    pub fn with_attacker(
        entity: Entity,
        position: (usize, usize, usize),
        attacker: Entity,
        material_type: MaterialType,
    ) -> Self {
        Self {
            entity,
            position,
            cause: DestructionCause::Attack { attacker },
            material_type: Some(material_type),
        }
    }

    pub fn environmental(
        entity: Entity,
        position: (usize, usize, usize),
        material_type: Option<MaterialType>,
    ) -> Self {
        Self {
            entity,
            position,
            cause: DestructionCause::Environmental,
            material_type,
        }
    }
}
