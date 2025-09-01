use bevy_ecs::prelude::*;

use crate::engine::{StableId, StableIdRegistry};

pub fn cleanup_despawned_stable_ids(
    mut registry: ResMut<StableIdRegistry>,
    mut removed: RemovedComponents<StableId>,
) {
    for entity in removed.read() {
        registry.unregister(entity);
    }
}
