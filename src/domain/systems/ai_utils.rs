use crate::{
    cfg::{MAP_SIZE, ZONE_SIZE},
    common::Rand,
    domain::{
        AttackAction, Collider, Energy, EnergyActionType, FactionMap, FactionMember,
        PlayerPosition, Zone,
        actions::MoveAction,
        are_hostile,
        get_base_energy_cost,
    },
    rendering::{Position, world_to_zone_idx, world_to_zone_local, zone_xyz},
};
use bevy_ecs::prelude::*;

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
    if let Some(dijkstra_map) = faction_map.get_map(target_faction.faction_id) {
        if let Some((dx, dy)) = dijkstra_map.get_best_direction(local_x, local_y) {
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
    let (x, y, z) = entity_pos.world();
    let (home_x, home_y, _) = home_pos.world();

    let dx = if home_x > x {
        1
    } else if home_x < x {
        -1
    } else {
        0
    };
    let dy = if home_y > y {
        1
    } else if home_y < y {
        -1
    } else {
        0
    };

    if dx != 0 || dy != 0 {
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
        // Use a smaller energy cost for teleport waiting so entities get turns frequently enough
        // to check if the wait period has elapsed (200 ticks)
        energy.consume_energy(50);
    }
}

fn is_move_valid(world: &mut World, new_x: usize, new_y: usize, z: usize) -> bool {
    let max_x = (MAP_SIZE.0 * ZONE_SIZE.0) - 1;
    let max_y = (MAP_SIZE.1 * ZONE_SIZE.1) - 1;

    if new_x > max_x || new_y > max_y {
        return false;
    }

    let dest_zone_idx = world_to_zone_idx(new_x, new_y, z);
    let (local_x, local_y) = world_to_zone_local(new_x, new_y);

    let entities_at_pos = {
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
        let target_pos = Position::new(player_pos.x as usize, player_pos.y as usize, player_pos.z as usize);
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
    let at_boundary = local_x <= 1 || local_x >= ZONE_SIZE.0 - 2 ||
        local_y <= 1 || local_y >= ZONE_SIZE.1 - 2;
    if at_boundary {
        println!("Entity at zone boundary: local ({}, {})", local_x, local_y);
    }
    at_boundary
}

pub fn teleport_to_zone_edge(
    entity: Entity,
    current_pos: &Position,
    target_zone_idx: usize,
    world: &mut World
) -> bool {
    println!("Attempting teleportation for entity {:?} to zone {}", entity, target_zone_idx);
    // Get player's current position to teleport near them
    let player_pos = if let Some(player_pos) = world.get_resource::<PlayerPosition>() {
        (player_pos.x as usize, player_pos.y as usize, player_pos.z as usize)
    } else {
        return false; // No player position available
    };

    let current_zone = current_pos.zone_idx();
    let (curr_zx, curr_zy, _curr_zz) = zone_xyz(current_zone);
    let (target_zx, target_zy, target_zz) = zone_xyz(target_zone_idx);

    // Determine which edge of the target zone to teleport to based on current zone position
    let (new_x, new_y) = if target_zx > curr_zx {
        // Coming from west, teleport to west edge of target zone
        (target_zx * ZONE_SIZE.0, player_pos.1)
    } else if target_zx < curr_zx {
        // Coming from east, teleport to east edge of target zone
        ((target_zx + 1) * ZONE_SIZE.0 - 2, player_pos.1)
    } else if target_zy > curr_zy {
        // Coming from north, teleport to north edge of target zone
        (player_pos.0, target_zy * ZONE_SIZE.1)
    } else if target_zy < curr_zy {
        // Coming from south, teleport to south edge of target zone
        (player_pos.0, (target_zy + 1) * ZONE_SIZE.1 - 2)
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
        energy.consume_energy(500); // High energy cost for teleportation
    }

    println!("Teleportation successful to ({}, {})", new_x, new_y);
    true
}
