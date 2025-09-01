use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::engine::{SerializableComponent, StableId, StableIdRegistry};

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct NeedsStableId;

pub fn auto_assign_stable_ids(
    mut cmds: Commands,
    mut registry: ResMut<StableIdRegistry>,
    q_needs_id: Query<Entity, (With<NeedsStableId>, Without<StableId>)>,
) {
    for entity in q_needs_id.iter() {
        let id = registry.generate_id();
        cmds.entity(entity)
            .insert(StableId::new(id))
            .remove::<NeedsStableId>();
    }
}

pub fn register_new_stable_ids(
    mut registry: ResMut<StableIdRegistry>,
    q_new_ids: Query<(Entity, &StableId), Added<StableId>>,
) {
    for (entity, stable_id) in q_new_ids.iter() {
        registry.register(entity, stable_id.0);
    }
}

pub fn cleanup_despawned_stable_ids(
    mut registry: ResMut<StableIdRegistry>,
    mut removed: RemovedComponents<StableId>,
) {
    for entity in removed.read() {
        registry.unregister(entity);
    }
}
