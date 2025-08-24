use std::collections::hash_map;

use bevy_ecs::prelude::*;
use macroquad::{prelude::trace, telemetry};
use serde::{Deserialize, Serialize};

use crate::{
    cfg::{MAP_SIZE, RENDER_DORMANT, ZONE_SIZE},
    common::{Grid, HashGrid, Rand},
    domain::{
        LoadZoneCommand, PlayerMovedEvent, PrefabId, Prefabs, SpawnConfig, Terrain,
        UnloadZoneCommand, Zone, ZoneGenerator, Zones,
    },
    engine::{SerializedEntity, deserialize_all},
    rendering::{world_to_zone_idx, zone_idx, zone_local_to_world, zone_xyz},
    states::CleanupStatePlay,
};

#[derive(Component, PartialEq, Eq, Clone, Copy)]
pub enum ZoneStatus {
    Active,
    Dormant,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct ZoneSaveData {
    pub idx: usize,
    pub terrain: Grid<Terrain>,
    pub entities: Vec<SerializedEntity>,
    pub explored: Grid<bool>,
}

#[derive(Event)]
pub struct LoadZoneEvent(pub usize);

#[derive(Event)]
pub struct UnloadZoneEvent(pub usize);

#[derive(Event)]
pub struct SetZoneStatusEvent {
    pub idx: usize,
    pub status: ZoneStatus,
}

pub fn on_load_zone(mut cmds: Commands, mut e_load_zone: EventReader<LoadZoneEvent>) {
    for LoadZoneEvent(zone_idx) in e_load_zone.read() {
        cmds.queue(LoadZoneCommand(*zone_idx));
    }
}

pub fn on_unload_zone(mut cmds: Commands, mut e_unload_zone: EventReader<UnloadZoneEvent>) {
    for UnloadZoneEvent(zone_idx) in e_unload_zone.read() {
        cmds.queue(UnloadZoneCommand {
            zone_idx: *zone_idx,
            despawn: true,
        });
    }
}

pub fn on_set_zone_status(
    mut e_set_zone_status: EventReader<SetZoneStatusEvent>,
    mut cmds: Commands,
    q_zones: Query<(Entity, &Zone, &Children)>,
    q_terrain: Query<Entity, With<Terrain>>,
) {
    for evt in e_set_zone_status.read() {
        let Some((zone_e, zone, children)) = q_zones.iter().find(|(_, z, _)| z.idx == evt.idx)
        else {
            continue;
        };

        cmds.entity(zone_e).insert(evt.status);

        for child in children.iter() {
            if evt.status == ZoneStatus::Dormant {
                if q_terrain.contains(child) {
                    cmds.entity(child).despawn();
                }
            } else {
                cmds.entity(child).insert(evt.status);
            }
        }

        if evt.status == ZoneStatus::Active {
            for (x, y, t) in zone.terrain.iter_xy() {
                let wpos = zone_local_to_world(zone.idx, x, y);
                let config = SpawnConfig::new(PrefabId::TerrainTile(*t), wpos);
                let terrain_entity = Prefabs::spawn(&mut cmds, config);

                cmds.entity(terrain_entity)
                    .insert(ChildOf(zone_e))
                    .insert(evt.status);
            }
        }

        for v in zone.entities.iter() {
            for e in v.iter() {
                cmds.entity(*e).try_insert(evt.status);
            }
        }
    }
}

// check when player moves to a different zone and set it as active
pub fn activate_zones_by_player(
    mut e_player_moved: EventReader<PlayerMovedEvent>,
    mut zones: ResMut<Zones>,
) {
    for e in e_player_moved.read() {
        let player_zone_idx = world_to_zone_idx(e.x, e.y, e.z);

        zones.player = player_zone_idx;

        if !zones.active.contains(&player_zone_idx) {
            zones.active = vec![player_zone_idx];
        }
    }
}

// determine which zones should
//  - be loaded
//  - be unloaded
//  - change status
pub fn load_nearby_zones(
    zones: Res<Zones>,
    mut e_load_zone: EventWriter<LoadZoneEvent>,
    mut e_unload_zone: EventWriter<UnloadZoneEvent>,
    mut e_set_zone_status: EventWriter<SetZoneStatusEvent>,
    q_zones: Query<(&Zone, &ZoneStatus)>,
) {
    let mut cur_dormant_zones = q_zones
        .iter()
        .filter_map(|(c, s)| match s {
            ZoneStatus::Active => None,
            ZoneStatus::Dormant => Some(c.idx),
        })
        .collect::<Vec<_>>();

    let mut cur_active_zones = q_zones
        .iter()
        .filter_map(|(c, s)| match s {
            ZoneStatus::Active => Some(c.idx),
            ZoneStatus::Dormant => None,
        })
        .collect::<Vec<_>>();

    let mut needed_zones = zones.active.clone();

    for idx in zones.active.iter() {
        let (x, y, z) = zone_xyz(*idx);

        if y < MAP_SIZE.1 - 1 {
            let north_idx = zone_idx(x, y + 1, z);
            needed_zones.push(north_idx);

            if x < MAP_SIZE.0 - 1 {
                let north_east_idx = zone_idx(x + 1, y + 1, z);
                needed_zones.push(north_east_idx);
            }

            if x > 0 {
                let north_west_idx = zone_idx(x - 1, y + 1, z);
                needed_zones.push(north_west_idx);
            }
        }

        if y > 0 {
            let south_idx = zone_idx(x, y - 1, z);
            needed_zones.push(south_idx);

            if x < MAP_SIZE.0 - 1 {
                let south_east_idx = zone_idx(x + 1, y - 1, z);
                needed_zones.push(south_east_idx);
            }

            if x > 0 {
                let south_west_idx = zone_idx(x - 1, y - 1, z);
                needed_zones.push(south_west_idx);
            }
        }

        if z > 0 {
            let above_idx = zone_idx(x, y, z - 1);
            needed_zones.push(above_idx);
        }

        if x < MAP_SIZE.0 - 1 {
            let east_idx = zone_idx(x + 1, y, z);
            needed_zones.push(east_idx);
        }

        if x > 0 {
            let west_idx = zone_idx(x - 1, y, z);
            needed_zones.push(west_idx);
        }

        if z < MAP_SIZE.2 - 1 {
            let below_idx = zone_idx(x, y, z + 1);
            needed_zones.push(below_idx);
        }
    }

    let mut zones_to_load = vec![];
    let mut zones_to_dormant = vec![];
    let mut zones_to_active = vec![];

    needed_zones.sort();
    needed_zones.dedup();

    for idx in needed_zones.iter() {
        let is_active = zones.active.contains(idx);

        if let Some(cur_pos) = cur_active_zones.iter().position(|&i| i == *idx) {
            cur_active_zones.swap_remove(cur_pos);

            // zone is active, but needs to be dormant.
            if !is_active {
                zones_to_dormant.push(*idx);
            }
        } else if let Some(cur_pos) = cur_dormant_zones.iter().position(|&i| i == *idx) {
            cur_dormant_zones.swap_remove(cur_pos);

            // zone is dormant but needs to be active
            if is_active {
                zones_to_active.push(*idx);
            }
        } else {
            // zone is not dormant or active, but needed. We must load it
            zones_to_load.push(*idx);

            // also needs to be active
            if is_active {
                zones_to_active.push(*idx);
            }
        }
    }

    let zones_to_unload = [cur_active_zones, cur_dormant_zones].concat();

    if let Some(idx) = zones_to_load.first() {
        e_load_zone.write(LoadZoneEvent(*idx));
    }

    if let Some(idx) = zones_to_unload.first() {
        e_unload_zone.write(UnloadZoneEvent(*idx));
    }

    for idx in zones_to_active.iter() {
        e_set_zone_status.write(SetZoneStatusEvent {
            idx: *idx,
            status: ZoneStatus::Active,
        });
    }

    for idx in zones_to_dormant.iter() {
        e_set_zone_status.write(SetZoneStatusEvent {
            idx: *idx,
            status: ZoneStatus::Dormant,
        });
    }
}

pub fn spawn_zone(world: &mut World, zone_idx: usize) {
    telemetry::begin_zone("spawn_zone");
    telemetry::begin_zone("generate_zone");

    let data = ZoneGenerator::generate_zone(world, zone_idx);

    telemetry::end_zone();

    let zone_entity_id = world
        .spawn((
            Zone::new(zone_idx, data.terrain.clone()),
            ZoneStatus::Dormant,
            CleanupStatePlay,
        ))
        .id();

    spawn_terrain(world, zone_idx, zone_entity_id, data.terrain);

    for config in data.entities.iter().flatten() {
        // todo: Remove clone
        Prefabs::spawn_world(world, config.clone());
    }

    telemetry::end_zone();
}

pub fn spawn_zone_load(world: &mut World, zone_data: ZoneSaveData) {
    let zone_entity_id = world
        .spawn((
            ZoneStatus::Dormant,
            CleanupStatePlay,
            Zone {
                idx: zone_data.idx,
                terrain: zone_data.terrain.clone(),
                entities: HashGrid::init(ZONE_SIZE.0, ZONE_SIZE.1),
                visible: Grid::init(ZONE_SIZE.0, ZONE_SIZE.1, false),
                explored: zone_data.explored,
            },
        ))
        .id();

    spawn_terrain(world, zone_data.idx, zone_entity_id, zone_data.terrain);

    deserialize_all(&zone_data.entities, world);
}

pub fn manage_zone_cache(
    mut zones: ResMut<Zones>,
    q_zones_added: Query<(Entity, &Zone), Added<Zone>>,
    mut removed_zones: RemovedComponents<Zone>,
) {
    for (entity, zone) in q_zones_added.iter() {
        zones.cache.insert(zone.idx, entity);
    }

    for entity in removed_zones.read() {
        zones
            .cache
            .retain(|_, &mut cached_entity| cached_entity != entity);
    }
}

fn spawn_terrain(
    world: &mut World,
    zone_idx: usize,
    zone_entity_id: Entity,
    terrain: Grid<Terrain>,
) {
    let zones = world.get_resource::<Zones>().unwrap();
    let is_active = zones.active.contains(&zone_idx);

    if is_active || RENDER_DORMANT {
        let z_status = if is_active {
            ZoneStatus::Active
        } else {
            ZoneStatus::Dormant
        };

        for (x, y, t) in terrain.iter_xy() {
            let wpos = zone_local_to_world(zone_idx, x, y);
            let config = SpawnConfig::new(PrefabId::TerrainTile(*t), wpos);
            let terrain_entity = Prefabs::spawn_world(world, config);

            world
                .entity_mut(terrain_entity)
                .insert(z_status)
                .insert(ChildOf(zone_entity_id));
        }
    }
}
