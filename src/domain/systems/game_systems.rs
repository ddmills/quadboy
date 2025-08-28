use bevy_ecs::{
    resource::Resource,
    system::{RunSystemOnce, SystemId},
    world::World,
};
use macroquad::{prelude::trace, telemetry};

use crate::{
    domain::{
        TurnState, ai_turn, process_energy_consumption, turn_scheduler,
        update_entity_visibility_flags, update_player_vision,
    },
    rendering::update_entity_pos,
};

#[derive(Resource)]
pub struct GameSystems {
    all: Vec<SystemId>,
}

pub fn register_game_systems(world: &mut World) {
    let systems = vec![
        world.register_system(turn_scheduler),
        world.register_system(ai_turn),
        world.register_system(update_entity_pos),
        world.register_system(process_energy_consumption),
        world.register_system(update_player_vision),
        world.register_system(update_entity_visibility_flags),
    ];

    world.insert_resource(GameSystems { all: systems });
}

fn exec_game_systems(world: &mut World) {
    let system_ids = {
        let Some(systems) = world.get_resource::<GameSystems>() else {
            return;
        };
        systems.all.clone()
    };

    let _ = world.run_system_once(bevy_ecs::schedule::ApplyDeferred);

    for id in system_ids {
        let _ = world.run_system(id);
    }
}

pub fn game_loop(world: &mut World) {
    telemetry::begin_zone("game_loop");
    let mut iterations = 0;
    const MAX_ITERATIONS: u32 = 25;

    loop {
        exec_game_systems(world);
        let Some(turn) = world.get_resource::<TurnState>() else {
            return;
        };

        if turn.is_players_turn {
            break;
        }

        iterations += 1;
        if iterations >= MAX_ITERATIONS {
            trace!("hit max iter");
            break;
        }
    }
    telemetry::end_zone();
}
