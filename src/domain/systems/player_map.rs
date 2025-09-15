use bevy_ecs::prelude::*;
use macroquad::prelude::trace;

use crate::{
    cfg::ZONE_SIZE,
    common::algorithm::dijkstra::DijkstraMap,
    domain::{Collider, InActiveZone, PlayerMovedEvent, Zone},
    rendering::{Position, world_to_zone_idx, world_to_zone_local},
};

#[derive(Resource)]
pub struct PlayerMap {
    map: DijkstraMap,
    dirty: bool,
    current_zone: Option<usize>,
}

impl PlayerMap {
    pub fn new() -> Self {
        Self {
            map: DijkstraMap::new(ZONE_SIZE.0, ZONE_SIZE.1),
            dirty: true,
            current_zone: None,
        }
    }

    pub fn get_map(&self) -> &DijkstraMap {
        &self.map
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    fn update_obstacles(
        &mut self,
        zone: &Zone,
        q_colliders: &Query<(Entity, &Position), (With<Collider>, With<InActiveZone>)>,
    ) {
        // Clear all blocked tiles first
        for x in 0..ZONE_SIZE.0 {
            for y in 0..ZONE_SIZE.1 {
                self.map.set_passable(x, y);
            }
        }

        let mut blocked_count = 0;
        let total_colliders = q_colliders.iter().len();

        // Mark tiles with colliders as blocked
        for (_entity, position) in q_colliders.iter() {
            let zone_idx = position.zone_idx();

            // Make sure the collider is in the same zone
            if zone_idx == zone.idx {
                let local_pos = position.zone_local();

                self.map.set_blocked(local_pos.0, local_pos.1);
                blocked_count += 1;
            }
        }

        trace!(
            "PlayerMap: Found {} total colliders, {} in zone {} (blocked {} tiles)",
            total_colliders, blocked_count, zone.idx, blocked_count
        );
    }

    pub fn recalculate(
        &mut self,
        player_zone_local: (usize, usize),
        zone: &Zone,
        q_colliders: &Query<(Entity, &Position), (With<Collider>, With<InActiveZone>)>,
    ) {
        if !self.dirty {
            return;
        }

        // Update obstacles first
        self.update_obstacles(zone, q_colliders);

        // Set player position as the goal
        let goals = vec![player_zone_local];
        self.map.calculate_uniform(&goals);

        self.dirty = false;
    }
}

pub fn update_player_map(
    mut player_map: ResMut<PlayerMap>,
    mut e_player_moved: EventReader<PlayerMovedEvent>,
    q_zones: Query<&Zone>,
    q_colliders: Query<(Entity, &Position), (With<Collider>, With<InActiveZone>)>,
) {
    for event in e_player_moved.read() {
        let player_world_pos = (event.x, event.y, event.z);
        let player_zone_local = world_to_zone_local(player_world_pos.0, player_world_pos.1);

        // Find the zone containing the player
        let zone_idx = world_to_zone_idx(event.x, event.y, event.z);

        if let Some(zone) = q_zones.iter().find(|z| z.idx == zone_idx) {
            // Check if we've moved to a different zone
            let zone_changed = player_map.current_zone != Some(zone_idx);

            if zone_changed {
                player_map.current_zone = Some(zone_idx);
                player_map.mark_dirty();
            } else {
                // Same zone, but player moved - still need to recalculate
                player_map.mark_dirty();
            }

            // Recalculate the map with new player position
            player_map.recalculate(player_zone_local, zone, &q_colliders);
        }
    }
}
