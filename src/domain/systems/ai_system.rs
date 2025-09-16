use bevy_ecs::prelude::*;
use macroquad::telemetry;

use crate::{
    cfg::{MAP_SIZE, ZONE_SIZE},
    common::Rand,
    domain::{
        AiBehavior, AttackAction, Collider, Energy, EnergyActionType, FactionMap, FactionMember, TurnState, Zone, actions::MoveAction, are_hostile,
        get_base_energy_cost,
    },
    rendering::{Position, world_to_zone_idx, world_to_zone_local},
};

pub fn ai_turn(world: &mut World) {
    telemetry::begin_zone("ai_turn");

    // Get turn state
    let Some(turn_state) = world.get_resource::<TurnState>() else {
        telemetry::end_zone();
        return;
    };

    // Only run AI when it's not the player's turn
    if turn_state.is_players_turn {
        telemetry::end_zone();
        return;
    }

    let Some(current_entity) = turn_state.current_turn_entity else {
        telemetry::end_zone();
        return;
    };

    // Get the current entity's components (read-only first)
    let Some(position) = world.get::<Position>(current_entity) else {
        if let Some(mut energy) = world.get_mut::<Energy>(current_entity) {
            let cost = get_base_energy_cost(EnergyActionType::Wait);
            energy.consume_energy(cost);
        }
        telemetry::end_zone();
        return;
    };

    let (x, y, z) = position.world();

    // Get AI behavior for this entity
    let ai_behavior = world
        .get::<AiBehavior>(current_entity)
        .cloned()
        .unwrap_or_default();

    // Check if the entity is a bear with special AI
    match ai_behavior {
        AiBehavior::BearAi {
            mut aggressive,
            detection_range,
        } => {
            // Check if bear should be aggressive based on nearby hostile entities
            if !aggressive {
                let zone_idx = world_to_zone_idx(x, y, z);
                let mut zone_query = world.query::<&Zone>();
                if let Some(zone) = zone_query.iter(world).find(|zone| zone.idx == zone_idx) {
                    let (local_x, local_y) = world_to_zone_local(x, y);

                    // Check surrounding tiles for hostile entities
                    for dx in -2..=2 {
                        for dy in -2..=2 {
                            let check_x = (local_x as i32 + dx).max(0) as usize;
                            let check_y = (local_y as i32 + dy).max(0) as usize;

                            if let Some(entities) = zone.entities.get(check_x, check_y) {
                                for &entity in entities {
                                    if are_hostile(current_entity, entity, world) {
                                        aggressive = true;
                                        break;
                                    }
                                }
                            }
                            if aggressive {
                                break;
                            }
                        }
                        if aggressive {
                            break;
                        }
                    }
                }
            }

            if aggressive {
                // Update the entity's AI behavior state
                if let Some(mut behavior) = world.get_mut::<AiBehavior>(current_entity)
                    && let AiBehavior::BearAi {
                        aggressive: agg, ..
                    } = &mut *behavior
                    {
                        *agg = true;
                    }
            }

            bear_ai_logic(world, current_entity, aggressive, (x, y, z));
        }
        AiBehavior::Wander => {
            wander_ai_logic(world, current_entity, (x, y, z));
        }
    }

    telemetry::end_zone();
}

fn bear_ai_logic(world: &mut World, entity: Entity, aggressive: bool, pos: (usize, usize, usize)) {
    let (x, y, z) = pos;

    // Check for adjacent hostile entities to attack
    let directions = [(-1, 0), (1, 0), (0, -1), (0, 1)];
    for (dx, dy) in directions.iter() {
        let check_x = (x as i32 + dx) as usize;
        let check_y = (y as i32 + dy) as usize;

        // Get entities at this position and check if any are hostile
        let zone_idx = world_to_zone_idx(check_x, check_y, z);
        let mut zone_query = world.query::<&Zone>();
        if let Some(zone) = zone_query.iter(world).find(|zone| zone.idx == zone_idx) {
            let (local_x, local_y) = world_to_zone_local(check_x, check_y);
            if let Some(entities) = zone.entities.get(local_x, local_y) {
                for &target_entity in entities {
                    if are_hostile(entity, target_entity, world) {
                        // Found a hostile entity adjacent! Attack
                        let attack_action = AttackAction {
                            attacker_entity: entity,
                            target_pos: (check_x, check_y, z),
                            is_bump_attack: false,
                        };
                        attack_action.apply(world);
                        return;
                    }
                }
            }
        }
    }

    if aggressive {
        // Find nearby hostile entities and their factions, then pathfind to them
        let zone_idx = world_to_zone_idx(x, y, z);
        let mut zone_query = world.query::<&Zone>();
        if let Some(zone) = zone_query.iter(world).find(|zone| zone.idx == zone_idx) {
            let (local_x, local_y) = world_to_zone_local(x, y);
            let mut target_faction = None;

            // Find the nearest hostile entity and get its faction
            for dx in -5..=5 {
                for dy in -5..=5 {
                    let check_x = (local_x as i32 + dx).max(0) as usize;
                    let check_y = (local_y as i32 + dy).max(0) as usize;

                    if let Some(entities) = zone.entities.get(check_x, check_y) {
                        for &target_entity in entities {
                            if are_hostile(entity, target_entity, world)
                                && let Some(faction_member) =
                                    world.get::<FactionMember>(target_entity)
                                {
                                    target_faction = Some(faction_member.faction_id);
                                    break;
                                }
                        }
                        if target_faction.is_some() {
                            break;
                        }
                    }
                }
                if target_faction.is_some() {
                    break;
                }
            }

            // Use the appropriate faction map to pathfind to the target
            if let Some(faction_id) = target_faction
                && let Some(faction_map) = world.get_resource::<FactionMap>()
                    && let Some(dijkstra_map) = faction_map.get_map(faction_id)
                        && let Some((dx, dy)) = dijkstra_map.get_best_direction(local_x, local_y) {
                            let new_x = (x as i32 + dx) as usize;
                            let new_y = (y as i32 + dy) as usize;

                            if is_move_valid(world, new_x, new_y, z) {
                                let move_action = MoveAction {
                                    entity,
                                    new_position: (new_x, new_y, z),
                                };
                                move_action.apply(world);
                                return; // MoveAction handles energy consumption
                            }
                        }
        }
    } else {
        // Wander randomly
        let mut rand = Rand::new();
        if !rand.bool(0.75) {
            let (dx, dy) = rand.pick(&directions);
            let new_x = (x as i32 + dx) as usize;
            let new_y = (y as i32 + dy) as usize;

            if is_move_valid(world, new_x, new_y, z) {
                let move_action = MoveAction {
                    entity,
                    new_position: (new_x, new_y, z),
                };
                move_action.apply(world);
                return; // MoveAction handles energy consumption
            }
        }
    }

    // Consume wait energy if we didn't move or attack
    if let Some(mut energy) = world.get_mut::<Energy>(entity) {
        let cost = get_base_energy_cost(EnergyActionType::Wait);
        energy.consume_energy(cost);
    }
}

fn wander_ai_logic(world: &mut World, entity: Entity, pos: (usize, usize, usize)) {
    let (x, y, z) = pos;

    // First check for adjacent hostile entities to attack
    let directions = [(-1, 0), (1, 0), (0, -1), (0, 1)];
    for (dx, dy) in directions.iter() {
        let check_x = (x as i32 + dx) as usize;
        let check_y = (y as i32 + dy) as usize;

        // Get entities at this position and check if any are hostile
        let zone_idx = world_to_zone_idx(check_x, check_y, z);
        let mut zone_query = world.query::<&Zone>();
        if let Some(zone) = zone_query.iter(world).find(|zone| zone.idx == zone_idx) {
            let (local_x, local_y) = world_to_zone_local(check_x, check_y);
            if let Some(entities) = zone.entities.get(local_x, local_y) {
                for &target_entity in entities {
                    if are_hostile(entity, target_entity, world) {
                        // Found a hostile entity adjacent! Attack
                        let attack_action = AttackAction {
                            attacker_entity: entity,
                            target_pos: (check_x, check_y, z),
                            is_bump_attack: false,
                        };
                        attack_action.apply(world);
                        return;
                    }
                }
            }
        }
    }

    // If no hostile entities to attack, do normal wander behavior
    let mut rand = Rand::new();

    // 75% chance to wait, 25% chance to move
    if !rand.bool(0.75) {
        let (dx, dy) = rand.pick(&directions);
        let new_x = (x as i32 + dx) as usize;
        let new_y = (y as i32 + dy) as usize;

        if is_move_valid(world, new_x, new_y, z) {
            let move_action = MoveAction {
                entity,
                new_position: (new_x, new_y, z),
            };
            move_action.apply(world);
            return; // MoveAction handles energy consumption
        }
    }

    // If not moving, consume wait energy
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

    // Check if destination zone is loaded
    let dest_zone_idx = world_to_zone_idx(new_x, new_y, z);

    let (local_x, local_y) = world_to_zone_local(new_x, new_y);

    // First check if zone exists and get entities at position
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

    // Then check for colliders
    if let Some(entities) = entities_at_pos {
        let mut collider_query = world.query::<&Collider>();
        for entity_at_pos in entities {
            if collider_query.get(world, entity_at_pos).is_ok() {
                // Found a collider at this position, can't move
                return false;
            }
        }
    }

    // Move is valid
    true
}
