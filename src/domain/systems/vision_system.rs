use std::collections::{HashMap, HashSet};

use crate::{
    common::algorithm::shadowcast::{ShadowcastSettings, shadowcast},
    domain::{
        ApplyVisibilityEffects, HideWhenNotVisible, IsExplored, IsVisible, Player, Vision,
        VisionBlocker, Zone,
    },
    engine::Clock,
    rendering::{Position, world_to_zone_idx, world_to_zone_local},
};
use bevy_ecs::prelude::*;
use macroquad::telemetry;

#[derive(Resource, Default)]
pub struct VisionCache {
    pub visibility_added: Vec<(usize, usize)>,
    pub visibility_removed: Vec<(usize, usize)>,
    pub zone_lookup: HashMap<usize, Entity>,
    pub changed_zones: HashSet<usize>,
    pub vision_blocker_cache: HashMap<(i32, i32, i32), bool>,
}

pub fn update_player_vision(
    q_player: Query<&Position, (With<Player>, With<Vision>)>,
    q_vision: Query<&Vision, With<Player>>,
    mut q_zones: Query<(Entity, &mut Zone)>,
    q_vision_blockers: Query<&Position, With<VisionBlocker>>,
    mut vision_cache: ResMut<VisionCache>,
    clock: Res<Clock>,
) {
    telemetry::begin_zone("update_player_vision");

    // Early returns for invalid states
    if clock.is_frozen() {
        telemetry::end_zone();
        return;
    }

    let (Ok(player_pos), Ok(vision)) = (q_player.single(), q_vision.single()) else {
        telemetry::end_zone();
        return;
    };

    let player_world_pos = player_pos.world();

    // Initialize frame cache
    vision_cache.visibility_added.clear();
    vision_cache.visibility_removed.clear();
    vision_cache.changed_zones.clear();

    // Store current visibility state and prepare zones
    let mut current_visibility = HashMap::new();
    let cache_needs_rebuild = vision_cache.zone_lookup.is_empty();

    for (zone_entity, mut zone) in q_zones.iter_mut() {
        // Build zone lookup cache on first run
        if cache_needs_rebuild {
            vision_cache.zone_lookup.insert(zone.idx, zone_entity);
        }

        // Store current visibility state before clearing
        for x in 0..zone.visible.width() {
            for y in 0..zone.visible.height() {
                if zone.visible.get(x, y).copied().unwrap_or(false) {
                    let grid_index = x * zone.visible.height() + y;
                    current_visibility.insert((zone.idx, grid_index), true);
                }
            }
        }

        // Clear visibility grid for new calculation
        zone.visible.fill(|_, _| false);
    }

    // Update vision blocker cache
    vision_cache.vision_blocker_cache.clear();
    for blocker_pos in q_vision_blockers.iter() {
        let world_pos = blocker_pos.world();
        vision_cache.vision_blocker_cache.insert(
            (world_pos.0 as i32, world_pos.1 as i32, world_pos.2 as i32),
            true,
        );
    }

    let (player_x, player_y, player_z) = (
        player_world_pos.0 as i32,
        player_world_pos.1 as i32,
        player_world_pos.2 as i32,
    );

    // Run shadowcast and collect visible tiles
    let mut lit_tiles = Vec::new();
    let settings = ShadowcastSettings {
        start_x: player_x,
        start_y: player_y,
        distance: vision.range as i32,
        is_blocker: |x: i32, y: i32| {
            vision_cache
                .vision_blocker_cache
                .contains_key(&(x, y, player_z))
        },
        on_light: |x: i32, y: i32, _distance: f64| {
            if x >= 0 && y >= 0 {
                lit_tiles.push((x as usize, y as usize));
            }
        },
    };
    shadowcast(settings);

    // Process lit tiles and update zones
    let mut zone_updates: HashMap<usize, Vec<(usize, usize)>> = HashMap::new();
    for (world_x, world_y) in lit_tiles {
        let zone_idx = world_to_zone_idx(world_x, world_y, player_world_pos.2);
        zone_updates
            .entry(zone_idx)
            .or_default()
            .push((world_x, world_y));
    }

    // Apply visibility updates to zones
    for (_, mut zone) in q_zones.iter_mut() {
        if let Some(tiles) = zone_updates.get(&zone.idx) {
            for &(world_x, world_y) in tiles {
                let (local_x, local_y) = world_to_zone_local(world_x, world_y);
                let grid_index = local_x * zone.visible.height() + local_y;

                // Track newly visible tiles
                if !current_visibility.contains_key(&(zone.idx, grid_index)) {
                    vision_cache.visibility_added.push((zone.idx, grid_index));
                    vision_cache.changed_zones.insert(zone.idx);
                }

                zone.visible.set(local_x, local_y, true);
                zone.explored.set(local_x, local_y, true);
            }
        }
    }

    // Find tiles that lost visibility
    let mut zones_to_check: HashMap<usize, Vec<usize>> = HashMap::new();
    for ((zone_idx, grid_index), _) in current_visibility {
        zones_to_check.entry(zone_idx).or_default().push(grid_index);
    }

    for (_, zone) in q_zones.iter() {
        if let Some(grid_indices) = zones_to_check.get(&zone.idx) {
            for &grid_index in grid_indices {
                let local_x = grid_index / zone.visible.height();
                let local_y = grid_index % zone.visible.height();

                if !zone.visible.get(local_x, local_y).copied().unwrap_or(false) {
                    vision_cache.visibility_removed.push((zone.idx, grid_index));
                    vision_cache.changed_zones.insert(zone.idx);
                }
            }
        }
    }

    telemetry::end_zone();
}

pub fn update_entity_visibility_flags(
    mut cmds: Commands,
    q_zones: Query<&Zone>,
    q_entities: Query<
        (
            Entity,
            &Position,
            Option<&IsVisible>,
            Option<&IsExplored>,
            Option<&HideWhenNotVisible>,
        ),
        With<ApplyVisibilityEffects>,
    >,
    vision_cache: Res<VisionCache>,
    clock: Res<Clock>,
) {
    // Early returns for invalid states
    if clock.is_frozen() || vision_cache.changed_zones.is_empty() {
        return;
    }

    telemetry::begin_zone("update_entity_visibility_flags");

    // Build zone lookup for O(1) access
    let zone_map: HashMap<usize, &Zone> = q_zones.iter().map(|zone| (zone.idx, zone)).collect();

    // Process only entities in zones with visibility changes
    for (entity, position, has_visible, has_explored, hide_when_not_visible) in q_entities.iter() {
        let world_pos = position.world();
        let zone_idx = world_to_zone_idx(world_pos.0, world_pos.1, world_pos.2);

        // Skip entities in unchanged zones
        if !vision_cache.changed_zones.contains(&zone_idx) {
            continue;
        }

        let Some(zone) = zone_map.get(&zone_idx) else {
            continue;
        };

        let (local_x, local_y) = world_to_zone_local(world_pos.0, world_pos.1);

        // Get current visibility state
        let is_visible = zone.visible.get(local_x, local_y).copied().unwrap_or(false);
        let is_explored = zone
            .explored
            .get(local_x, local_y)
            .copied()
            .unwrap_or(false);

        let currently_visible = has_visible.is_some();
        let currently_explored = has_explored.is_some();

        // Update visibility component if changed
        if is_visible != currently_visible {
            match is_visible {
                true => cmds.entity(entity).insert(IsVisible),
                false => cmds.entity(entity).remove::<IsVisible>(),
            };
        }

        // Update explored component (skip for hide-when-not-visible entities)
        if hide_when_not_visible.is_none() && is_explored && !currently_explored {
            cmds.entity(entity).insert(IsExplored);
        }
    }

    telemetry::end_zone();
}
