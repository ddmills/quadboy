use bevy_ecs::prelude::*;
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};

use crate::domain::{Collider, InActiveZone, RecalculateColliderFlagsEvent, Zone, ZoneStatus};
use crate::rendering::{world_to_zone_idx, world_to_zone_local};

use crate::engine::SerializableComponent;
use crate::tracy_span;

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent, Debug)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    #[serde(skip)]
    prev_zone_idx: usize,
}

impl Position {
    pub fn new_world(world_pos: (usize, usize, usize)) -> Self {
        Self {
            x: world_pos.0 as f32,
            y: world_pos.1 as f32,
            z: world_pos.2 as f32,
            prev_zone_idx: 9999999,
        }
    }

    pub fn new(x: usize, y: usize, z: usize) -> Self {
        Self {
            x: x as f32,
            y: y as f32,
            z: z as f32,
            prev_zone_idx: 9999999,
        }
    }

    pub fn new_f32(x: f32, y: f32, z: f32) -> Self {
        Self {
            x,
            y,
            z,
            prev_zone_idx: 9999999,
        }
    }

    #[inline]
    pub fn zone_idx(&self) -> usize {
        world_to_zone_idx(self.x as usize, self.y as usize, self.z as usize)
    }

    #[inline]
    pub fn world(&self) -> (usize, usize, usize) {
        (self.x as usize, self.y as usize, self.z as usize)
    }

    #[inline]
    pub fn zone_local(&self) -> (usize, usize) {
        world_to_zone_local(self.x as usize, self.y as usize)
    }
}

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct RecordZonePosition;

pub fn update_entity_pos(world: &mut World) {
    tracy_span!("update_entity_pos");

    // Collect entities that need position updates
    let entities_to_update: Vec<(Entity, Position, Option<crate::domain::ColliderFlags>)> = {
        let mut q_moved = world
            .query_filtered::<(Entity, &Position, Option<&crate::domain::Collider>), (
                Or<(Changed<Position>, Added<Position>)>,
                With<RecordZonePosition>,
            )>();

        q_moved
            .iter(world)
            .map(|(e, pos, opt_collider)| {
                let collider_flags = opt_collider.map(|c| c.flags);
                (e, pos.clone(), collider_flags)
            })
            .collect()
    };

    // Collect events to send after zone updates
    let mut recalc_events = Vec::new();

    // Process each entity
    for (e, mut pos, opt_collider_flags) in entities_to_update {
        let new_zone_idx = pos.zone_idx();
        let old_zone_idx = pos.prev_zone_idx;

        // Remove from old zone if moving between zones
        if new_zone_idx != old_zone_idx {
            let mut q_zones = world.query::<&mut Zone>();
            if let Some(mut old_zone) = q_zones.iter_mut(world).find(|z| z.idx == old_zone_idx) {
                let _ = old_zone.entities.remove(&e);
                // Note: We use regular remove here and rely on the new zone insertion
                // to properly update the cache. Inter-zone moves are less critical
                // for the ghost flag issue since we're completely leaving the zone.
                let _ = old_zone.colliders.remove(&e);
            }
        }

        // Add to new zone
        {
            let mut zone_data: Option<(Entity, ZoneStatus)> = None;
            {
                let mut q_zones = world.query::<(Entity, &mut Zone, &ZoneStatus)>();
                if let Some((zone_e, mut zone, zone_status)) = q_zones
                    .iter_mut(world)
                    .find(|(_, z, _)| z.idx == new_zone_idx)
                {
                    let (local_x, local_y) = world_to_zone_local(pos.x as usize, pos.y as usize);

                    let _ = zone.entities.remove(&e);
                    zone.entities.insert(local_x, local_y, e);

                    // Remove from collider cache and get old position for intra-zone moves
                    let old_collider_pos = zone.colliders.remove(&e);

                    // If moving within the same zone, fire event for old position
                    if new_zone_idx == old_zone_idx {
                        if let Some((old_x, old_y)) = old_collider_pos {
                            recalc_events.push(crate::domain::RecalculateColliderFlagsEvent {
                                zone_idx: new_zone_idx,
                                x: old_x,
                                y: old_y,
                            });
                        }
                    }

                    // If we have collision flags, insert at new position
                    if let Some(collider_flags) = opt_collider_flags {
                        zone.colliders.insert(local_x, local_y, e, collider_flags);
                    }

                    // Queue recalculation event for new position
                    recalc_events.push(crate::domain::RecalculateColliderFlagsEvent {
                        zone_idx: new_zone_idx,
                        x: local_x,
                        y: local_y,
                    });

                    zone_data = Some((zone_e, *zone_status));
                }
            }

            // Update entity components
            if let Some((zone_e, zone_status)) = zone_data {
                pos.prev_zone_idx = new_zone_idx;

                world
                    .entity_mut(e)
                    .insert(pos)
                    .insert(zone_status)
                    .insert(ChildOf(zone_e));

                if zone_status == ZoneStatus::Active {
                    world.entity_mut(e).insert(InActiveZone);
                } else {
                    world.entity_mut(e).remove::<InActiveZone>();
                }
            }
        }
    }

    // Send all queued recalculation events
    for event in recalc_events {
        world.send_event(event);
    }
}
