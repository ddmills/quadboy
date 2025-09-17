use bevy_ecs::prelude::*;
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};

use crate::domain::{Collider, InActiveZone, Zone, ZoneStatus};
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

pub fn update_entity_pos(
    mut cmds: Commands,
    mut q_moved: Query<
        (Entity, &mut Position, Option<&Collider>),
        (
            Or<(Changed<Position>, Added<Position>)>,
            With<RecordZonePosition>,
        ),
    >,
    mut q_zones: Query<(Entity, &mut Zone, &ZoneStatus)>,
) {
    tracy_span!("update_entity_pos");
    for (e, mut pos, opt_collider) in q_moved.iter_mut() {
        let new_zone_idx = pos.zone_idx();
        let old_zone_idx = pos.prev_zone_idx;

        if new_zone_idx != old_zone_idx
            && let Some((_, mut old_zone, _)) =
                q_zones.iter_mut().find(|(_, x, _)| x.idx == old_zone_idx)
        {
            old_zone.entities.remove(&e);
            old_zone.colliders.remove(&e);
        }

        if let Some((zone_e, mut zone, zone_status)) =
            q_zones.iter_mut().find(|(_, x, _)| x.idx == new_zone_idx)
        {
            let (local_x, local_y) = world_to_zone_local(pos.x as usize, pos.y as usize);

            zone.entities.remove(&e);
            zone.entities.insert(local_x, local_y, e);

            zone.colliders.remove(&e);
            if opt_collider.is_some() {
                zone.colliders.insert(local_x, local_y, e);
            }

            pos.prev_zone_idx = new_zone_idx;
            cmds.entity(e).insert(*zone_status).insert(ChildOf(zone_e));

            if *zone_status == ZoneStatus::Active {
                cmds.entity(e).try_insert(InActiveZone);
            } else {
                cmds.entity(e).remove::<InActiveZone>();
            }
        }
    }
}
