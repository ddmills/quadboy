use bevy_ecs::{prelude::*, system::RunSystemOnce};
use macroquad::prelude::trace;
use rand;

use crate::{
    domain::{
        ActiveConditions, AiController, AiState, AiTemplate, Condition, ConditionSource, ConditionType, Energy,
        EnergyActionType, Health, MoveAction, PlayerPosition, PursuingTarget, TurnState,
        get_base_energy_cost,
    },
    engine::{Clock, StableId, StableIdRegistry},
    rendering::Position,
    tracy_span,
};

use super::ai_actions::*;
use super::ai_utils::*;
use super::condition_system::apply_condition_to_entity;

pub fn ai_turn(world: &mut World) {
    tracy_span!("ai_turn");

    let Some(turn_state) = world.get_resource::<TurnState>() else {
        return;
    };

    if turn_state.is_players_turn {
        return;
    }

    let Some(current_entity) = turn_state.current_turn_entity else {
        return;
    };

    let (position, ai_controller) = {
        tracy_span!("ai_turn_get_components");
        let Some(position) = world.get::<Position>(current_entity) else {
            if let Some(mut energy) = world.get_mut::<Energy>(current_entity) {
                let cost = get_base_energy_cost(EnergyActionType::Wait);
                energy.consume_energy(cost);
            }
            return;
        };

        let Some(ai_controller) = world.get::<AiController>(current_entity) else {
            if let Some(mut energy) = world.get_mut::<Energy>(current_entity) {
                let cost = get_base_energy_cost(EnergyActionType::Wait);
                energy.consume_energy(cost);
            }
            return;
        };

        (position.clone(), ai_controller.clone())
    };

    {
        tracy_span!("ai_turn_process_template");
        process_ai_template(world, current_entity, &ai_controller, &position);
    }
}

fn process_ai_template(
    world: &mut World,
    entity: Entity,
    ai: &AiController,
    entity_pos: &Position,
) {
    {
        tracy_span!("ai_check_returning_state");
        // If AI is returning home, continue returning until home is reached
        if ai.state == AiState::Returning {
            // Apply or refresh ReturningHome condition while returning
            {
                tracy_span!("ai_apply_returning_home_condition");
                let returning_condition = ConditionType::ReturningHome {
                    health_regen_per_tick: 2,
                    armor_regen_multiplier: 3.0,
                    tick_interval: 25,
                };

                // Check if condition needs to be applied or refreshed
                let needs_condition = if let Some(conditions) = world.get::<ActiveConditions>(entity) {
                    // Check if we need to apply or refresh the condition
                    let existing_condition = conditions.conditions.iter().find(|c| {
                        matches!(&c.condition_type, ConditionType::ReturningHome { .. })
                    });

                    match existing_condition {
                        None => true, // No condition exists, need to apply
                        Some(condition) => {
                            // Refresh if condition has less than 200 ticks remaining
                            condition.duration_remaining < 200
                        }
                    }
                } else {
                    true
                };

                if needs_condition {
                    let condition = Condition::new(
                        returning_condition.clone(),
                        returning_condition.get_base_duration_ticks(),  // Use base duration from conditions.rs
                        1.0,
                        ConditionSource::environment(),
                    );

                    let _ = apply_condition_to_entity(entity, condition, world);
                }
            }

            // Check if we've reached home
            let (entity_x, entity_y, entity_z) = entity_pos.world();
            let (home_x, home_y, home_z) = ai.home_position.world();

            if entity_x == home_x && entity_y == home_y && entity_z == home_z {
                // Reached home, switch to idle state
                update_ai_state(world, entity, AiState::Idle);
                update_ai_target(world, entity, None);
                clear_cached_path(world, entity);
                remove_pursuing_target_component(world, entity);

                // Extend the ReturningHome condition for 250 ticks for recovery at home
                if let Some(mut conditions) = world.get_mut::<ActiveConditions>(entity) {
                    for condition in &mut conditions.conditions {
                        if matches!(&condition.condition_type, ConditionType::ReturningHome { .. }) {
                            condition.duration_remaining = 250; // 250 ticks of recovery time at home
                            break;
                        }
                    }
                }

                consume_wait_energy(entity, world);
                return;
            }

            // Continue returning home
            if return_to_home(entity, entity_pos, &ai.home_position, world) {
                // Successfully moved toward home, stay in Returning state
                return;
            } else {
                // Couldn't move toward home (blocked), wait
                consume_wait_energy(entity, world);
                return;
            }
        }
    }

    {
        tracy_span!("ai_check_leash_range");
        let home_distance = {
            tracy_span!("ai_calculate_home_distance");
            distance_from_home(entity_pos, &ai.home_position)
        };

        if home_distance > ai.leash_range {
            {
                tracy_span!("ai_setup_returning_state");
                // Always set to Returning state when beyond leash range
                if ai.state != AiState::Returning {
                    clear_cached_path(world, entity);
                }
                update_ai_state(world, entity, AiState::Returning);
                remove_pursuing_target_component(world, entity);
            }


            {
                tracy_span!("ai_return_to_home");
                // Try to move toward home
                if !return_to_home(entity, entity_pos, &ai.home_position, world) {
                    // If can't move (blocked), consume wait energy and stay in Returning state
                    consume_wait_energy(entity, world);
                }
            }

            return;
        }
    }

    {
        tracy_span!("ai_check_conditions");
        // Check for condition-based behavior overrides
        let conditions = world.get::<ActiveConditions>(entity).cloned();
        if let Some(conditions) = conditions {
            if process_condition_behaviors(world, entity, ai, entity_pos, &conditions) {
                // Condition behavior took precedence, skip normal AI processing
                return;
            }
        }
    }

    {
        tracy_span!("ai_process_template");
        match ai.template {
            AiTemplate::BasicAggressive => {
                process_basic_aggressive(world, entity, ai, entity_pos);
            }
            AiTemplate::Timid => {
                process_timid(world, entity, ai, entity_pos);
            }
            AiTemplate::Scavenger => {
                process_scavenger(world, entity, ai, entity_pos);
            }
            AiTemplate::Ambush { strike_range } => {
                process_ambush(world, entity, ai, entity_pos, strike_range);
            }
        }
    }
}

fn process_basic_aggressive(
    world: &mut World,
    entity: Entity,
    ai: &AiController,
    entity_pos: &Position,
) {
    if try_attack_adjacent(entity, entity_pos, world) {
        return;
    }

    if let Some(hostile) = find_hostile_in_range(entity, entity_pos, ai.detection_range, world) {
        if try_pursue_target(entity, entity_pos, hostile, world) {
            add_pursuing_target_component(world, entity, hostile);
            clear_cached_pursuit_path(world, entity); // Clear cached path when starting new pursuit
            return;
        }
    }

    // Check if already pursuing - continue toward last known position
    if let Some(pursuing) = world.get::<PursuingTarget>(entity) {
        let clock = world.get_resource::<Clock>().unwrap();
        let current_tick = clock.current_tick();
        let pursuing_clone = pursuing.clone();

        // Normal pursuit logic
        let last_seen_at = pursuing_clone.last_seen_at;
        let last_seen_pos = Position::new(last_seen_at.0, last_seen_at.1, last_seen_at.2);

        // Check if we've reached the last known position first
        if entity_pos.world() == last_seen_at {
            if let Some(mut pursuing) = world.get_mut::<PursuingTarget>(entity) {
                if !pursuing.searching_at_last_position {
                    pursuing.start_searching(current_tick);
                    update_ai_state(world, entity, AiState::Pursuing); // Keep pursuing while searching
                    return;
                }

                // Already searching - check if we should stop
                if pursuing.should_stop_searching(current_tick) {
                    // Search timeout - go to Returning state to head home
                    remove_pursuing_target_component(world, entity);
                    clear_cached_pursuit_path(world, entity); // Clear pursuit path when giving up
                    update_ai_state(world, entity, AiState::Returning);
                    return;
                }

                // Continue searching - wander around last known position
                let last_known_pos = Position::new(last_seen_at.0, last_seen_at.1, last_seen_at.2);
                if wander_near_point(entity, entity_pos, &last_known_pos, ai.wander_range, world) {
                    update_ai_state(world, entity, AiState::Pursuing); // Still pursuing/searching
                    return;
                } else {
                    // Can't wander but still searching - wait
                    consume_wait_energy(entity, world);
                    update_ai_state(world, entity, AiState::Pursuing);
                    return;
                }
            }
        } else {
            // Not at last known position - try to move toward it
            if move_toward_last_known_position(entity, entity_pos, &last_seen_pos, world) {
                update_ai_state(world, entity, AiState::Pursuing);
                return;
            } else {
                // Can't move toward last known position (blocked) - wait but keep pursuing
                consume_wait_energy(entity, world);
                update_ai_state(world, entity, AiState::Pursuing);
                return;
            }
        }
    }

    // If no pursuit component, try normal wandering near home
    if try_wander_near(
        entity,
        entity_pos,
        &ai.home_position,
        ai.wander_range,
        world,
    ) {
        update_ai_target(world, entity, None);
        return;
    }

    set_idle_and_wait(entity, world);
    update_ai_target(world, entity, None);
}

fn process_timid(world: &mut World, entity: Entity, ai: &AiController, entity_pos: &Position) {
    // Timid creatures flee from hostiles and never attack
    if let Some(hostile) = find_hostile_in_range(entity, entity_pos, ai.detection_range, world) {
        // Get hostile position for fleeing
        if let Some(hostile_pos) = world.get::<Position>(hostile).cloned() {
            if try_flee_from(entity, entity_pos, &hostile_pos, world) {
                update_ai_target(world, entity, Some(hostile));
                return;
            } else {
                // Can't flee, just wait and hope for the best
                set_idle_and_wait(entity, world);
                update_ai_target(world, entity, Some(hostile));
                return;
            }
        }
    }

    // If no threat, wander peacefully near home
    if try_wander_near(
        entity,
        entity_pos,
        &ai.home_position,
        ai.wander_range,
        world,
    ) {
        update_ai_target(world, entity, None);
        return;
    }

    // Can't wander, just idle
    set_idle_and_wait(entity, world);
    update_ai_target(world, entity, None);
}

fn process_scavenger(world: &mut World, entity: Entity, ai: &AiController, entity_pos: &Position) {
    // Scavengers attack adjacent wounded targets first
    if try_attack_adjacent(entity, entity_pos, world) {
        return;
    }

    // Look for wounded hostiles to attack aggressively
    if let Some(wounded_hostile) =
        find_wounded_hostile_in_range(entity, entity_pos, ai.detection_range, world)
    {
        if try_pursue_target(entity, entity_pos, wounded_hostile, world) {
            add_pursuing_target_component(world, entity, wounded_hostile);
            clear_cached_pursuit_path(world, entity);
            return;
        }
    }

    // Look for healthy hostiles to follow at a safe distance
    if let Some(healthy_hostile) =
        find_hostile_in_range(entity, entity_pos, ai.detection_range, world)
    {
        let safe_distance = 5.0; // Tiles to maintain from healthy targets

        if let Some(target_pos) = world.get::<Position>(healthy_hostile).cloned() {
            let (entity_x, entity_y, entity_z) = entity_pos.world();
            let (target_x, target_y, _) = target_pos.world();

            let distance = {
                let dx = entity_x as f32 - target_x as f32;
                let dy = entity_y as f32 - target_y as f32;
                (dx * dx + dy * dy).sqrt()
            };

            // If too close to healthy target, move away
            if distance < safe_distance {
                if try_flee_from(entity, entity_pos, &target_pos, world) {
                    update_ai_target(world, entity, Some(healthy_hostile));
                    return;
                }
            }
            // If too far, move closer but maintain safe distance
            else if distance > ai.detection_range * 0.8 {
                if move_toward_target(entity, entity_pos, healthy_hostile, world) {
                    update_ai_state(world, entity, AiState::Wandering);
                    update_ai_target(world, entity, Some(healthy_hostile));
                    return;
                }
            }
            // At good distance, just watch
            else {
                update_ai_state(world, entity, AiState::Wandering);
                update_ai_target(world, entity, Some(healthy_hostile));
                consume_wait_energy(entity, world);
                return;
            }
        }
    }

    // No targets found, wander near home
    if try_wander_near(
        entity,
        entity_pos,
        &ai.home_position,
        ai.wander_range,
        world,
    ) {
        update_ai_target(world, entity, None);
        return;
    }

    // Can't wander, just idle
    set_idle_and_wait(entity, world);
    update_ai_target(world, entity, None);
}

fn process_ambush(
    world: &mut World,
    entity: Entity,
    ai: &AiController,
    entity_pos: &Position,
    strike_range: f32,
) {
    // Ambush predators attack adjacent targets first
    if try_attack_adjacent(entity, entity_pos, world) {
        return;
    }

    // Look for hostiles within strike range
    if let Some(hostile) = find_hostile_in_range(entity, entity_pos, strike_range, world) {
        // Calculate distance to target
        if let Some(target_pos) = world.get::<Position>(hostile) {
            let (entity_x, entity_y, _) = entity_pos.world();
            let (target_x, target_y, _) = target_pos.world();

            let distance = {
                let dx = entity_x as f32 - target_x as f32;
                let dy = entity_y as f32 - target_y as f32;
                (dx * dx + dy * dy).sqrt()
            };

            // If target is within strike range - 1, attempt a quick strike
            if distance <= (strike_range - 1.0).max(1.0) {
                if try_pursue_target(entity, entity_pos, hostile, world) {
                    return;
                }
            }
            // If target is exactly at strike range, prepare to strike but don't move
            else if distance <= strike_range {
                update_ai_state(world, entity, AiState::Waiting); // Alert/coiled state
                update_ai_target(world, entity, Some(hostile));
                consume_wait_energy(entity, world);
                return;
            }
        }
    }

    // No targets in strike range - return to idle ambush position
    set_idle_and_wait(entity, world);
    update_ai_target(world, entity, None);
}

fn add_pursuing_target_component(world: &mut World, entity: Entity, target_entity: Entity) {
    let clock = world.get_resource::<Clock>().unwrap();
    let current_tick = clock.current_tick();

    // Get target position
    let target_pos = if let Some(pos) = world.get::<Position>(target_entity) {
        pos.world()
    } else {
        // Fallback to player position resource
        if let Some(player_pos) = world.get_resource::<PlayerPosition>() {
            (
                player_pos.x as usize,
                player_pos.y as usize,
                player_pos.z as usize,
            )
        } else {
            return; // Can't track without position
        }
    };

    if let Some(mut pursuing) = world.get_mut::<PursuingTarget>(entity) {
        // Update last seen position if already pursuing
        pursuing.update_last_seen(target_pos);
    } else {
        // Create new pursuing component
        let pursuing = PursuingTarget::new(target_pos, current_tick);
        world.entity_mut(entity).insert(pursuing);
    }
}

fn remove_pursuing_target_component(world: &mut World, entity: Entity) {
    clear_cached_pursuit_path(world, entity); // Clear pursuit path when removing pursuit component
    world.entity_mut(entity).remove::<PursuingTarget>();
}

pub fn manage_pursuit_timeout(world: &mut World) {
    tracy_span!("manage_pursuit_timeout");
    let clock = world.get_resource::<Clock>().unwrap();
    let current_tick = clock.current_tick();

    let mut entities_to_remove: Vec<Entity> = Vec::new();

    let mut q_pursuing = world.query::<(Entity, &PursuingTarget)>();
    for (entity, pursuing) in q_pursuing.iter(world) {
        let pursuit_duration = pursuing.pursuit_duration(current_tick);

        // if pursuit_duration > 1000 {
        //     entities_to_remove.push(entity);
        // }
    }

    for entity in entities_to_remove {
        world.entity_mut(entity).remove::<PursuingTarget>();

        if let Some(mut ai) = world.get_mut::<AiController>(entity) {
            ai.state = AiState::Idle;
            ai.current_target = None;
        }
    }
}

fn clear_cached_path(world: &mut World, entity: Entity) {
    if let Some(mut ai) = world.get_mut::<AiController>(entity) {
        ai.cached_home_path = None;
    }
}

fn clear_cached_pursuit_path(world: &mut World, entity: Entity) {
    if let Some(mut ai) = world.get_mut::<AiController>(entity) {
        ai.cached_pursuit_path = None;
    }
}

/// Process condition-based AI behavior overrides
/// Returns true if a condition behavior was processed (and normal AI should be skipped)
fn process_condition_behaviors(
    world: &mut World,
    entity: Entity,
    ai: &AiController,
    entity_pos: &Position,
    conditions: &ActiveConditions,
) -> bool {
    tracy_span!("process_condition_behaviors");

    // Check for highest priority conditions first
    for condition in &conditions.conditions {
        match &condition.condition_type {
            ConditionType::Stunned => {
                // Stunned entities can't act at all
                consume_wait_energy(entity, world);
                return true;
            }

            ConditionType::Feared {
                flee_from,
                min_distance,
            } => {
                if let Some(target_entity) = get_entity_by_stable_id(world, *flee_from) {
                    let target_pos = if let Some(pos) = world.get::<Position>(target_entity) {
                        pos.clone()
                    } else {
                        return false; // Can't find target position
                    };

                    let distance = calculate_distance(entity_pos, &target_pos);

                    if distance < *min_distance {
                        // Need to flee - move away from the feared entity
                        if flee_from_target(world, entity, entity_pos, &target_pos) {
                            return true; // Successfully fled
                        } else {
                            // Can't flee, wait in terror
                            consume_wait_energy(entity, world);
                            return true;
                        }
                    }
                }
                // Fear target not found or far enough away, continue with normal behavior
            }

            ConditionType::Taunted {
                move_toward,
                force_target,
            } => {
                if let Some(target_entity) = get_entity_by_stable_id(world, *move_toward) {
                    let target_pos = if let Some(pos) = world.get::<Position>(target_entity) {
                        pos.clone()
                    } else {
                        return false; // Can't find target position
                    };

                    // Force the AI to move toward the taunting entity
                    if *force_target {
                        // Override normal target selection
                        update_ai_target(world, entity, Some(target_entity));
                        update_ai_state(world, entity, AiState::Pursuing);
                    }

                    // Move toward the taunter
                    if move_toward_target_pos(world, entity, entity_pos, &target_pos) {
                        return true; // Successfully moved toward taunter
                    } else {
                        // Can't move toward taunter, wait
                        consume_wait_energy(entity, world);
                        return true;
                    }
                }
                // Taunt target not found, continue with normal behavior
            }

            ConditionType::Confused { random_chance } => {
                // Random chance to act randomly instead of normally
                if rand::random::<f32>() < *random_chance {
                    // Take a random action
                    if perform_random_action(world, entity, entity_pos) {
                        return true; // Took random action
                    }
                }
                // Not confused this turn or random action failed, continue normally
            }

            _ => {
                // Other conditions don't override AI behavior
            }
        }
    }

    false // No condition behavior took precedence
}

/// Helper function to get an entity by its StableId
fn get_entity_by_stable_id(world: &World, stable_id: StableId) -> Option<Entity> {
    if let Some(registry) = world.get_resource::<StableIdRegistry>() {
        registry.get_entity(stable_id.0)
    } else {
        None
    }
}

/// Calculate distance between two positions
fn calculate_distance(pos1: &Position, pos2: &Position) -> f32 {
    let dx = pos1.x - pos2.x;
    let dy = pos1.y - pos2.y;
    let dz = pos1.z - pos2.z;
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Move away from a target position (for Fear)
fn flee_from_target(
    world: &mut World,
    entity: Entity,
    entity_pos: &Position,
    target_pos: &Position,
) -> bool {
    tracy_span!("flee_from_target");

    // Calculate direction away from target
    let dx = entity_pos.x - target_pos.x;
    let dy = entity_pos.y - target_pos.y;

    // Normalize and pick the strongest direction to flee
    let flee_x = if dx.abs() > dy.abs() {
        if dx > 0.0 { 1 } else { -1 }
    } else {
        0
    };

    let flee_y = if dy.abs() > dx.abs() {
        if dy > 0.0 { 1 } else { -1 }
    } else {
        0
    };

    // Try to move in the flee direction
    let (current_x, current_y, current_z) = entity_pos.world();
    let new_x = (current_x as i32 + flee_x).max(0) as usize;
    let new_y = (current_y as i32 + flee_y).max(0) as usize;
    let new_z = current_z;

    // Use the simple movement helper
    simple_move_to_position(world, entity, (new_x, new_y, new_z))
}

/// Move toward a target position (for Taunt)
fn move_toward_target_pos(
    world: &mut World,
    entity: Entity,
    entity_pos: &Position,
    target_pos: &Position,
) -> bool {
    tracy_span!("move_toward_target_pos");

    // Calculate direction toward target
    let dx = target_pos.x - entity_pos.x;
    let dy = target_pos.y - entity_pos.y;

    // Normalize and pick the strongest direction to move
    let move_x = if dx.abs() > dy.abs() {
        if dx > 0.0 { 1 } else { -1 }
    } else {
        0
    };

    let move_y = if dy.abs() > dx.abs() {
        if dy > 0.0 { 1 } else { -1 }
    } else {
        0
    };

    // Try to move toward the target
    let (current_x, current_y, current_z) = entity_pos.world();
    let new_x = (current_x as i32 + move_x).max(0) as usize;
    let new_y = (current_y as i32 + move_y).max(0) as usize;
    let new_z = current_z;

    // Use the simple movement helper
    simple_move_to_position(world, entity, (new_x, new_y, new_z))
}

/// Perform a random action (for Confusion)
fn perform_random_action(world: &mut World, entity: Entity, entity_pos: &Position) -> bool {
    tracy_span!("perform_random_action");

    let actions = [
        (1, 0),  // East
        (-1, 0), // West
        (0, 1),  // South
        (0, -1), // North
        (0, 0),  // Wait
    ];

    let random_index = (rand::random::<f32>() * actions.len() as f32).floor() as usize;
    let random_action = actions[random_index];

    if random_action == (0, 0) {
        // Random wait
        consume_wait_energy(entity, world);
        true
    } else {
        // Random movement
        let (current_x, current_y, current_z) = entity_pos.world();
        let new_x = (current_x as i32 + random_action.0).max(0) as usize;
        let new_y = (current_y as i32 + random_action.1).max(0) as usize;
        let new_z = current_z;

        simple_move_to_position(world, entity, (new_x, new_y, new_z))
    }
}

/// Simple movement helper that uses the existing MoveAction command
fn simple_move_to_position(
    world: &mut World,
    entity: Entity,
    new_position: (usize, usize, usize),
) -> bool {
    tracy_span!("simple_move_to_position");

    // Check if the new position is valid using existing utility
    if !is_move_valid(world, new_position.0, new_position.1, new_position.2) {
        return false;
    }

    // Execute the move using MoveAction
    let move_cmd = MoveAction {
        entity,
        new_position,
    };

    // Directly execute the MoveAction
    move_cmd.apply(world);

    true
}
