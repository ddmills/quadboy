use core::f32;

use crate::{
    cfg::{MAP_SIZE, WORLD_SIZE, ZONE_SIZE},
    common::{
        algorithm::{
            astar::{astar, AStarSettings},
            distance::Distance,
        }, Rand
    },
    domain::{
        actions::MoveAction, are_hostile, AiController, AttackAction, Collider, Energy, FactionMap, FactionMember, PlayerPosition, StairDown, StairUp, Zone
    },
    rendering::{world_to_zone_idx, world_to_zone_local, zone_xyz, Position},
    tracy_span,
};
use bevy_ecs::prelude::*;
use macroquad::prelude::trace;

pub fn find_hostile_in_range(
    entity: Entity,
    entity_pos: &Position,
    range: f32,
    world: &mut World,
) -> Option<Entity> {
    let (x, y, z) = entity_pos.world();
    let zone_idx = world_to_zone_idx(x, y, z);

    let mut zone_query = world.query::<&Zone>();
    let zone = zone_query.iter(world).find(|zone| zone.idx == zone_idx)?;

    let (local_x, local_y) = world_to_zone_local(x, y);
    let range_tiles = range as i32;

    for dx in -range_tiles..=range_tiles {
        for dy in -range_tiles..=range_tiles {
            let check_x_i32 = local_x as i32 + dx;
            let check_y_i32 = local_y as i32 + dy;

            // Bounds check for zone coordinates
            if check_x_i32 < 0
                || check_x_i32 >= ZONE_SIZE.0 as i32
                || check_y_i32 < 0
                || check_y_i32 >= ZONE_SIZE.1 as i32
            {
                continue;
            }

            let check_x = check_x_i32 as usize;
            let check_y = check_y_i32 as usize;

            if let Some(entities) = zone.entities.get(check_x, check_y) {
                for &target_entity in entities {
                    // Don't target self
                    if entity == target_entity {
                        continue;
                    }

                    if are_hostile(entity, target_entity, world) {
                        let distance = ((dx * dx + dy * dy) as f32).sqrt();
                        if distance <= range {
                            return Some(target_entity);
                        }
                    }
                }
            }
        }
    }

    None
}

pub fn attack_if_adjacent(entity: Entity, entity_pos: &Position, world: &mut World) -> bool {
    let (x, y, z) = entity_pos.world();
    let directions = [(-1, 0), (1, 0), (0, -1), (0, 1)];

    for (dx, dy) in directions.iter() {
        let check_x = (x as i32 + dx) as usize;
        let check_y = (y as i32 + dy) as usize;

        let zone_idx = world_to_zone_idx(check_x, check_y, z);
        let mut zone_query = world.query::<&Zone>();
        if let Some(zone) = zone_query.iter(world).find(|zone| zone.idx == zone_idx) {
            let (local_x, local_y) = world_to_zone_local(check_x, check_y);
            if let Some(entities) = zone.entities.get(local_x, local_y) {
                for &target_entity in entities {
                    if are_hostile(entity, target_entity, world) {
                        let attack_action = AttackAction {
                            attacker_entity: entity,
                            target_pos: (check_x, check_y, z),
                            is_bump_attack: false,
                        };
                        attack_action.apply(world);
                        return true;
                    }
                }
            }
        }
    }

    false
}

pub fn move_toward_target(
    entity: Entity,
    entity_pos: &Position,
    target_entity: Entity,
    world: &mut World,
) -> bool {
    let Some(target_faction) = world.get::<FactionMember>(target_entity) else {
        return false;
    };
    let Some(faction_map) = world.get_resource::<FactionMap>() else {
        return false;
    };
    let (x, y, z) = entity_pos.world();
    let zone_idx = world_to_zone_idx(x, y, z);

    let (local_x, local_y) = world_to_zone_local(x, y);

    // Try Dijkstra pathfinding first (same-zone movement)
    if let Some(dijkstra_map) = faction_map.get_map(target_faction.faction_id)
        && let Some((dx, dy)) = dijkstra_map.get_best_direction(local_x, local_y) {
            let new_x = (x as i32 + dx) as usize;
            let new_y = (y as i32 + dy) as usize;

            if is_move_valid(world, new_x, new_y, z) {
                let move_action = MoveAction {
                    entity,
                    new_position: (new_x, new_y, z),
                };
                move_action.apply(world);
                return true;
            }
        }

    // Fallback: Cross-zone movement toward target
    move_toward_target_cross_zone(entity, entity_pos, target_entity, world)
}

pub fn wander_near_point(
    entity: Entity,
    entity_pos: &Position,
    center: &Position,
    range: f32,
    world: &mut World,
) -> bool {
    let mut rand = Rand::new();

    if !rand.bool(0.75) {
        let directions = [(-1, 0), (1, 0), (0, -1), (0, 1)];
        let (dx, dy) = rand.pick(&directions);

        let (x, y, z) = entity_pos.world();
        let new_x = (x as i32 + dx) as usize;
        let new_y = (y as i32 + dy) as usize;

        let (center_x, center_y, _) = center.world();
        let distance_to_center = ((new_x as f32 - center_x as f32).powi(2)
            + (new_y as f32 - center_y as f32).powi(2))
        .sqrt();

        if distance_to_center <= range && is_move_valid(world, new_x, new_y, z) {
            let move_action = MoveAction {
                entity,
                new_position: (new_x, new_y, z),
            };
            move_action.apply(world);
            return true;
        }
    }

    false
}

pub fn return_to_home(
    entity: Entity,
    entity_pos: &Position,
    home_pos: &Position,
    world: &mut World,
) -> bool {
    tracy_span!("return_to_home");

    let (x, y, z) = entity_pos.world();
    let (home_x, home_y, home_z) = home_pos.world();

    // If already at home, we're done
    if x == home_x && y == home_y && z == home_z {
        return true;
    }

    let current_pos = (x, y, z);
    let target_pos = (home_x, home_y, home_z);

    // Get cached path if it exists
    let cached_path = {
        tracy_span!("get_cached_path");
        if let Some(ai) = world.get::<AiController>(entity) {
            ai.cached_home_path.clone()
        } else {
            None
        }
    };

    // Try to use cached path first
    if let Some(mut path) = cached_path {
        tracy_span!("use_cached_path");

        // Remove positions we've already passed
        while !path.is_empty() && path[0] == current_pos {
            path.remove(0);
        }

        // Try to move to the next step in the cached path
        if !path.is_empty() {
            let next_step = path[0];
            if is_move_valid(world, next_step.0, next_step.1, next_step.2) {
                tracy_span!("cached_path_move");
                let move_action = MoveAction {
                    entity,
                    new_position: next_step,
                };
                move_action.apply(world);

                // Update the cached path by removing the step we just took
                path.remove(0);
                if let Some(mut ai) = world.get_mut::<AiController>(entity) {
                    ai.cached_home_path = if path.is_empty() { None } else { Some(path) };
                }
                return true;
            } else {
                // Next step is blocked, clear the cached path and recalculate
                if let Some(mut ai) = world.get_mut::<AiController>(entity) {
                    ai.cached_home_path = None;
                }
            }
        }
    }

    // No cached path or cached path is blocked, calculate new path with A*
    {
        tracy_span!("astar_pathfinding");
        if let Some(path) = find_path_astar(current_pos, target_pos, world, Some(50.0)) {
            tracy_span!("cache_new_path");

            // Cache the new path (excluding the current position)
            let path_to_cache = if path.len() > 1 {
                path[1..].to_vec()
            } else {
                vec![]
            };

            if let Some(mut ai) = world.get_mut::<AiController>(entity) {
                ai.cached_home_path = if path_to_cache.is_empty() {
                    None
                } else {
                    Some(path_to_cache)
                };
            }

            // Move to the next step in the path
            if path.len() > 1 {
                let next_step = path[1];
                let move_action = MoveAction {
                    entity,
                    new_position: next_step,
                };
                move_action.apply(world);
                return true;
            }
        }
    }

    trace!("STUCK");
    // If A* also fails, we're truly stuck
    false
}

pub fn move_toward_last_known_position(
    entity: Entity,
    entity_pos: &Position,
    last_known_pos: &Position,
    world: &mut World,
) -> bool {
    move_toward_position(entity, entity_pos, last_known_pos, world)
}

pub fn distance_from_home(entity_pos: &Position, home_pos: &Position) -> f32 {
    let (x, y, _) = entity_pos.world();
    let (home_x, home_y, _) = home_pos.world();

    ((x as f32 - home_x as f32).powi(2) + (y as f32 - home_y as f32).powi(2)).sqrt()
}

pub fn consume_wait_energy(entity: Entity, world: &mut World) {
    if let Some(mut energy) = world.get_mut::<Energy>(entity) {
        // Use moderate energy cost for teleport waiting - enough to check wait period but not spam turns
        // 200 energy gives entities turns roughly every 2-3 game ticks while waiting
        energy.consume_energy(200);
    }
}

fn is_move_valid(world: &mut World, new_x: usize, new_y: usize, z: usize) -> bool {
    tracy_span!("is_move_valid");

    let max_x = (MAP_SIZE.0 * ZONE_SIZE.0) - 1;
    let max_y = (MAP_SIZE.1 * ZONE_SIZE.1) - 1;

    if new_x > max_x || new_y > max_y {
        return false;
    }

    let dest_zone_idx = world_to_zone_idx(new_x, new_y, z);
    let (local_x, local_y) = world_to_zone_local(new_x, new_y);

    let entities_at_pos = {
        tracy_span!("get_zone_entities");
        let mut zone_query = world.query::<&Zone>();
        let zone = zone_query
            .iter(world)
            .find(|zone| zone.idx == dest_zone_idx);

        if let Some(zone) = zone {
            zone.entities.get(local_x, local_y).cloned()
        } else {
            return false;
        }
    };

    if let Some(entities) = entities_at_pos {
        tracy_span!("check_colliders");
        let mut collider_query = world.query::<&Collider>();
        for entity_at_pos in entities {
            if collider_query.get(world, entity_at_pos).is_ok() {
                return false;
            }
        }
    }

    true
}

/// Cross-zone pathfinding: move toward target even if in different zone
fn move_toward_target_cross_zone(
    entity: Entity,
    entity_pos: &Position,
    target_entity: Entity,
    world: &mut World,
) -> bool {
    // Try to get target position directly first
    if let Some(target_pos) = world.get::<Position>(target_entity) {
        let target_pos = target_pos.clone();
        return move_toward_position(entity, entity_pos, &target_pos, world);
    }

    // Fallback: Move toward player's last known position
    if let Some(player_pos) = world.get_resource::<PlayerPosition>() {
        let target_pos = Position::new(
            player_pos.x as usize,
            player_pos.y as usize,
            player_pos.z as usize,
        );
        return move_toward_position(entity, entity_pos, &target_pos, world);
    }

    false
}

/// Move directly toward a target position using simple directional movement
fn move_toward_position(
    entity: Entity,
    entity_pos: &Position,
    target_pos: &Position,
    world: &mut World,
) -> bool {
    let (current_x, current_y, current_z) = entity_pos.world();
    let (target_x, target_y, target_z) = target_pos.world();

    // Only move on same Z level for now
    if current_z != target_z {
        return false;
    }

    // Calculate direction to target
    let dx = if target_x > current_x {
        1
    } else if target_x < current_x {
        -1
    } else {
        0
    };

    let dy = if target_y > current_y {
        1
    } else if target_y < current_y {
        -1
    } else {
        0
    };

    // Try to move in calculated direction
    if dx != 0 || dy != 0 {
        let new_x = (current_x as i32 + dx) as usize;
        let new_y = (current_y as i32 + dy) as usize;

        if is_move_valid(world, new_x, new_y, current_z) {
            let move_action = MoveAction {
                entity,
                new_position: (new_x, new_y, current_z),
            };
            move_action.apply(world);
            return true;
        }
    }

    false
}

pub fn check_at_zone_boundary(pos: &Position) -> bool {
    let (local_x, local_y) = pos.zone_local();
    let at_boundary =
        local_x <= 1 || local_x >= ZONE_SIZE.0 - 2 || local_y <= 1 || local_y >= ZONE_SIZE.1 - 2;
    if at_boundary {
        println!("Entity at zone boundary: local ({}, {})", local_x, local_y);
    }
    at_boundary
}

pub fn teleport_to_zone_edge(
    entity: Entity,
    current_pos: &Position,
    target_zone_idx: usize,
    world: &mut World,
) -> bool {
    println!(
        "Attempting teleportation for entity {:?} to zone {}",
        entity, target_zone_idx
    );
    // Get player's current position to teleport near them
    let player_pos = if let Some(player_pos) = world.get_resource::<PlayerPosition>() {
        (
            player_pos.x as usize,
            player_pos.y as usize,
            player_pos.z as usize,
        )
    } else {
        return false; // No player position available
    };

    let current_zone = current_pos.zone_idx();
    let (curr_zx, curr_zy, curr_zz) = zone_xyz(current_zone);
    let (target_zx, target_zy, target_zz) = zone_xyz(target_zone_idx);

    // Check for vertical movement (different Z levels)
    if target_zz != curr_zz {
        // For vertical movement, find nearest staircase in current zone
        if let Some((stair_x, stair_y)) = find_nearest_staircase(current_pos, target_zz, world) {
            println!(
                "Teleporting entity {:?} to staircase at ({}, {})",
                entity, stair_x, stair_y
            );

            // Update entity position to staircase and move to target Z level
            if let Some(mut pos) = world.get_mut::<Position>(entity) {
                pos.x = stair_x as f32;
                pos.y = stair_y as f32;
                pos.z = target_zz as f32; // Move to target Z level where player is
            }

            // Consume energy for teleportation
            if let Some(mut energy) = world.get_mut::<Energy>(entity) {
                energy.consume_energy(100);
            }

            return true;
        } else {
            println!("No suitable staircase found for vertical pursuit");
            return false;
        }
    }

    // Convert player world coordinates to local coordinates within target zone
    let (player_local_x, player_local_y) = (player_pos.0 % ZONE_SIZE.0, player_pos.1 % ZONE_SIZE.1);

    // Find an open tile near the player on the appropriate edge, fallback to original working positions
    let (new_x, new_y) = if target_zx > curr_zx {
        // Coming from west, place on west edge of target zone
        let edge_x = target_zx * ZONE_SIZE.0;
        // Use player's local Y position within the target zone
        let target_y = target_zy * ZONE_SIZE.1 + player_local_y;
        if let Some(open_y) =
            find_open_position_along_edge(world, edge_x, target_y, target_zz, true)
        {
            (edge_x, open_y)
        } else {
            // Fallback to original working position: west edge, center of zone
            (edge_x, target_zy * ZONE_SIZE.1 + ZONE_SIZE.1 / 2)
        }
    } else if target_zx < curr_zx {
        // Coming from east, place on east edge of target zone
        let edge_x = (target_zx + 1) * ZONE_SIZE.0 - 1;
        // Use player's local Y position within the target zone
        let target_y = target_zy * ZONE_SIZE.1 + player_local_y;
        if let Some(open_y) =
            find_open_position_along_edge(world, edge_x, target_y, target_zz, true)
        {
            (edge_x, open_y)
        } else {
            // Fallback to original working position: east edge, center of zone
            (edge_x, target_zy * ZONE_SIZE.1 + ZONE_SIZE.1 / 2)
        }
    } else if target_zy > curr_zy {
        // Coming from north, place on north edge of target zone
        let edge_y = target_zy * ZONE_SIZE.1;
        // Use player's local X position within the target zone
        let target_x = target_zx * ZONE_SIZE.0 + player_local_x;
        if let Some(open_x) =
            find_open_position_along_edge(world, edge_y, target_x, target_zz, false)
        {
            trace!("using open pos on edge!");
            (open_x, edge_y)
        } else {
            trace!("fallback to center!");
            // Fallback to original working position: north edge, center of zone
            (target_zx * ZONE_SIZE.0 + ZONE_SIZE.0 / 2, edge_y)
        }
    } else if target_zy < curr_zy {
        // Coming from south, place on south edge of target zone
        let edge_y = (target_zy + 1) * ZONE_SIZE.1 - 1;
        // Use player's local X position within the target zone
        let target_x = target_zx * ZONE_SIZE.0 + player_local_x;
        if let Some(open_x) =
            find_open_position_along_edge(world, edge_y, target_x, target_zz, false)
        {
            (open_x, edge_y)
        } else {
            // Fallback to original working position: south edge, center of zone
            (target_zx * ZONE_SIZE.0 + ZONE_SIZE.0 / 2, edge_y)
        }
    } else {
        return false; // Same zone
    };

    // Update entity position directly
    if let Some(mut pos) = world.get_mut::<Position>(entity) {
        pos.x = new_x as f32;
        pos.y = new_y as f32;
        pos.z = target_zz as f32;
    }

    // Consume energy for teleportation
    if let Some(mut energy) = world.get_mut::<Energy>(entity) {
        energy.consume_energy(100); // High energy cost for teleportation
    }

    println!("Teleportation successful to ({}, {})", new_x, new_y);
    true
}

/// Find an open position along an edge closest to a target coordinate
/// If search_along_y is true, varies Y coordinate around target_coord while keeping X fixed
/// If search_along_y is false, varies X coordinate around target_coord while keeping Y fixed
fn find_open_position_along_edge(
    world: &mut World,
    fixed_coord: usize,
    target_coord: usize,
    z: usize,
    search_along_y: bool,
) -> Option<usize> {
    let mut valid_positions = Vec::new();

    // First try the target position
    let (test_x, test_y) = if search_along_y {
        (fixed_coord, target_coord)
    } else {
        (target_coord, fixed_coord)
    };

    if is_move_valid(world, test_x, test_y, z) {
        valid_positions.push(target_coord);
    }

    // Determine zone bounds for search
    let zone_idx = world_to_zone_idx(
        if search_along_y {
            fixed_coord
        } else {
            target_coord
        },
        if search_along_y {
            target_coord
        } else {
            fixed_coord
        },
        z,
    );
    let (zx, zy, _) = zone_xyz(zone_idx);
    let zone_min_x = zx * ZONE_SIZE.0;
    let zone_max_x = zone_min_x + ZONE_SIZE.0 - 1;
    let zone_min_y = zy * ZONE_SIZE.1;
    let zone_max_y = zone_min_y + ZONE_SIZE.1 - 1;

    // Search outward from target position in both directions
    for offset in 1..=10 {
        // Try positive offset with bounds check
        let pos_coord = target_coord + offset;
        let pos_valid = if search_along_y {
            pos_coord <= zone_max_y
        } else {
            pos_coord <= zone_max_x
        };

        if pos_valid {
            let (test_x, test_y) = if search_along_y {
                (fixed_coord, pos_coord)
            } else {
                (pos_coord, fixed_coord)
            };

            if is_move_valid(world, test_x, test_y, z) {
                valid_positions.push(pos_coord);
            }
        }

        // Try negative offset with bounds check
        if offset <= target_coord {
            let neg_coord = target_coord - offset;
            let neg_valid = if search_along_y {
                neg_coord >= zone_min_y
            } else {
                neg_coord >= zone_min_x
            };

            if neg_valid {
                let (test_x, test_y) = if search_along_y {
                    (fixed_coord, neg_coord)
                } else {
                    (neg_coord, fixed_coord)
                };

                if is_move_valid(world, test_x, test_y, z) {
                    valid_positions.push(neg_coord);
                }
            }
        }
    }

    // Return the position closest to the target coordinate
    valid_positions.into_iter().min_by_key(|&coord| {
        coord.abs_diff(target_coord)
    })
}

fn find_nearest_staircase(
    current_pos: &Position,
    target_z: usize,
    world: &mut World,
) -> Option<(usize, usize)> {
    let (current_x, current_y, current_z) = current_pos.world();
    let current_zone_idx = world_to_zone_idx(current_x, current_y, current_z);

    let need_stair_up = target_z < current_z;
    let need_stair_down = target_z > current_z;

    if !need_stair_up && !need_stair_down {
        return None;
    }

    let mut nearest_stair: Option<(usize, usize, f32)> = None;

    if need_stair_up {
        let mut q_stairs_up = world.query_filtered::<&Position, With<StairUp>>();
        for stair_pos in q_stairs_up.iter(world) {
            let (stair_x, stair_y, stair_z) = stair_pos.world();
            let stair_zone_idx = world_to_zone_idx(stair_x, stair_y, stair_z);

            if stair_zone_idx == current_zone_idx && stair_z == current_z {
                let distance = ((current_x as f32 - stair_x as f32).powi(2)
                    + (current_y as f32 - stair_y as f32).powi(2))
                .sqrt();

                if nearest_stair.is_none() || distance < nearest_stair.unwrap().2 {
                    nearest_stair = Some((stair_x, stair_y, distance));
                }
            }
        }
    } else if need_stair_down {
        let mut q_stairs_down = world.query_filtered::<&Position, With<StairDown>>();
        for stair_pos in q_stairs_down.iter(world) {
            let (stair_x, stair_y, stair_z) = stair_pos.world();
            let stair_zone_idx = world_to_zone_idx(stair_x, stair_y, stair_z);

            if stair_zone_idx == current_zone_idx && stair_z == current_z {
                let distance = ((current_x as f32 - stair_x as f32).powi(2)
                    + (current_y as f32 - stair_y as f32).powi(2))
                .sqrt();

                if nearest_stair.is_none() || distance < nearest_stair.unwrap().2 {
                    nearest_stair = Some((stair_x, stair_y, distance));
                }
            }
        }
    }

    nearest_stair.map(|(x, y, _)| (x, y))
}

fn get_valid_neighbors(pos: (usize, usize, usize), world: &mut World) -> Vec<[usize; 3]> {
    let (x, y, z) = pos;
    let mut neighbors = Vec::new();

    // 8-directional movement (4 cardinal + 4 diagonal)
    let directions = [
        (-1, -1),
        (0, -1),
        (1, -1), // NW, N, NE
        (-1, 0),
        (1, 0), // W,     E
        (-1, 1),
        (0, 1),
        (1, 1), // SW, S, SE
    ];

    for (dx, dy) in directions.iter() {
        let new_x_i32 = x as i32 + dx;
        let new_y_i32 = y as i32 + dy;

        // Check bounds
        if new_x_i32 < 0 || new_y_i32 < 0 {
            continue;
        }

        let new_x = new_x_i32 as usize;
        let new_y = new_y_i32 as usize;

        // Check world bounds
        let max_x = (MAP_SIZE.0 * ZONE_SIZE.0) - 1;
        let max_y = (MAP_SIZE.1 * ZONE_SIZE.1) - 1;
        if new_x > max_x || new_y > max_y {
            continue;
        }

        // Check if move is valid (no colliders)
        if is_move_valid(world, new_x, new_y, z) {
            neighbors.push([new_x, new_y, z]);
        }
    }

    neighbors
}

pub fn find_path_astar(
    from: (usize, usize, usize),
    to: (usize, usize, usize),
    world: &mut World,
    max_distance: Option<f32>,
) -> Option<Vec<(usize, usize, usize)>> {
    tracy_span!("find_path_astar");

    let zone_idx = world_to_zone_idx(from.0, from.1, from.2);
    let zone = world.query::<&Zone>().iter(world).find(|z| z.idx == zone_idx)?;

    let result = {
        tracy_span!("astar_algorithm");
        astar(AStarSettings {
            start: [from.0, from.1, from.2],
            is_goal: |[x, y, z]| x == to.0 && y == to.1 && z == to.2,
            cost: |[_from_x, _from_y, _from_z], [to_x, to_y, _to_z]| {
                let (local_x, local_y) = world_to_zone_local(to_x, to_y);
                let Some(v) = zone.colliders.get(local_x, local_y) else {
                    return f32::INFINITY;
                };

                if v.is_empty() {
                    return 1.0;
                }

                f32::INFINITY
            },
            heuristic: |[x, y, z]| {
                Distance::chebyshev(
                    [x as i32, y as i32, z as i32],
                    [to.0 as i32, to.1 as i32, to.2 as i32],
                )
            },
            neighbors: |[x, y, z]| {
                let mut neighbors = Vec::new();

                // 8-directional movement
                let directions = [
                    (-1, -1),
                    (0, -1),
                    (1, -1),
                    (-1, 0),
                    (1, 0),
                    (-1, 1),
                    (0, 1),
                    (1, 1),
                ];

                for (dx, dy) in directions.iter() {
                    let new_x_i32 = x as i32 + dx;
                    let new_y_i32 = y as i32 + dy;

                    if new_x_i32 >= 0
                        && new_x_i32 < WORLD_SIZE.0 as i32
                        && new_y_i32 >= 0
                        && new_y_i32 < WORLD_SIZE.1 as i32
                    {
                        neighbors.push([
                            new_x_i32 as usize,
                            new_y_i32 as usize,
                            z
                        ]);
                    }
                }

                neighbors
            },
            max_depth: 1000, // Reasonable depth for pathfinding
            max_cost: max_distance,
        })
    };

    if result.is_success {
        tracy_span!("process_result");
        // A* returns path in reverse order, so reverse it to get from start to goal
        let mut path: Vec<(usize, usize, usize)> =
            result.path.into_iter().map(|[x, y, z]| (x, y, z)).collect();
        path.reverse();
        Some(path)
    } else {
        None
    }
}
