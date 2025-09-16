use bevy_ecs::prelude::*;
use std::collections::HashMap;

use crate::{
    cfg::ZONE_SIZE,
    common::algorithm::dijkstra::DijkstraMap,
    domain::{Collider, FactionId, FactionMember, InActiveZone, PlayerPosition, Zone},
    engine::Clock,
    rendering::Position,
};

#[derive(Resource)]
pub struct FactionMap {
    maps: HashMap<FactionId, DijkstraMap>,
}

impl FactionMap {
    pub fn new() -> Self {
        Self {
            maps: HashMap::new(),
        }
    }

    pub fn get_map(&self, faction_id: FactionId) -> Option<&DijkstraMap> {
        self.maps.get(&faction_id)
    }

    fn update_obstacles(
        &mut self,
        zone: &Zone,
        faction_ids: &[FactionId],
        q_colliders: &Query<(Entity, &Position), With<Collider>>,
    ) {
        for faction_id in faction_ids {
            if let Some(map) = self.maps.get_mut(faction_id) {
                // Clear the map first
                for x in 0..ZONE_SIZE.0 {
                    for y in 0..ZONE_SIZE.1 {
                        map.set_passable(x, y);
                    }
                }

                // Add obstacles
                for (_entity, position) in q_colliders.iter() {
                    let zone_idx = position.zone_idx();
                    if zone_idx == zone.idx {
                        let local_pos = position.zone_local();
                        map.set_blocked(local_pos.0, local_pos.1);
                    }
                }
            }
        }
    }

    pub fn recalculate(
        &mut self,
        zone: &Zone,
        q_colliders: &Query<(Entity, &Position), With<Collider>>,
        q_faction_members: &Query<(&Position, &FactionMember), With<InActiveZone>>,
    ) {
        let mut faction_goals: HashMap<FactionId, Vec<(usize, usize)>> = HashMap::new();

        // Find all faction members in this zone
        for (position, faction_member) in q_faction_members.iter() {
            let zone_idx = position.zone_idx();
            if zone_idx == zone.idx {
                let local_pos = position.zone_local();
                faction_goals
                    .entry(faction_member.faction_id)
                    .or_default()
                    .push(local_pos);
            }
        }

        // Create maps for any new factions
        for faction_id in faction_goals.keys() {
            if !self.maps.contains_key(faction_id) {
                self.maps.insert(*faction_id, DijkstraMap::new(ZONE_SIZE.0, ZONE_SIZE.1));
            }
        }

        // Update obstacles for all factions in this zone
        let faction_list: Vec<_> = faction_goals.keys().cloned().collect();
        self.update_obstacles(zone, &faction_list, q_colliders);

        // Calculate pathfinding for each faction
        for (faction_id, goals) in faction_goals {
            if let Some(map) = self.maps.get_mut(&faction_id) {
                map.calculate_uniform(&goals);
            }
        }
    }

}

pub fn update_faction_maps(
    mut faction_map: ResMut<FactionMap>,
    clock: Res<Clock>,
    player_pos: Res<PlayerPosition>,
    q_zones: Query<&Zone>,
    q_colliders: Query<(Entity, &Position), With<Collider>>,
    q_faction_members: Query<(&Position, &FactionMember), With<InActiveZone>>,
) {
    if clock.is_frozen() {
        return;
    }

    // Process only the player's current zone for faction mapping
    let player_zone_idx = player_pos.zone_idx();
    if let Some(zone) = q_zones.iter().find(|z| z.idx == player_zone_idx) {
        faction_map.recalculate(zone, &q_colliders, &q_faction_members);
    }
}
