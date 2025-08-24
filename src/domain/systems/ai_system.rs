use bevy_ecs::prelude::*;
use macroquad::telemetry;

use crate::{
    cfg::{MAP_SIZE, ZONE_SIZE},
    common::Rand,
    domain::{Collider, Energy, EnergyActionType, TurnState, Zone, get_energy_cost},
    rendering::{Position, world_to_zone_idx, world_to_zone_local},
};

pub fn ai_turn(
    turn_state: Res<TurnState>,
    mut q_energy: Query<&mut Energy>,
    mut q_position: Query<&mut Position>,
    q_zones: Query<&Zone>,
    q_colliders: Query<&Collider>,
) {
    telemetry::begin_zone("ai_turn");

    // Only run AI when it's not the player's turn
    if turn_state.is_players_turn {
        telemetry::end_zone();
        return;
    }

    let Some(current_entity) = turn_state.current_turn_entity else {
        telemetry::end_zone();
        return;
    };

    let Ok(mut energy) = q_energy.get_mut(current_entity) else {
        telemetry::end_zone();
        return;
    };

    let mut rand = Rand::new();

    let mut action = EnergyActionType::Wait;

    if rand.bool(0.75) {
        let cost = get_energy_cost(action);
        energy.consume_energy(cost);
        telemetry::end_zone();
        return;
    }

    let Ok(mut position) = q_position.get_mut(current_entity) else {
        let cost = get_energy_cost(action);
        energy.consume_energy(cost);
        telemetry::end_zone();
        return;
    };

    let (x, y, z) = position.world();

    let directions = [(-1, 0), (1, 0), (0, -1), (0, 1)];
    let (dx, dy) = rand.pick(&directions);
    let new_x = (x as i32 + dx) as usize;
    let new_y = (y as i32 + dy) as usize;

    let max_x = (MAP_SIZE.0 * ZONE_SIZE.0) - 1;
    let max_y = (MAP_SIZE.1 * ZONE_SIZE.1) - 1;

    if new_x > max_x || new_y > max_y {
        let cost = get_energy_cost(action);
        energy.consume_energy(cost);
        telemetry::end_zone();
        return;
    }

    // Check if destination zone is loaded
    let dest_zone_idx = world_to_zone_idx(new_x, new_y, z);
    let Some(zone) = q_zones.iter().find(|zone| zone.idx == dest_zone_idx) else {
        let cost = get_energy_cost(action);
        energy.consume_energy(cost);
        telemetry::end_zone();
        return;
    };

    let (local_x, local_y) = world_to_zone_local(new_x, new_y);
    if let Some(entities) = zone.entities.get(local_x, local_y) {
        for entity in entities {
            if q_colliders.get(*entity).is_ok() {
                // Found a collider at this position, can't move
                let cost = get_energy_cost(action);
                energy.consume_energy(cost);
                telemetry::end_zone();
                return;
            }
        }
    }

    position.x = new_x as f32;
    position.y = new_y as f32;
    action = EnergyActionType::Move;

    let cost = get_energy_cost(action);
    energy.consume_energy(cost);
    telemetry::end_zone();
}
