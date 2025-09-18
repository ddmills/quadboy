use bevy_ecs::{
    resource::Resource,
    system::{RunSystemOnce, SystemId},
    world::World,
};
use macroquad::prelude::trace;

use crate::{
    domain::{
        PlayerPosition, TurnState, Zones, ai_turn, manage_pursuit_timeout,
        systems::{
            armor_regen_system::armor_regen_system,
            cleanup_system::on_entity_destroyed_cleanup,
            condition_system::process_conditions,
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
    rendering::update_entity_pos,
    tracy_plot, tracy_span,
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
        world.register_system(update_entity_pos),
        world.register_system(equipment_stat_modifier_system),
        world.register_system(recalculate_stats_system),
        world.register_system(process_conditions),
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
        world.register_system(manage_pursuit_timeout),
    ];
    let post = vec![
        world.register_system(update_lighting_system),
        world.register_system(update_player_vision),
        world.register_system(update_entity_visibility_flags),
        world.register_system(update_faction_maps),
    ];

    world.insert_resource(GameSystems { all, post });
}

fn exec_game_systems(world: &mut World) {
    tracy_span!("exec_game_systems");

    let system_ids = {
        tracy_span!("exec_game_systems:clone_systems");
        let Some(systems) = world.get_resource::<GameSystems>() else {
            return;
        };
        systems.all.clone()
    };

    {
        tracy_span!("exec_game_systems:iter");
        for id in system_ids {
            let _ = world.run_system(id);
        }
    }
}

fn exec_game_post_systems(world: &mut World) {
    tracy_span!("exec_game_post_systems");

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
pub fn apply_deferred(world: &mut World) {
    tracy_span!("apply_deferred");
    let _ = world.run_system_once(bevy_ecs::schedule::ApplyDeferred);
}

pub fn game_loop(world: &mut World) {
    tracy_span!("game_loop");
    let mut iterations = 0;
    const MAX_ITERATIONS: u32 = 100;

    loop {
        tracy_span!("game_loop:iter");
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
            trace!("hit max iter");
            break;
        }
    }

    exec_game_post_systems(world);

    tracy_plot!("Game Loop Iterations", iterations as f64);
}
