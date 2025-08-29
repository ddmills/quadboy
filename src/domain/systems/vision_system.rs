use std::collections::HashMap;

use crate::{
    cfg::SURFACE_LEVEL_Z,
    common::algorithm::shadowcast::{ShadowcastSettings, shadowcast},
    domain::{
        ApplyVisibilityEffects, BitmaskGlyph, InActiveZone, IsExplored, IsVisible, Player,
        PlayerPosition, RefreshBitmask, Vision, VisionBlocker, Zone, Zones,
    },
    engine::Clock,
    rendering::{Position, world_to_zone_idx, world_to_zone_local},
};
use bevy_ecs::prelude::*;
use macroquad::telemetry;

pub fn update_player_vision(
    q_player: Query<&Vision, With<Player>>,
    player_pos: Res<PlayerPosition>,
    mut q_zones: Query<&mut Zone>,
    q_vision_blockers: Query<&Position, (With<VisionBlocker>, With<InActiveZone>)>,
    clock: Res<Clock>,
    zones: Res<Zones>,
) {
    telemetry::begin_zone("update_player_vision");

    if clock.is_frozen() {
        telemetry::end_zone();
        return;
    }

    let Ok(vision) = q_player.single() else {
        telemetry::end_zone();
        return;
    };

    let player_world_pos = player_pos.world();
    let player_zone_idx =
        world_to_zone_idx(player_world_pos.0, player_world_pos.1, player_world_pos.2);

    let Some(zone_entity) = zones.cache.get(&player_zone_idx) else {
        telemetry::end_zone();
        return;
    };

    let Ok(mut zone) = q_zones.get_mut(*zone_entity) else {
        telemetry::end_zone();
        return;
    };

    zone.visible.clear(false);
    let mut vis = vec![];

    let mut blocker_cache: HashMap<(i32, i32, i32), bool> = HashMap::new();
    for blocker_pos in q_vision_blockers.iter() {
        let world_pos = blocker_pos.world();
        blocker_cache.insert(
            (world_pos.0 as i32, world_pos.1 as i32, world_pos.2 as i32),
            true,
        );
    }

    let player_x = player_world_pos.0 as i32;
    let player_y = player_world_pos.1 as i32;
    let player_z = player_world_pos.2 as i32;

    let is_underground = player_world_pos.2 > SURFACE_LEVEL_Z;
    let vision_range = if is_underground {
        vision.underground_range
    } else {
        vision.range
    };

    let settings = ShadowcastSettings {
        start_x: player_x,
        start_y: player_y,
        distance: vision_range as i32,
        is_blocker: |x: i32, y: i32| blocker_cache.contains_key(&(x, y, player_z)),
        on_light: |x: i32, y: i32, _distance: f64| {
            if x < 0 || y < 0 {
                return;
            }

            let world_x = x as usize;
            let world_y = y as usize;
            let world_z = player_z as usize;
            let zone_idx = world_to_zone_idx(world_x, world_y, world_z);

            if zone_idx == player_zone_idx {
                vis.push((x as usize, y as usize));
            }
        },
    };

    shadowcast(settings);

    for (world_x, world_y) in vis {
        let (local_x, local_y) = world_to_zone_local(world_x, world_y);
        zone.visible.set(local_x, local_y, true);
        zone.explored.set(local_x, local_y, true);
    }

    telemetry::end_zone();
}

pub fn update_entity_visibility_flags(
    mut cmds: Commands,
    q_zones: Query<&Zone>,
    mut q_entities: Query<
        (
            Entity,
            &Position,
            Option<&IsVisible>,
            Option<&IsExplored>,
            Option<&BitmaskGlyph>,
        ),
        (With<ApplyVisibilityEffects>, With<InActiveZone>),
    >,
    clock: Res<Clock>,
    zones: Res<Zones>,
    mut e_refresh_bitmask: EventWriter<RefreshBitmask>,
) {
    if clock.is_frozen() {
        return;
    }

    telemetry::begin_zone("update_entity_visibility_flags");

    for (entity, position, has_visible, has_explored, has_bitmask) in q_entities.iter_mut() {
        let world_pos = position.world();
        let zone_idx = world_to_zone_idx(world_pos.0, world_pos.1, world_pos.2);

        let Some(zone_entity) = zones.cache.get(&zone_idx) else {
            continue;
        };

        let Ok(zone) = q_zones.get(*zone_entity) else {
            continue;
        };

        let (local_x, local_y) = world_to_zone_local(world_pos.0, world_pos.1);

        let is_visible = zone.visible.get(local_x, local_y).copied().unwrap_or(false);

        let is_explored = zone
            .explored
            .get(local_x, local_y)
            .copied()
            .unwrap_or(false);

        match (is_visible, has_visible.is_some()) {
            (true, false) => {
                cmds.entity(entity).insert(IsVisible);

                if has_bitmask.is_some() {
                    e_refresh_bitmask.write(RefreshBitmask(entity));

                    let neighbors = Zone::get_neighbors(world_pos, &q_zones);
                    for neighbor in neighbors.iter().flatten() {
                        e_refresh_bitmask.write(RefreshBitmask(*neighbor));
                    }
                }
            }
            (false, true) => {
                cmds.entity(entity).remove::<IsVisible>();
            }
            _ => {}
        }

        if let (true, false) = (is_explored, has_explored.is_some()) {
            cmds.entity(entity).insert(IsExplored);
        }
    }
    telemetry::end_zone();
}
