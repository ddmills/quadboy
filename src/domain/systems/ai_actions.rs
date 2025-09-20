use bevy_ecs::prelude::*;

use crate::{
    domain::{AiController, AiState, Energy, EnergyActionType, MoveAction, get_base_energy_cost},
    rendering::Position,
    tracy_span,
};

use super::ai_utils::*;

/// Try to attack any adjacent hostile entities
pub fn try_attack_adjacent(entity: Entity, entity_pos: &Position, world: &mut World) -> bool {
    tracy_span!("try_attack_adjacent");
    attack_if_adjacent(entity, entity_pos, world)
}

/// Try to pursue a specific target entity
pub fn try_pursue_target(
    entity: Entity,
    entity_pos: &Position,
    target: Entity,
    world: &mut World,
) -> bool {
    tracy_span!("try_pursue_target");
    if move_toward_target(entity, entity_pos, target, world) {
        update_ai_state(world, entity, AiState::Pursuing);
        update_ai_target(world, entity, Some(target));
        true
    } else {
        false
    }
}

/// Try to flee from a threat position
pub fn try_flee_from(
    entity: Entity,
    entity_pos: &Position,
    threat_pos: &Position,
    world: &mut World,
) -> bool {
    tracy_span!("try_flee_from");
    let flee_direction = calculate_flee_direction(entity_pos, threat_pos);
    if try_move_in_direction(entity, entity_pos, flee_direction, world) {
        update_ai_state(world, entity, AiState::Fleeing);
        true
    } else {
        false
    }
}

/// Try to wander near a specific position
pub fn try_wander_near(
    entity: Entity,
    entity_pos: &Position,
    target_pos: &Position,
    range: f32,
    world: &mut World,
) -> bool {
    tracy_span!("try_wander_near");
    if wander_near_point(entity, entity_pos, target_pos, range, world) {
        update_ai_state(world, entity, AiState::Wandering);
        true
    } else {
        false
    }
}

/// Try to return to home position
pub fn try_return_home(
    entity: Entity,
    entity_pos: &Position,
    ai: &AiController,
    world: &mut World,
) -> bool {
    tracy_span!("try_return_home");
    if return_to_home(entity, entity_pos, &ai.home_position, world) {
        update_ai_state(world, entity, AiState::Returning);
        true
    } else {
        false
    }
}

/// Set AI to idle state and consume wait energy
pub fn set_idle_and_wait(entity: Entity, world: &mut World) {
    tracy_span!("set_idle_and_wait");
    update_ai_state(world, entity, AiState::Idle);
    update_ai_target(world, entity, None);
    consume_wait_energy(entity, world);
}

/// Set AI to fighting state
pub fn set_fighting_state(entity: Entity, target: Option<Entity>, world: &mut World) {
    tracy_span!("set_fighting_state");
    update_ai_state(world, entity, AiState::Fighting);
    update_ai_target(world, entity, target);
}

// Helper functions that need to be accessible from ai_actions
pub fn update_ai_state(world: &mut World, entity: Entity, new_state: AiState) {
    if let Some(mut ai) = world.get_mut::<AiController>(entity) {
        ai.state = new_state;
    }
}

pub fn update_ai_target(world: &mut World, entity: Entity, new_target: Option<Entity>) {
    if let Some(mut ai) = world.get_mut::<AiController>(entity) {
        ai.current_target = new_target;
    }
}

fn calculate_flee_direction(entity_pos: &Position, threat_pos: &Position) -> (i32, i32) {
    let (entity_x, entity_y, _) = entity_pos.world();
    let (threat_x, threat_y, _) = threat_pos.world();

    // Calculate direction away from threat
    let dx = entity_x as i32 - threat_x as i32;
    let dy = entity_y as i32 - threat_y as i32;

    // Normalize to unit direction (prefer cardinal directions)
    let flee_x = if dx > 0 {
        1
    } else if dx < 0 {
        -1
    } else {
        0
    };
    let flee_y = if dy > 0 {
        1
    } else if dy < 0 {
        -1
    } else {
        0
    };

    // If we're exactly on the same position, pick a random direction
    if flee_x == 0 && flee_y == 0 {
        return (1, 0); // Default to east
    }

    (flee_x, flee_y)
}

fn try_move_in_direction(
    entity: Entity,
    entity_pos: &Position,
    direction: (i32, i32),
    world: &mut World,
) -> bool {
    let (current_x, current_y, current_z) = entity_pos.world();
    let new_x = (current_x as i32 + direction.0) as usize;
    let new_y = (current_y as i32 + direction.1) as usize;

    // Try the preferred direction first
    if is_move_valid_for_entity(world, Some(entity), new_x, new_y, current_z) {
        let move_action = MoveAction {
            entity,
            new_position: (new_x, new_y, current_z),
        };
        move_action.apply(world);
        return true;
    }

    // Try alternative directions if blocked
    let alternatives = [
        (direction.0, 0),            // Just X component
        (0, direction.1),            // Just Y component
        (-direction.0, direction.1), // Opposite X
        (direction.0, -direction.1), // Opposite Y
    ];

    for (dx, dy) in alternatives {
        let alt_x = (current_x as i32 + dx) as usize;
        let alt_y = (current_y as i32 + dy) as usize;

        if is_move_valid_for_entity(world, Some(entity), alt_x, alt_y, current_z) {
            let move_action = MoveAction {
                entity,
                new_position: (alt_x, alt_y, current_z),
            };
            move_action.apply(world);
            return true;
        }
    }

    false // Couldn't move in any direction
}
