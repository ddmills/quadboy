use bevy_ecs::prelude::*;

use crate::{
    common::algorithm::distance::Distance,
    domain::{
        AiController, AiState, AiTemplate, Energy, EnergyActionType, Health, PlayerPosition,
        PursuingPlayer, Stats, TurnState, get_base_energy_cost, get_energy_cost,
    },
    engine::Clock,
    rendering::Position,
    tracy_span,
};

use super::ai_utils::*;

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
            // Check if we've reached home
            let (entity_x, entity_y, entity_z) = entity_pos.world();
            let (home_x, home_y, home_z) = ai.home_position.world();

            if entity_x == home_x && entity_y == home_y && entity_z == home_z {
                // Reached home, switch to idle state
                update_ai_state(world, entity, AiState::Idle);
                update_ai_target(world, entity, None);
                clear_cached_path(world, entity);
                remove_pursuing_player_component(world, entity);
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
                remove_pursuing_player_component(world, entity);
            }

            {
                tracy_span!("ai_enable_armor_regen");
                // Enable armor regeneration while returning home
                if let Some(mut health) = world.get_mut::<Health>(entity) {
                    health.last_damage_tick = 0; // Reset to enable immediate armor regen
                }
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
        tracy_span!("ai_process_template");
        match ai.template {
            AiTemplate::BasicAggressive => {
                process_basic_aggressive(world, entity, ai, entity_pos);
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
    if attack_if_adjacent(entity, entity_pos, world) {
        update_ai_state(world, entity, AiState::Fighting);
        return;
    }

    if let Some(hostile) = find_hostile_in_range(entity, entity_pos, ai.detection_range, world)
        && move_toward_target(entity, entity_pos, hostile, world) {
            update_ai_state(world, entity, AiState::Pursuing);
            update_ai_target(world, entity, Some(hostile));
            add_pursuing_player_component(world, entity, hostile);
            return;
        }

    // Check if already pursuing - continue toward last known position
    if let Some(pursuing) = world.get::<PursuingPlayer>(entity) {
        let clock = world.get_resource::<Clock>().unwrap();
        let current_tick = clock.current_tick();
        let pursuing_clone = pursuing.clone();

        // Check if waiting to teleport
        if pursuing_clone.waiting_to_teleport {
            if pursuing_clone.should_teleport(current_tick) {
                println!(
                    "Wait period elapsed for entity {:?}, attempting teleportation",
                    entity
                );
                // Get current player zone for teleportation
                let player_zone = if let Some(player_pos) = world.get_resource::<PlayerPosition>() {
                    player_pos.zone_idx()
                } else {
                    pursuing_clone.target_zone
                };

                // Attempt teleportation to player's current zone
                if teleport_to_zone_edge(entity, entity_pos, player_zone, world) {
                    if let Some(mut pursuing) = world.get_mut::<PursuingPlayer>(entity) {
                        pursuing.target_zone = player_zone; // Update target zone
                        pursuing.reset_teleport_wait();
                    }
                    update_ai_state(world, entity, AiState::Pursuing);
                    return;
                } else {
                    println!("Teleportation failed for entity {:?}", entity);
                }
            } else {
                // Still waiting
                consume_wait_energy(entity, world);
                update_ai_state(world, entity, AiState::Waiting);
                return;
            }
        }

        // Get current player zone for teleportation target
        let player_zone = if let Some(player_pos) = world.get_resource::<PlayerPosition>() {
            player_pos.zone_idx()
        } else {
            pursuing_clone.target_zone
        };

        // Check if entity is in different zone from player - start teleport wait
        if entity_pos.zone_idx() != player_zone {
            let wait_duration = calculate_teleport_wait_time(
                world,
                entity,
                entity_pos.world(),
                pursuing_clone.last_seen_at,
            );

            // Start teleport wait
            println!(
                "Starting teleport wait for entity {:?} - player in different zone, wait time: {}",
                entity, wait_duration
            );
            if let Some(mut pursuing) = world.get_mut::<PursuingPlayer>(entity) {
                pursuing.target_zone = player_zone; // Update to current player zone
                pursuing.start_teleport_wait(current_tick, wait_duration);
            }
            update_ai_state(world, entity, AiState::Waiting);
            consume_wait_energy(entity, world);
            return;
        }

        // Normal pursuit logic
        let last_seen_at = pursuing_clone.last_seen_at;
        let last_seen_pos = Position::new(last_seen_at.0, last_seen_at.1, last_seen_at.2);
        if move_toward_last_known_position(entity, entity_pos, &last_seen_pos, world) {
            update_ai_state(world, entity, AiState::Pursuing);
            return;
        }
        // If we reached last known position, stop pursuing
        if entity_pos.world() == last_seen_at {
            remove_pursuing_player_component(world, entity);
        }
    }

    if wander_near_point(
        entity,
        entity_pos,
        &ai.home_position,
        ai.wander_range,
        world,
    ) {
        update_ai_state(world, entity, AiState::Wandering);
        update_ai_target(world, entity, None);
        remove_pursuing_player_component(world, entity);
        return;
    }

    update_ai_state(world, entity, AiState::Idle);
    update_ai_target(world, entity, None);
    remove_pursuing_player_component(world, entity);
    consume_wait_energy(entity, world);
}

fn update_ai_state(world: &mut World, entity: Entity, new_state: AiState) {
    if let Some(mut ai) = world.get_mut::<AiController>(entity) {
        ai.state = new_state;
    }
}

fn update_ai_target(world: &mut World, entity: Entity, new_target: Option<Entity>) {
    if let Some(mut ai) = world.get_mut::<AiController>(entity) {
        ai.current_target = new_target;
    }
}

fn add_pursuing_player_component(world: &mut World, entity: Entity, target_entity: Entity) {
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

    if let Some(mut pursuing) = world.get_mut::<PursuingPlayer>(entity) {
        // Update last seen position if already pursuing
        pursuing.update_last_seen(target_pos);
    } else {
        // Create new pursuing component
        let pursuing = PursuingPlayer::new(target_pos, current_tick);
        world.entity_mut(entity).insert(pursuing);
    }
}

fn remove_pursuing_player_component(world: &mut World, entity: Entity) {
    world.entity_mut(entity).remove::<PursuingPlayer>();
}

pub fn manage_pursuit_timeout(world: &mut World) {
    tracy_span!("manage_pursuit_timeout");
    let clock = world.get_resource::<Clock>().unwrap();
    let current_tick = clock.current_tick();

    let mut entities_to_remove: Vec<Entity> = Vec::new();

    let mut q_pursuing = world.query::<(Entity, &PursuingPlayer)>();
    for (entity, pursuing) in q_pursuing.iter(world) {
        let pursuit_duration = pursuing.pursuit_duration(current_tick);

        if pursuit_duration > 1000 {
            entities_to_remove.push(entity);
        }
    }

    for entity in entities_to_remove {
        world.entity_mut(entity).remove::<PursuingPlayer>();

        if let Some(mut ai) = world.get_mut::<AiController>(entity) {
            ai.state = AiState::Idle;
            ai.current_target = None;
        }
    }
}

fn calculate_teleport_wait_time(
    world: &mut World,
    entity: Entity,
    from: (usize, usize, usize),
    to: (usize, usize, usize),
) -> u32 {
    let stats = world.get::<Stats>(entity);
    let movement_cost = get_energy_cost(EnergyActionType::Move, stats) as u32;

    let distance = Distance::chebyshev(
        [from.0 as i32, from.1 as i32, from.2 as i32],
        [to.0 as i32, to.1 as i32, to.2 as i32],
    )
    .ceil() as u32;

    let base_wait = movement_cost * distance;
    base_wait.max(100)
}

fn clear_cached_path(world: &mut World, entity: Entity) {
    if let Some(mut ai) = world.get_mut::<AiController>(entity) {
        ai.cached_home_path = None;
    }
}
