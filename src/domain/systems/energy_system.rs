use bevy_ecs::prelude::*;

use crate::{
    domain::{Energy, InActiveZone, Player, PursuingPlayer, StatType, Stats},
    engine::Clock,
    tracy_plot, tracy_span,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnergyActionType {
    Move,
    Wait,
    DropItem,
    PickUpItem,
    EquipItem,
    UnequipItem,
    TransferItem,
    ToggleLight,
    Shoot,
    Attack,
    Reload,
    Eat,
}

#[derive(Resource, Default)]
pub struct TurnState {
    pub is_players_turn: bool,
    pub current_turn_entity: Option<Entity>,
}

pub fn turn_scheduler(
    mut q_energy: Query<(Entity, &mut Energy), Or<(With<InActiveZone>, With<PursuingPlayer>)>>,
    mut turn_state: ResMut<TurnState>,
    mut clock: ResMut<Clock>,
    q_player: Query<Entity, With<Player>>,
) {
    tracy_span!("turn_scheduler");
    ("turn_scheduler");

    // Clear tick delta at the start of each turn scheduling cycle
    clock.clear_tick_delta();

    let energy_entity_count = q_energy.iter().count() as f64;
    tracy_plot!("Entities with Energy", energy_entity_count);
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
        return;
    };

    turn_state.is_players_turn = highest_entity == player_entity;

    tracy_plot!(
        "Player Turn",
        if turn_state.is_players_turn { 1.0 } else { 0.0 }
    );
}

pub fn get_base_energy_cost(action: EnergyActionType) -> i32 {
    match action {
        EnergyActionType::Move => 100,
        EnergyActionType::Wait => 100,
        EnergyActionType::DropItem => 50,
        EnergyActionType::PickUpItem => 75,
        EnergyActionType::EquipItem => 75,
        EnergyActionType::UnequipItem => 50,
        EnergyActionType::TransferItem => 10,
        EnergyActionType::ToggleLight => 25,
        EnergyActionType::Shoot => 150,
        EnergyActionType::Attack => 150,
        EnergyActionType::Reload => 100,
        EnergyActionType::Eat => 50,
    }
}

pub fn get_energy_cost(action: EnergyActionType, stats: Option<&Stats>) -> i32 {
    let base_cost = get_base_energy_cost(action);
    match action {
        EnergyActionType::Move => {
            if let Some(stats) = stats {
                let speed = stats.get_stat(StatType::Speed);
                (base_cost - (speed * 2)).max(1) // Ensure minimum cost of 1
            } else {
                base_cost
            }
        }
        _ => base_cost, // Other actions use base cost for now
    }
}
