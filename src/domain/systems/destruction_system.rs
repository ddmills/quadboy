use bevy_ecs::prelude::*;

#[derive(Debug, Clone, Copy)]
pub enum DestructionCause {
    Attack,
}

#[derive(Event)]
pub struct EntityDestroyedEvent {
    pub entity: Entity,
    pub position: (usize, usize, usize),
    pub cause: DestructionCause,
}

impl EntityDestroyedEvent {
    pub fn new(entity: Entity, position: (usize, usize, usize), cause: DestructionCause) -> Self {
        Self {
            entity,
            position,
            cause,
        }
    }
}
