use crate::{
    domain::{
        Collider, ColliderFlags, DynamicEntity, InActiveZone, RecalculateColliderFlagsEvent,
        StaticEntitySpawnedEvent, Zone, ZoneStatus, Zones,
    },
    rendering::{Position, world_to_zone_local},
};
use bevy_ecs::prelude::*;
use quadboy_macros::profiled_system;
// ChildOf is part of prelude now

/// Event-driven placement of static entities in zone caches
/// Only processes entities when StaticEntitySpawnedEvent is received
#[profiled_system]
pub fn place_static_entities(world: &mut World) {

    // Read events for static entities that need placement
    let events: Vec<StaticEntitySpawnedEvent> = {
        let mut event_reader = world
            .get_resource_mut::<Events<StaticEntitySpawnedEvent>>()
            .unwrap();
        event_reader.drain().collect()
    };

    if events.is_empty() {
        return;
    }

    // Get zones cache for O(1) lookup
    let zones_cache = world
        .get_resource::<Zones>()
        .map(|zones| zones.cache.clone())
        .unwrap_or_default();

    // Collect events to send after zone updates
    let mut recalc_events = Vec::new();

    // Process each static entity event
    for event in events {
        let mut pos = event.position;
        let zone_idx = pos.zone_idx();

        if let Some(&zone_entity) = zones_cache.get(&zone_idx) {
            // Get zone status first to avoid borrowing conflicts
            let zone_status = world.entity(zone_entity).get::<ZoneStatus>().copied();

            if let (Some(mut zone), Some(zone_status)) =
                (world.entity_mut(zone_entity).get_mut::<Zone>(), zone_status)
            {
                let (local_x, local_y) = world_to_zone_local(pos.x as usize, pos.y as usize);

                // Add to zone entities grid
                zone.entities.insert(local_x, local_y, event.entity);

                // Add to collider cache if has collider
                if let Some(collider_flags) = event.collider_flags {
                    zone.colliders
                        .insert(local_x, local_y, event.entity, collider_flags);

                    // Fire recalculation event for colliders
                    recalc_events.push(RecalculateColliderFlagsEvent {
                        zone_idx,
                        x: local_x,
                        y: local_y,
                    });
                }

                // Update entity components - critical for visibility!
                pos.set_prev_zone_idx(zone_idx);

                let mut entity_mut = world.entity_mut(event.entity);
                entity_mut.insert(pos);
                entity_mut.insert(zone_status);
                entity_mut.insert(ChildOf(zone_entity));

                if zone_status == ZoneStatus::Active {
                    entity_mut.insert(InActiveZone);
                } else {
                    entity_mut.remove::<InActiveZone>();
                }
            }
        }
    }

    // Send all queued recalculation events
    for event in recalc_events {
        world.send_event(event);
    }
}

/// Handle movement of dynamic entities only
/// This should process far fewer entities than the original system
#[profiled_system]
pub fn update_dynamic_entity_pos(world: &mut World) {

    // Collect only dynamic entities that have moved or were just added
    let entities_to_update: Vec<(Entity, Position, Option<ColliderFlags>)> = {
        let mut q_moved = world.query_filtered::<(Entity, &Position, Option<&Collider>), (
            Or<(Changed<Position>, Added<Position>)>,
            With<DynamicEntity>,
        )>();

        q_moved
            .iter(world)
            .map(|(e, pos, opt_collider)| {
                let collider_flags = opt_collider.map(|c| c.flags);
                (e, pos.clone(), collider_flags)
            })
            .collect()
    };

    if entities_to_update.is_empty() {
        return;
    }

    // Get zones cache for O(1) lookup
    let zones_cache = world
        .get_resource::<Zones>()
        .map(|zones| zones.cache.clone())
        .unwrap_or_default();

    // Collect events to send after zone updates
    let mut recalc_events = Vec::new();

    // Process each dynamic entity
    for (entity, mut pos, opt_collider_flags) in entities_to_update {
        let new_zone_idx = pos.zone_idx();
        let old_zone_idx = pos.prev_zone_idx();

        // Remove from old zone if moving between zones
        if new_zone_idx != old_zone_idx
            && let Some(&old_zone_entity) = zones_cache.get(&old_zone_idx)
            && let Some(mut old_zone) = world.entity_mut(old_zone_entity).get_mut::<Zone>()
        {
            let _ = old_zone.entities.remove(&entity);
            let _ = old_zone.colliders.remove(&entity);
        }

        // Add to new zone
        if let Some(&zone_entity) = zones_cache.get(&new_zone_idx) {
            // Get zone status first to avoid borrowing conflicts
            let zone_status = world.entity(zone_entity).get::<ZoneStatus>().copied();

            if let (Some(mut zone), Some(zone_status)) =
                (world.entity_mut(zone_entity).get_mut::<Zone>(), zone_status)
            {
                let (local_x, local_y) = world_to_zone_local(pos.x as usize, pos.y as usize);

                // Remove from current position and get old position for intra-zone moves
                let _ = zone.entities.remove(&entity);
                let old_collider_pos = zone.colliders.remove(&entity);

                // Add at new position
                zone.entities.insert(local_x, local_y, entity);

                // If moving within the same zone, fire event for old position
                if new_zone_idx == old_zone_idx
                    && let Some((old_x, old_y)) = old_collider_pos
                {
                    recalc_events.push(RecalculateColliderFlagsEvent {
                        zone_idx: new_zone_idx,
                        x: old_x,
                        y: old_y,
                    });
                }

                // If we have collision flags, insert at new position
                if let Some(collider_flags) = opt_collider_flags {
                    zone.colliders
                        .insert(local_x, local_y, entity, collider_flags);
                }

                // Queue recalculation event for new position
                recalc_events.push(RecalculateColliderFlagsEvent {
                    zone_idx: new_zone_idx,
                    x: local_x,
                    y: local_y,
                });

                // Update entity components
                pos.set_prev_zone_idx(new_zone_idx);

                let mut entity_mut = world.entity_mut(entity);
                entity_mut.insert(pos);
                entity_mut.insert(zone_status);
                entity_mut.insert(ChildOf(zone_entity));

                if zone_status == ZoneStatus::Active {
                    entity_mut.insert(InActiveZone);
                } else {
                    entity_mut.remove::<InActiveZone>();
                }
            }
        }
    }

    // Send all queued recalculation events
    for event in recalc_events {
        world.send_event(event);
    }
}
