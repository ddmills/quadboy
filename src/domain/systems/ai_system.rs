use bevy_ecs::prelude::*;

use crate::domain::{Energy, EnergyActionType, TurnState, get_energy_cost};

pub fn ai_turn(turn_state: Res<TurnState>, mut q_energy: Query<&mut Energy>) {
    // Only run AI when it's not the player's turn
    if turn_state.is_players_turn {
        return;
    }

    let Some(current_entity) = turn_state.current_turn_entity else {
        return;
    };

    let Ok(mut energy) = q_energy.get_mut(current_entity) else {
        return;
    };

    let cost = get_energy_cost(EnergyActionType::Wait);
    energy.consume_energy(cost);
}
