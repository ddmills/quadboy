use bevy_ecs::prelude::*;

use crate::domain::{ConsumeEnergyEvent, EnergyActionType, TurnState};

pub fn ai_action_system(
    turn_state: Res<TurnState>,
    mut e_consume_energy: EventWriter<ConsumeEnergyEvent>,
) {
    // Only run AI when it's not the player's turn
    if turn_state.is_players_turn {
        return;
    }

    // Get the current entity that should act
    let Some(current_entity) = turn_state.current_turn_entity else {
        return;
    };

    // For now, AI just waits
    e_consume_energy.write(ConsumeEnergyEvent::new(
        current_entity,
        EnergyActionType::Wait,
    ));
}
