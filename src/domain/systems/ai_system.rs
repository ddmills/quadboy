use bevy_ecs::prelude::*;
use macroquad::telemetry;

use crate::{
    domain::{
        AiController, AiState, AiTemplate, Energy, EnergyActionType, FactionId, FactionMember,
        Player, TurnState, get_base_energy_cost,
    },
    rendering::Position,
};

use super::ai_utils::*;

pub fn ai_turn(world: &mut World) {
    telemetry::begin_zone("ai_turn");

    let Some(turn_state) = world.get_resource::<TurnState>() else {
        telemetry::end_zone();
        return;
    };

    if turn_state.is_players_turn {
        telemetry::end_zone();
        return;
    }

    let Some(current_entity) = turn_state.current_turn_entity else {
        telemetry::end_zone();
        return;
    };

    let (position, ai_controller) = {
        let Some(position) = world.get::<Position>(current_entity) else {
            if let Some(mut energy) = world.get_mut::<Energy>(current_entity) {
                let cost = get_base_energy_cost(EnergyActionType::Wait);
                energy.consume_energy(cost);
            }
            telemetry::end_zone();
            return;
        };

        let Some(ai_controller) = world.get::<AiController>(current_entity) else {
            if let Some(mut energy) = world.get_mut::<Energy>(current_entity) {
                let cost = get_base_energy_cost(EnergyActionType::Wait);
                energy.consume_energy(cost);
            }
            telemetry::end_zone();
            return;
        };

        (position.clone(), ai_controller.clone())
    };

    process_ai_template(world, current_entity, &ai_controller, &position);

    telemetry::end_zone();
}

fn process_ai_template(
    world: &mut World,
    entity: Entity,
    ai: &AiController,
    entity_pos: &Position,
) {
    let home_distance = distance_from_home(entity_pos, &ai.home_position);

    if home_distance > ai.leash_range {
        if return_to_home(entity, entity_pos, &ai.home_position, world) {
            update_ai_state(world, entity, AiState::Returning);
            return;
        }
    }

    match ai.template {
        AiTemplate::BasicAggressive => {
            process_basic_aggressive(world, entity, ai, entity_pos);
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

    if let Some(hostile) = find_hostile_in_range(entity, entity_pos, ai.detection_range, world) {
        if move_toward_target(entity, entity_pos, hostile, world) {
            update_ai_state(world, entity, AiState::Pursuing);
            update_ai_target(world, entity, Some(hostile));
            return;
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
        return;
    }

    update_ai_state(world, entity, AiState::Idle);
    update_ai_target(world, entity, None);
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
