use bevy_ecs::prelude::*;
use macroquad::prelude::trace;

use crate::{
    domain::{
        Actor, Energy, EnergyActionType, TurnState, ai_try_attacking_nearby,
        ai_try_move_toward_target, ai_try_select_target, detect_actors, get_base_energy_cost,
    },
    tracy_span,
};

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

    let mut context = build_ai_context(world, current_entity);

    if ai_try_attacking_nearby(world, current_entity, &mut context) {
        return;
    }

    if ai_try_select_target(world, current_entity, &mut context) {
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
    AiContext {
        detected: detect_actors(world, entity),
        target: None,
    }
}
