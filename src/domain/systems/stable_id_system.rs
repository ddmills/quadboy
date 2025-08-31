use bevy_ecs::prelude::*;

use crate::{
    domain::Inventory,
    engine::{StableId, StableIdRegistry},
};

pub fn reconcile_inventory_ids(
    mut q_inventory: Query<&mut Inventory>,
    registry: Res<StableIdRegistry>,
) {
    for mut inventory in q_inventory.iter_mut() {
        inventory.items.clear();

        let item_ids = inventory.item_ids.clone();
        for item_id in item_ids {
            if let Some(entity) = registry.get_entity(item_id) {
                inventory.items.push(entity);
            }
        }
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
