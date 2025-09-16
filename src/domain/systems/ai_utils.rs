use crate::{
    cfg::{MAP_SIZE, ZONE_SIZE},
    common::Rand,
    domain::{
        AttackAction, Collider, Energy, EnergyActionType, FactionMap, FactionMember, InActiveZone,
        Player, Zone,
        actions::MoveAction,
        are_hostile,
        components::ai_controller::{AiController, AiState},
        get_base_energy_cost,
    },
    rendering::{Position, world_to_zone_idx, world_to_zone_local},
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
    let Some(dijkstra_map) = faction_map.get_map(target_faction.faction_id) else {
        return false;
    };

    let (x, y, z) = entity_pos.world();
    let (local_x, local_y) = world_to_zone_local(x, y);

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

    false
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

pub fn distance_from_home(entity_pos: &Position, home_pos: &Position) -> f32 {
    let (x, y, _) = entity_pos.world();
    let (home_x, home_y, _) = home_pos.world();

    ((x as f32 - home_x as f32).powi(2) + (y as f32 - home_y as f32).powi(2)).sqrt()
}

pub fn consume_wait_energy(entity: Entity, world: &mut World) {
    if let Some(mut energy) = world.get_mut::<Energy>(entity) {
        let cost = get_base_energy_cost(EnergyActionType::Wait);
        energy.consume_energy(cost);
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
