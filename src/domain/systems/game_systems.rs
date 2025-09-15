use bevy_ecs::{
    resource::Resource,
    system::{RunSystemOnce, SystemId},
    world::World,
};
use macroquad::{prelude::trace, telemetry};

use crate::{
    domain::{
        PlayerPosition, TurnState, Zones, ai_turn,
        systems::{
            armor_regen_system::armor_regen_system,
            cleanup_system::on_entity_destroyed_cleanup,
            health_system::update_health_system,
            loot_drop_system::on_entity_destroyed_loot,
            player_map::update_player_map,
            stats_system::{equipment_stat_modifier_system, recalculate_stats_system},
        },
        turn_scheduler, update_entity_visibility_flags, update_lighting_system,
        update_player_position_resource, update_player_vision,
    },
    rendering::update_entity_pos,
};

#[derive(Resource)]
pub struct GameSystems {
    all: Vec<SystemId>,
}

pub fn register_game_systems(world: &mut World) {
    let systems = vec![
        world.register_system(apply_deferred),
        world.register_system(update_player_position_resource),
        world.register_system(update_player_map), // After player position update
        world.register_system(update_entity_pos),
        world.register_system(equipment_stat_modifier_system),
        world.register_system(recalculate_stats_system),
        world.register_system(update_health_system), // After stats system
        world.register_system(armor_regen_system),   // After health system
        world.register_system(update_player_vision),
        world.register_system(update_entity_visibility_flags),
        world.register_system(update_lighting_system),
        world.register_system(turn_scheduler),
        world.register_system(ai_turn),
        world.register_system(on_entity_destroyed_loot),
        world.register_system(on_entity_destroyed_cleanup),
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

#[inline]
pub fn apply_deferred(world: &mut World) {
    let _ = world.run_system_once(bevy_ecs::schedule::ApplyDeferred);
}

pub fn game_loop(world: &mut World) {
    telemetry::begin_zone("game_loop");
    let mut iterations = 0;
    const MAX_ITERATIONS: u32 = 200;

    loop {
        {
            let Some(player_pos) = world.get_resource::<PlayerPosition>() else {
                return;
            };

            let player_zone_idx = player_pos.zone_idx();

            let Some(zones) = world.get_resource::<Zones>() else {
                return;
            };

            if !zones.active.contains(&player_zone_idx) {
                telemetry::end_zone();
                return;
            };
        }

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
