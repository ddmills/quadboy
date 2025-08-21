use bevy_ecs::prelude::*;
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};

use crate::domain::{Zone, ZoneStatus};
use crate::rendering::{world_to_zone_idx, world_to_zone_local};

use crate::engine::SerializableComponent;

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    #[serde(skip)]
    prev_zone_idx: usize,
}

impl Position {
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
}

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct TrackZone;

pub fn update_entity_pos(
    mut cmds: Commands,
    mut q_moved: Query<(Entity, &mut Position), (Changed<Position>, With<TrackZone>)>,
    mut q_zones: Query<(&mut Zone, &ZoneStatus)>,
) {
    for (e, mut pos) in q_moved.iter_mut() {
        let new_zone_idx = pos.zone_idx();
        let old_zone_idx = pos.prev_zone_idx;

        if new_zone_idx != old_zone_idx
            && let Some((mut old_zone, _)) = q_zones.iter_mut().find(|(x, _)| x.idx == old_zone_idx)
            {
                old_zone.entities.remove(&e);
            }

        if let Some((mut zone, zone_status)) =
            q_zones.iter_mut().find(|(x, _)| x.idx == new_zone_idx)
        {
            let (local_x, local_y) = world_to_zone_local(pos.x as usize, pos.y as usize);
            zone.entities.remove(&e);
            zone.entities.insert(local_x, local_y, e);
            pos.prev_zone_idx = new_zone_idx;
            cmds.entity(e).insert(*zone_status);
        }
    }
}
