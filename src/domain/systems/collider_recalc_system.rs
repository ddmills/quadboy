use bevy_ecs::prelude::*;
use quadboy_macros::profiled_system;

use crate::{
    domain::{Collider, RecalculateColliderFlagsEvent, Zone},
};

#[profiled_system]
pub fn recalculate_collider_flags_system(
    mut events: EventReader<RecalculateColliderFlagsEvent>,
    mut q_zones: Query<&mut Zone>,
    q_colliders: Query<&Collider>,
) {

    for event in events.read() {
        if let Some(mut zone) = q_zones.iter_mut().find(|z| z.idx == event.zone_idx) {
            zone.colliders
                .recalculate_flags_at_with_query(event.x, event.y, &q_colliders);
        }
    }
}
