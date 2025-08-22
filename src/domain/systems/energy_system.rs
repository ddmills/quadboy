use bevy_ecs::prelude::*;

use crate::{
    domain::{Energy, Player},
    engine::Clock,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnergyActionType {
    Move,
    Wait,
    Sleep,
    Attack,
}

#[derive(Resource, Default)]
pub struct TurnState {
    pub is_players_turn: bool,
    pub current_turn_entity: Option<Entity>,
}

#[derive(Event)]
pub struct ConsumeEnergyEvent {
    pub entity: Entity,
    pub action: EnergyActionType,
}

impl ConsumeEnergyEvent {
    pub fn new(entity: Entity, action: EnergyActionType) -> Self {
        Self { entity, action }
    }
}

pub fn energy_tick_system(
    mut q_energy: Query<(Entity, &mut Energy)>,
    mut turn_state: ResMut<TurnState>,
    mut clock: ResMut<Clock>,
    q_player: Query<Entity, With<Player>>,
) {
    let Some((highest_entity, highest_energy)) =
        q_energy.iter().max_by_key(|(_, energy)| energy.value)
    else {
        return;
    };

    // once all entities have less than zero energy, we advance the clock
    // by the difference, and increase the energy amount for all entities.
    if highest_energy.value < 0 {
        let tick_amount = -highest_energy.value as u32;

        clock.increment_tick(tick_amount);

        for (_, mut energy) in q_energy.iter_mut() {
            energy.add_energy(tick_amount);
        }

        turn_state.current_turn_entity = None;
        turn_state.is_players_turn = false;

        return;
    }

    turn_state.current_turn_entity = Some(highest_entity);

    let Ok(player_entity) = q_player.single() else {
        turn_state.is_players_turn = false;
        // no player?
        return;
    };

    turn_state.is_players_turn = highest_entity == player_entity;
}

pub fn process_energy_consumption(
    mut events: EventReader<ConsumeEnergyEvent>,
    mut q_energy: Query<&mut Energy>,
) {
    for event in events.read() {
        if let Ok(mut energy) = q_energy.get_mut(event.entity) {
            let cost = get_energy_cost(event.action);
            energy.consume_energy(cost);
        }
    }
}

pub fn get_energy_cost(action: EnergyActionType) -> i32 {
    match action {
        EnergyActionType::Move => 100,
        EnergyActionType::Wait => 50,
        EnergyActionType::Sleep => 1000,
        EnergyActionType::Attack => 150,
    }
}
