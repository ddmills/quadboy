use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::engine::SerializableComponent;

#[derive(
    Component,
    Serialize,
    Deserialize,
    Clone,
    SerializableComponent,
    Debug,
    PartialEq,
    Eq,
    Hash,
    Copy,
)]
pub struct StableId(pub u64);

impl StableId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

#[derive(Resource, Serialize, Deserialize, Default)]
pub struct StableIdRegistry {
    next_id: u64,
    #[serde(skip)]
    entity_to_id: HashMap<Entity, u64>,
    #[serde(skip)]
    id_to_entity: HashMap<u64, Entity>,
}

impl StableIdRegistry {
    pub fn new() -> Self {
        Self {
            next_id: 1,
            entity_to_id: HashMap::new(),
            id_to_entity: HashMap::new(),
        }
    }

    pub fn generate_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn register(&mut self, entity: Entity, id: u64) {
        self.entity_to_id.insert(entity, id);
        self.id_to_entity.insert(id, entity);
    }

    pub fn unregister(&mut self, entity: Entity) {
        if let Some(id) = self.entity_to_id.remove(&entity) {
            self.id_to_entity.remove(&id);
        }
    }

    pub fn get_entity(&self, id: u64) -> Option<Entity> {
        self.id_to_entity.get(&id).copied()
    }

    pub fn get_id(&self, entity: Entity) -> Option<u64> {
        self.entity_to_id.get(&entity).copied()
    }

    pub fn rebuild_mappings(&mut self, entities: Vec<(Entity, u64)>) {
        self.entity_to_id.clear();
        self.id_to_entity.clear();

        for (entity, id) in entities {
            self.register(entity, id);
        }
    }
}

pub fn assign_stable_id(entity: Entity, world: &mut World) -> u64 {
    if let Some(existing_id) = world.get::<StableId>(entity) {
        return existing_id.0;
    }

    let id = {
        let mut registry = world.resource_mut::<StableIdRegistry>();
        let id = registry.generate_id();
        registry.register(entity, id);
        id
    };

    world.entity_mut(entity).insert(StableId::new(id));
    id
}

pub fn reconcile_stable_ids(world: &mut World) {
    let mut entities = Vec::new();
    let mut query = world.query::<(Entity, &StableId)>();
    for (entity, stable_id) in query.iter(world) {
        entities.push((entity, stable_id.0));
    }

    let mut registry = world.resource_mut::<StableIdRegistry>();
    registry.rebuild_mappings(entities);
}
