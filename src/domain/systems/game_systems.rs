use bevy_ecs::{
    resource::Resource,
    system::{RunSystemOnce, SystemId},
    world::World,
};
use macroquad::prelude::trace;
use quadboy_macros::profiled_system;

use crate::{
    domain::{
        PlayerPosition, TurnState, Zones, ai_turn, recalculate_collider_flags_system,
        systems::{
            armor_regen_system::armor_regen_system,
            cleanup_system::on_entity_destroyed_cleanup,
            condition_system::{process_conditions, spawn_condition_particles},
            death_check_system::death_check_system,
            explosion_system::explosion_system,
            faction_map::update_faction_maps,
            fuse_system::fuse_system,
            health_system::update_health_system,
            loot_drop_system::on_entity_destroyed_loot,
            stats_system::{equipment_stat_modifier_system, recalculate_stats_system},
        },
        tick_faction_modifiers, turn_scheduler, update_entity_visibility_flags,
        update_lighting_system, update_player_position_resource, update_player_vision,
    },
    rendering::position_systems::{place_static_entities, update_dynamic_entity_pos},
    tracy_plot,
};

#[derive(Resource)]
pub struct GameSystems {
    all: Vec<SystemId>,
    post: Vec<SystemId>,
}

pub fn register_game_systems(world: &mut World) {
    let all = vec![
        world.register_system(apply_deferred),
        world.register_system(update_player_position_resource),
        world.register_system(place_static_entities),
        world.register_system(update_dynamic_entity_pos),
        world.register_system(recalculate_collider_flags_system),
        world.register_system(equipment_stat_modifier_system),
        world.register_system(recalculate_stats_system),
        world.register_system(process_conditions),
        world.register_system(spawn_condition_particles),
        world.register_system(death_check_system),
        world.register_system(update_health_system),
        world.register_system(armor_regen_system),
        world.register_system(tick_faction_modifiers),
        world.register_system(turn_scheduler),
        world.register_system(fuse_system),
        world.register_system(explosion_system),
        world.register_system(on_entity_destroyed_loot),
        world.register_system(on_entity_destroyed_cleanup),
        world.register_system(ai_turn),
    ];
    let post = vec![
        world.register_system(update_lighting_system),
        world.register_system(update_player_vision),
        world.register_system(update_entity_visibility_flags),
        world.register_system(update_faction_maps),
    ];

    world.insert_resource(GameSystems { all, post });
}

#[profiled_system]
fn exec_game_systems(world: &mut World) {
    let system_ids = {
        let Some(systems) = world.get_resource::<GameSystems>() else {
            return;
        };
        systems.all.clone()
    };

    for id in system_ids {
        let result = world.run_system(id);
        let _ = result;
    }
}

#[profiled_system]
fn exec_game_post_systems(world: &mut World) {
    let system_ids = {
        let Some(systems) = world.get_resource::<GameSystems>() else {
            return;
        };
        systems.post.clone()
    };

    for id in system_ids {
        let _ = world.run_system(id);
    }
}

#[inline]
#[profiled_system]
pub fn apply_deferred(world: &mut World) {
    let _ = world.run_system_once(bevy_ecs::schedule::ApplyDeferred);
}

#[profiled_system]
pub fn game_loop(world: &mut World) {
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
            break;
        }
    }

    exec_game_post_systems(world);

    tracy_plot!("Game Loop Iterations", iterations as f64);
}
