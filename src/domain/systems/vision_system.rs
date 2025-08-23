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
    pub last_player_pos: Option<(usize, usize, usize)>,
    pub last_vision_range: Option<usize>,
}

pub fn update_player_vision(
    q_player: Query<&Position, (With<Player>, With<Vision>)>,
    q_vision: Query<&Vision, With<Player>>,
    mut q_zones: Query<&mut Zone>,
    q_vision_blockers: Query<&Position, With<VisionBlocker>>,
    mut vision_cache: ResMut<VisionCache>,
    clock: Res<Clock>,
) {
    let Ok(player_pos) = q_player.single() else {
        return;
    };

    let Ok(vision) = q_vision.single() else {
        return;
    };

    // Early exit: if no in-game time has passed, no need to update vision
    if clock.tick_delta() == 0 {
        return;
    }

    telemetry::begin_zone("update_player_vision");

    let player_world_pos = player_pos.world();

    // Clear all visible grids before recomputing
    for mut zone in q_zones.iter_mut() {
        zone.visible.fill(|_, _| false);
    }

    // Build vision blocker cache for faster lookups
    let mut blocker_cache: std::collections::HashMap<(i32, i32, i32), bool> =
        std::collections::HashMap::new();
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

    let settings = ShadowcastSettings {
        start_x: player_x,
        start_y: player_y,
        distance: vision.range as i32,
        is_blocker: |x: i32, y: i32| blocker_cache.contains_key(&(x, y, player_z)),
        on_light: |x: i32, y: i32, _distance: f64| {
            if x < 0 || y < 0 {
                return; // Skip negative coordinates
            }

            let world_x = x as usize;
            let world_y = y as usize;
            let world_z = player_z as usize;

            let zone_idx = world_to_zone_idx(world_x, world_y, world_z);

            // Find the zone and update its grids
            for mut zone in q_zones.iter_mut() {
                if zone.idx == zone_idx {
                    let (local_x, local_y) = world_to_zone_local(world_x, world_y);
                    zone.visible.set(local_x, local_y, true);
                    zone.explored.set(local_x, local_y, true);
                    break;
                }
            }
        },
    };

    shadowcast(settings);

    // Update cache
    vision_cache.last_player_pos = Some(player_world_pos);
    vision_cache.last_vision_range = Some(vision.range);
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
            Option<&HideWhenNotVisible>,
        ),
        With<ApplyVisibilityEffects>,
    >,
    clock: Res<Clock>,
) {
    // Early exit: if no in-game time has passed, no need to update visibility flags
    if clock.tick_delta() == 0 {
        return;
    }

    telemetry::begin_zone("update_entity_visibility_flags");

    for (entity, position, has_visible, has_explored, hide_when_not_visible) in
        q_entities.iter_mut()
    {
        let world_pos = position.world();
        let zone_idx = world_to_zone_idx(world_pos.0, world_pos.1, world_pos.2);

        // Find the zone this entity is in
        let Some(zone) = q_zones.iter().find(|z| z.idx == zone_idx) else {
            continue;
        };

        let (local_x, local_y) = world_to_zone_local(world_pos.0, world_pos.1);

        // Check if entity should be visible
        let is_visible = zone.visible.get(local_x, local_y).copied().unwrap_or(false);
        let is_explored = zone
            .explored
            .get(local_x, local_y)
            .copied()
            .unwrap_or(false);

        // Update IsVisible component
        match (is_visible, has_visible.is_some()) {
            (true, false) => {
                cmds.entity(entity).insert(IsVisible);
            }
            (false, true) => {
                cmds.entity(entity).remove::<IsVisible>();
            }
            _ => {} // No change needed
        }

        // Update IsExplored component (but not for entities that hide when not visible)
        if hide_when_not_visible.is_none() {
            match (is_explored, has_explored.is_some()) {
                (true, false) => {
                    cmds.entity(entity).insert(IsExplored);
                }
                // Note: Once explored, entities remain explored (no removal)
                _ => {}
            }
        }
    }
    telemetry::end_zone();
}
