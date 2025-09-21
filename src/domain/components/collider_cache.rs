use bevy_ecs::prelude::*;

use crate::{
    cfg::ZONE_SIZE,
    common::{Grid, HashGrid},
    domain::ColliderFlags,
};

#[derive(Clone)]
pub struct ColliderCache {
    entities: HashGrid<Entity>,
    flags: Grid<ColliderFlags>,
}

impl ColliderCache {
    pub fn new() -> Self {
        Self {
            entities: HashGrid::init(ZONE_SIZE.0, ZONE_SIZE.1),
            flags: Grid::init_fill(ZONE_SIZE.0, ZONE_SIZE.1, |_, _| ColliderFlags::empty()),
        }
    }

    pub fn insert(&mut self, x: usize, y: usize, entity: Entity, entity_flags: ColliderFlags) {
        self.entities.insert(x, y, entity);

        if let Some(current_flags) = self.flags.get_mut(x, y) {
            *current_flags |= entity_flags;
        }
    }

    pub fn remove(&mut self, entity: &Entity) -> Option<(usize, usize)> {
        self.entities.remove(entity)
    }

    pub fn get_entities(&self, x: usize, y: usize) -> Option<&Vec<Entity>> {
        self.entities.get(x, y)
    }

    // For backward compatibility with existing HashGrid API
    pub fn get(&self, x: usize, y: usize) -> Option<&Vec<Entity>> {
        self.entities.get(x, y)
    }

    pub fn get_flags(&self, x: usize, y: usize) -> ColliderFlags {
        self.flags
            .get(x, y)
            .copied()
            .unwrap_or(ColliderFlags::empty())
    }

    pub fn has_entity(&self, entity: &Entity) -> bool {
        self.entities.has(entity)
    }

    // For backward compatibility with existing HashGrid API
    pub fn has(&self, entity: &Entity) -> bool {
        self.entities.has(entity)
    }

    pub fn iter_entities(&self) -> impl Iterator<Item = &Vec<Entity>> {
        self.entities.iter()
    }

    pub fn recalculate_flags_at(&mut self, x: usize, y: usize, world: &World) {
        let mut combined_flags = ColliderFlags::empty();

        if let Some(entities) = self.entities.get(x, y) {
            for &entity in entities {
                if let Some(collider) = world.get::<crate::domain::Collider>(entity) {
                    combined_flags |= collider.flags;
                }
            }
        }

        if let Some(flags) = self.flags.get_mut(x, y) {
            *flags = combined_flags;
        }
    }

    pub fn recalculate_flags_at_with_query(
        &mut self,
        x: usize,
        y: usize,
        q_colliders: &Query<&crate::domain::Collider>,
    ) {
        let mut combined_flags = ColliderFlags::empty();

        if let Some(entities) = self.entities.get(x, y) {
            for &entity in entities {
                if let Ok(collider) = q_colliders.get(entity) {
                    combined_flags |= collider.flags;
                }
            }
        }

        if let Some(flags) = self.flags.get_mut(x, y) {
            *flags = combined_flags;
        }
    }
}
