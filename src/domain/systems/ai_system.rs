use bevy_ecs::prelude::*;
use macroquad::prelude::trace;

use crate::{
    domain::{
        DetectedActors, Energy, EnergyActionType, TurnState, ai_try_attacking_nearby,
        ai_try_move_toward_nearest, detect_actors, get_base_energy_cost,
    },
    tracy_span,
};

pub struct AiContext {
    pub detected: Vec<DetectedActors>,
}

impl AiContext {
    pub fn nearest_hostile(&self) -> Option<&DetectedActors> {
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

    // Try to attack anything next to you
    if ai_try_attacking_nearby(world, current_entity, &mut context) {
        return;
    }

    if ai_try_move_toward_nearest(world, current_entity, &mut context) {
        return;
    }

    if let Some(mut energy) = world.get_mut::<Energy>(current_entity) {
        let cost = get_base_energy_cost(EnergyActionType::Wait);
        energy.consume_energy(cost);
    }
}

pub fn build_ai_context(world: &mut World, entity: Entity) -> AiContext {
    AiContext {
        detected: detect_actors(world, entity),
    }
}
