use std::collections::HashMap;

use crate::{
    cfg::ZONE_SIZE,
    common::algorithm::shadowcast::{ShadowcastSettings, shadowcast},
    domain::{
        ApplyVisibilityEffects, BitmaskGlyph, InActiveZone, IsExplored, IsVisible, Player,
        PlayerPosition, RefreshBitmask, Vision, VisionBlocker, Zone, Zones,
    },
    engine::Clock,
    rendering::{LightingData, Position, world_to_zone_idx, world_to_zone_local},
    tracy_span,
};
use bevy_ecs::prelude::*;
use ordered_float::Pow;

pub fn update_player_vision(
    q_player: Query<&Vision, With<Player>>,
    player_pos: Res<PlayerPosition>,
    mut q_zones: Query<&mut Zone>,
    q_vision_blockers: Query<&Position, (With<VisionBlocker>, With<InActiveZone>)>,
    clock: ResMut<Clock>,
    zones: Res<Zones>,
    lighting_data: Res<LightingData>,
) {
    tracy_span!("update_player_vision");

    // if clock.is_frozen() {
    //     return;
    // }

    if clock.tick_delta_accum() == 0 {
        return;
    }

    let Ok(vision) = q_player.single() else {
        return;
    };

    let player_world_pos = player_pos.world();
    let player_local_pos = player_pos.zone_local();
    let player_zone_idx =
        world_to_zone_idx(player_world_pos.0, player_world_pos.1, player_world_pos.2);

    let Some(zone_entity) = zones.cache.get(&player_zone_idx) else {
        return;
    };

    let Ok(mut zone) = q_zones.get_mut(*zone_entity) else {
        return;
    };

    zone.visible.clear(false);
    let mut vis = vec![];

    let blocker_cache: HashMap<(i32, i32), bool> = {
        tracy_span!("vision_build_blocker_cache");
        let mut blocker_cache: HashMap<(i32, i32), bool> = HashMap::new();
        for blocker_pos in q_vision_blockers.iter() {
            if blocker_pos.zone_idx() == player_zone_idx {
                let local_pos = blocker_pos.zone_local();
                blocker_cache.insert((local_pos.0 as i32, local_pos.1 as i32), true);
            }
        }

        blocker_cache
    };

    let (player_x, player_y, max_vision_range, vision_range) = {
        tracy_span!("vision_calculate_settings");
        let player_x = player_local_pos.0 as i32;
        let player_y = player_local_pos.1 as i32;

        let max_vision_range = vision.range;

        let daylight = lighting_data.get_ambient_intensity().pow(3.);
        let vision_range = (daylight * max_vision_range as f32).round().max(2.0) as f64;

        (player_x, player_y, max_vision_range, vision_range)
    };

    let settings = ShadowcastSettings {
        start_x: player_x,
        start_y: player_y,
        distance: max_vision_range as i32,
        is_blocker: |x: i32, y: i32| blocker_cache.contains_key(&(x, y)),
        on_light: |x: i32, y: i32, distance: f64| {
            if x < 0 || y < 0 || x >= ZONE_SIZE.0 as i32 || y >= ZONE_SIZE.1 as i32 {
                return;
            }

            let local_x = x as usize;
            let local_y = y as usize;

            if distance > vision_range {
                let light_intensity = lighting_data
                    .get_light(local_x, local_y)
                    .map(|light| light.intensity)
                    .unwrap_or(0.0);

                if light_intensity > 0.0 {
                    vis.push((x as usize, y as usize));
                }
            } else {
                vis.push((x as usize, y as usize));
            }
        },
    };

    {
        tracy_span!("vision_shadowcast");
        shadowcast(settings);
    }

    {
        tracy_span!("vision_apply_results");
        for (local_x, local_y) in vis {
            zone.visible.set(local_x, local_y, true);
            zone.explored.set(local_x, local_y, true);
        }
    }
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
    tracy_span!("update_entity_visibility_flags");
    // if clock.is_frozen() {
    //     return;
    // }

    if clock.tick_delta_accum() == 0 {
        return;
    }

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
}
