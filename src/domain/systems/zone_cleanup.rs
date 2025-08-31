use bevy_ecs::prelude::*;

use crate::{domain::Zone, rendering::Position};

pub fn cleanup_zone_entities_on_position_removal(
    mut removed: RemovedComponents<Position>,
    mut q_zones: Query<&mut Zone>,
) {
    for entity in removed.read() {
        // Check all zones and remove the entity if found
        for mut zone in q_zones.iter_mut() {
            // HashGrid has a remove method that handles finding and removing the entity
            zone.entities.remove(&entity);
        }
    }
}
