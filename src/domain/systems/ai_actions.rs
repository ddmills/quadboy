use std::collections::HashMap;

use bevy_ecs::prelude::*;
use macroquad::prelude::trace;

use crate::{
    cfg::WORLD_SIZE,
    common::algorithm::{
        astar::{astar, AStarSettings},
        distance::Distance,
    },
    domain::{AiContext, AttackAction, MoveAction, MovementCapabilities, StairDown, StairUp, Zone},
    rendering::{world_to_zone_idx, world_to_zone_local, zone_local_to_world, zone_xyz, Position},
};

pub fn ai_try_use_stair(world: &mut World, entity: Entity, going_down: bool) -> bool {
    let Some(position) = world.get::<Position>(entity) else {
        return false;
    };

    let pos_world = position.world();
    let entity_zone_idx = world_to_zone_idx(pos_world.0, pos_world.1, pos_world.2);
    let entity_local = world_to_zone_local(pos_world.0, pos_world.1);

    let mut zone_cache: HashMap<usize, &Zone> = HashMap::new();
    for zone in world.query::<&Zone>().iter(world) {
        zone_cache.insert(zone.idx, zone);
    }

    let Some(current_zone) = zone_cache.get(&entity_zone_idx) else {
        return false;
    };

    let Some(entities_at_pos) = current_zone.entities.get(entity_local.0, entity_local.1) else {
        return false;
    };

    for &stair_entity in entities_at_pos.iter() {
        let has_correct_stair = if going_down {
            world.get::<StairDown>(stair_entity).is_some()
        } else {
            world.get::<StairUp>(stair_entity).is_some()
        };

        if has_correct_stair {
            let new_z = if going_down {
                pos_world.2 + 1
            } else {
                pos_world.2.saturating_sub(1)
            };

            let action = MoveAction {
                entity,
                new_position: (pos_world.0, pos_world.1, new_z),
            };
            action.apply(world);
            return true;
        }
    }

    false
}

pub fn ai_try_navigate_to_stair(world: &mut World, entity: Entity, current_z: i32, target_z: i32) -> bool {
    let Some(position) = world.get::<Position>(entity) else {
        return false;
    };

    let pos_world = position.world();
    let going_down = current_z < target_z;
    let mut nearest_stair_pos: Option<(i32, i32)> = None;
    let mut nearest_distance = f32::INFINITY;

    for zone in world.query::<&Zone>().iter(world) {
        let (_, _, zone_z) = zone_xyz(zone.idx);
        if zone_z != current_z as usize {
            continue;
        }

        for y in 0..zone.entities.height() {
            for x in 0..zone.entities.width() {
                let Some(entities) = zone.entities.get(x, y) else {
                    continue;
                };

                for &stair_entity in entities.iter() {
                    let should_target = if going_down {
                        world.get::<StairDown>(stair_entity).is_some()
                    } else {
                        world.get::<StairUp>(stair_entity).is_some()
                    };

                    if should_target {
                        let (world_x, world_y, world_z) = zone_local_to_world(zone.idx, x, y);

                        let distance = Distance::diagonal(
                            [pos_world.0 as i32, pos_world.1 as i32, current_z],
                            [world_x as i32, world_y as i32, world_z as i32]
                        );

                        if distance < nearest_distance {
                            nearest_distance = distance;
                            nearest_stair_pos = Some((world_x as i32, world_y as i32));
                        }
                    }
                }
            }
        }
    }

    if let Some((stair_x, stair_y)) = nearest_stair_pos {
        return ai_try_move_toward(world, entity, (stair_x as usize, stair_y as usize, current_z as usize));
    }

    false
}

pub fn ai_try_attacking_nearby(world: &mut World, entity: Entity, context: &mut AiContext) -> bool {
    let Some(nearest) = context.nearest_hostile() else {
        return false;
    };

    // what is 'entity' attack range?
    if nearest.distance > 1.75 {
        return false;
    }

    let attack = AttackAction {
        attacker_entity: entity,
        target_pos: nearest.pos,
        is_bump_attack: true,
    };

    attack.apply(world);
    true
}

pub fn ai_try_select_target(world: &mut World, entity: Entity, context: &mut AiContext) -> bool {
    if let Some(t) = context.target {
        context.target = Some(t);
        return true;
    }

    let Some(nearest) = context.nearest_hostile() else {
        return false;
    };

    context.target = Some(*nearest);
    true
}

pub fn ai_try_move_toward_target(
    world: &mut World,
    entity: Entity,
    context: &mut AiContext,
) -> bool {
    let Some(target) = context.target else {
        return false;
    };

    ai_try_move_toward(world, entity, target.pos)
}

pub fn ai_try_move_toward(
    world: &mut World,
    entity: Entity,
    target_pos: (usize, usize, usize),
) -> bool {
    let mut zone_cache: HashMap<usize, &Zone> = HashMap::new();

    let (start_x, start_y, start_z) = {
        let Some(position) = world.get::<Position>(entity) else {
            return false;
        };

        let pos_world = position.world();

        (pos_world.0 as i32, pos_world.1 as i32, pos_world.2 as i32)
    };

    let (target_x, target_y, target_z) = (
        target_pos.0 as i32,
        target_pos.1 as i32,
        target_pos.2 as i32,
    );

    // check if on different Z levels!
    if start_z != target_z {
        let going_down = start_z < target_z;

        // First try to use a stair we're already on
        if ai_try_use_stair(world, entity, going_down) {
            return true;
        }

        // If that failed, try to navigate to the nearest appropriate stair
        return ai_try_navigate_to_stair(world, entity, start_z, target_z);
    }

    let movement_flags = world
        .get::<MovementCapabilities>(entity)
        .unwrap_or(&MovementCapabilities::terrestrial())
        .flags;

    for zone in world.query::<&Zone>().iter(world) {
        zone_cache.insert(zone.idx, zone);
    }

    let deltas = [
        (-1, -1),
        (0, -1),
        (1, -1),
        (-1, 0),
        (1, 0),
        (-1, 1),
        (0, 1),
        (1, 1),
    ];

    let mut result = astar(AStarSettings {
        start: (start_x, start_y),
        is_goal: |(x, y)| x == target_x && y == target_y,
        cost: |(from_x, from_y), (to_x, to_y)| {
            if to_x < 0
                || to_y < 0
                || to_x as usize >= WORLD_SIZE.0
                || to_y as usize >= WORLD_SIZE.1
            {
                return f32::INFINITY;
            }

            let to_zone_idx = world_to_zone_idx(to_x as usize, to_y as usize, target_z as usize);
            let Some(zone) = zone_cache.get(&to_zone_idx) else {
                trace!(
                    "zone not in cache {},{},{} = {}",
                    to_x, to_y, target_z, to_zone_idx
                );
                return f32::INFINITY;
            };

            // Convert to local zone coordinates
            let local = world_to_zone_local(to_x as usize, to_y as usize);

            // Get cached collider flags at target position
            let collider_flags = zone.colliders.get_flags(local.0, local.1);

            // Check if movement is blocked
            if movement_flags.is_blocked_by(collider_flags) {
                return f32::INFINITY;
            }

            Distance::diagonal([from_x, from_y, target_z], [to_x, to_y, target_z])
        },
        heuristic: |(to_x, to_y)| {
            Distance::diagonal([start_x, start_y, start_z], [to_x, to_y, target_z])
        },
        neighbors: |(x, y)| deltas.iter().map(|(dx, dy)| (x + dx, y + dy)).collect(),
        max_depth: 500,
        max_cost: Some(100.),
    });

    // last element is the starting position
    result.path.pop();

    if !result.is_success || result.path.is_empty() {
        trace!("A* FAILED");
        return false;
    }

    let Some(move_to_target) = result.path.last() else {
        return false;
    };

    let action = MoveAction {
        entity,
        new_position: (
            move_to_target.0 as usize,
            move_to_target.1 as usize,
            start_z as usize,
        ),
    };

    action.apply(world);
    true
}
