use bevy_ecs::prelude::*;
use macroquad::prelude::trace;
use quadboy_macros::profiled_system;

use crate::domain::{
    Actor, AiController, Energy, EnergyActionType, Health, TurnState, ai_try_attacking_nearby,
    ai_try_move_toward_target, ai_try_select_target, detect_actors, get_actor,
    get_base_energy_cost, try_handle_conditions,
};

#[derive(Default)]
pub struct AiContext {
    pub detected: Vec<Actor>,
    pub target: Option<Actor>,
}

impl AiContext {
    pub fn nearest_hostile(&self) -> Option<&Actor> {
        self.detected
            .iter()
            .filter(|x| x.relationship < 0)
            .min_by_key(|x| x.distance.round() as u32)
    }
}

#[profiled_system]
pub fn ai_turn(world: &mut World) {
    let Some(turn_state) = world.get_resource::<TurnState>() else {
        return;
    };

    if turn_state.is_players_turn {
        return;
    }

    let Some(current_entity) = turn_state.current_turn_entity else {
        return;
    };

    let mut context = build_ai_context(world, current_entity);

    if try_handle_conditions(world, current_entity, &mut context) {
        return;
    }

    if ai_try_attacking_nearby(world, current_entity, &mut context) {
        return;
    }

    if ai_try_select_target(world, current_entity, &mut context) {
        if let Some(mut ai_controller) = world.get_mut::<AiController>(current_entity) {
            ai_controller.current_target_id = context.target.map(|x| x.stable_id);
        };

        if ai_try_move_toward_target(world, current_entity, &mut context) {
            return;
        }

        // we have a target, but we can't move toward it!
        trace!("AI: Can't reach target!");
    }

    if let Some(mut energy) = world.get_mut::<Energy>(current_entity) {
        let cost = get_base_energy_cost(EnergyActionType::Wait);
        energy.consume_energy(cost);
    }
}

pub fn build_ai_context(world: &mut World, entity: Entity) -> AiContext {
    let detected = detect_actors(world, entity);
    let Some(ai_controller) = world.get::<AiController>(entity) else {
        return AiContext::default();
    };

    let mut target = if let Some(target_id) = ai_controller.current_target_id {
        get_actor(world, entity, target_id)
    } else {
        None
    };

    // Check if this AI has been attacked and doesn't have a current target
    if target.is_none() {
        if let Some(health) = world.get::<Health>(entity) {
            if let Some(attacker_id) = health.last_damage_source {
                // Try to target the attacker, even if they're not in detection range
                if let Some(attacker_actor) = get_actor(world, entity, attacker_id) {
                    target = Some(attacker_actor);
                }
            }
        }
    }

    AiContext { detected, target }
}
