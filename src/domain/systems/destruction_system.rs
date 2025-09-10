use bevy_ecs::prelude::*;

#[derive(Debug, Clone, Copy)]
pub enum DestructionCause {
    Attack,
}

#[derive(Event)]
pub struct EntityDestroyedEvent {
    pub entity: Entity,
    pub position: (usize, usize, usize),
}

impl EntityDestroyedEvent {
    pub fn new(entity: Entity, position: (usize, usize, usize), _cause: DestructionCause) -> Self {
        Self {
            entity,
            position,
        }
    }
}
