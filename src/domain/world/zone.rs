use bevy_ecs::prelude::*;

use crate::{cfg::{MAP_SIZE, ZONE_SIZE}, common::{Grid, Palette}, domain::{PlayerMovedEvent, Zone, Zones}, rendering::{world_to_zone_idx, zone_idx, zone_local_to_world, zone_xyz, Glyph, Position, RenderLayer}};

#[derive(Component, PartialEq, Eq, Clone, Copy)]
pub enum ZoneStatus {
    Active,
    Dormant,
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

pub fn on_load_zone(mut cmds: Commands, mut e_load_zone: EventReader<LoadZoneEvent>)
{
    for LoadZoneEvent(zone_idx) in e_load_zone.read() {
        let zone_e = cmds.spawn(ZoneStatus::Dormant).id();

        let tiles = Grid::init_fill(ZONE_SIZE.0, ZONE_SIZE.1, |x, y| {
            let wpos = zone_local_to_world(*zone_idx, x, y);

            cmds.spawn((
                Position::new(wpos.0, wpos.1),
                Glyph::new(x + y, Palette::Brown, Palette::Green)
                    .layer(RenderLayer::Ground),
                ChildOf(zone_e),
                ZoneStatus::Dormant,
            )).id()
        });

        cmds.entity(zone_e).insert(Zone::new(*zone_idx, tiles));
    }
}

pub fn on_unload_zone(
    mut cmds: Commands,
    mut e_unload_zone: EventReader<UnloadZoneEvent>,
    q_zones: Query<(Entity, &Zone)>
)
{
    for UnloadZoneEvent(zone_idx) in e_unload_zone.read()
    {
        let Some((zone_e, zone)) = q_zones.iter().find(|(_, c)| c.idx == *zone_idx) else {
            continue;
        };

        // save_zone(&zone.to_save());

        cmds.entity(zone_e).despawn();
    }
}

pub fn on_set_zone_status(
    mut e_set_zone_status: EventReader<SetZoneStatusEvent>,
    mut cmds: Commands,
    q_zones: Query<(Entity, &Zone)>,
) {
    for e in e_set_zone_status.read() {
        let Some((zone_e, zone)) = q_zones.iter().find(|(_, c)| c.idx == e.idx) else {
            continue;
        };

        cmds.entity(zone_e).insert(e.status);

        for tile in zone.tiles.iter() {
            cmds.entity(*tile).insert(e.status);
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
