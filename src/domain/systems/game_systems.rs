use bevy_ecs::{
    resource::Resource,
    system::SystemId,
    world::World,
};
use macroquad::telemetry;

use crate::{
    domain::{TurnState, ai_turn, process_energy_consumption, turn_scheduler},
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

    for id in system_ids {
        let _ = world.run_system(id);
    }
}

pub fn game_loop(world: &mut World) {
    telemetry::begin_zone("game_loop");
    loop {
        exec_game_systems(world);
        let Some(turn) = world.get_resource::<TurnState>() else {
            return;
        };
        if turn.is_players_turn {
            break;
        }
    }
    telemetry::end_zone();
}
