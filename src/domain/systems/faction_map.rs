use bevy_ecs::prelude::*;
use std::collections::HashMap;

use crate::{
    cfg::ZONE_SIZE,
    common::algorithm::dijkstra::DijkstraMap,
    domain::{Collider, FactionId, FactionMember, InActiveZone, Zone},
    engine::Clock,
    rendering::Position,
};

#[derive(Resource)]
pub struct FactionMap {
    maps: HashMap<FactionId, DijkstraMap>,
    current_zone: Option<usize>,
}

impl FactionMap {
    pub fn new() -> Self {
        Self {
            maps: HashMap::new(),
            current_zone: None,
        }
    }

    pub fn get_map(&self, faction_id: FactionId) -> Option<&DijkstraMap> {
        self.maps.get(&faction_id)
    }

    fn update_obstacles(
        &mut self,
        zone: &Zone,
        q_colliders: &Query<(Entity, &Position), (With<Collider>, With<InActiveZone>)>,
    ) {
        for map in self.maps.values_mut() {
            for x in 0..ZONE_SIZE.0 {
                for y in 0..ZONE_SIZE.1 {
                    map.set_passable(x, y);
                }
            }

            for (_entity, position) in q_colliders.iter() {
                let zone_idx = position.zone_idx();

                if zone_idx == zone.idx {
                    let local_pos = position.zone_local();
                    map.set_blocked(local_pos.0, local_pos.1);
                }
            }
        }
    }

    pub fn recalculate(
        &mut self,
        zone: &Zone,
        q_colliders: &Query<(Entity, &Position), (With<Collider>, With<InActiveZone>)>,
        q_faction_members: &Query<(&Position, &FactionMember), With<InActiveZone>>,
    ) {
        let mut faction_goals: HashMap<FactionId, Vec<(usize, usize)>> = HashMap::new();

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

        for faction_id in faction_goals.keys() {
            if !self.maps.contains_key(faction_id) {
                self.maps
                    .insert(*faction_id, DijkstraMap::new(ZONE_SIZE.0, ZONE_SIZE.1));
            }
        }

        self.maps
            .retain(|faction_id, _| faction_goals.contains_key(faction_id));

        self.update_obstacles(zone, q_colliders);

        for (faction_id, goals) in faction_goals {
            if let Some(map) = self.maps.get_mut(&faction_id) {
                map.calculate_uniform(&goals);
            }
        }
    }

    pub fn set_current_zone(&mut self, zone_idx: usize) {
        if self.current_zone != Some(zone_idx) {
            self.current_zone = Some(zone_idx);
            self.maps.clear();
        }
    }
}

pub fn update_faction_maps(
    mut faction_map: ResMut<FactionMap>,
    clock: Res<Clock>,
    q_zones: Query<&Zone>,
    q_colliders: Query<(Entity, &Position), (With<Collider>, With<InActiveZone>)>,
    q_faction_members: Query<(&Position, &FactionMember), With<InActiveZone>>,
) {
    if clock.is_frozen() {
        return;
    }

    if let Some(zone) = q_zones.iter().next() {
        faction_map.set_current_zone(zone.idx);
        faction_map.recalculate(zone, &q_colliders, &q_faction_members);
    }
}
