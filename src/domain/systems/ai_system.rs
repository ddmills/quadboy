use bevy_ecs::prelude::*;
use macroquad::prelude::trace;
use quadboy_macros::profiled_system;

use crate::{
    common::Rand,
    domain::{
        Actor, AiController, Energy, EnergyActionType, Health, TurnState, ai_try_attacking_nearby,
        ai_try_move_toward_target, ai_try_ranged_attack, ai_try_select_target, ai_try_wait,
        ai_try_wander, detect_actors, get_actor, get_base_energy_cost, try_handle_conditions,
    },
    rendering::{Position, spawn_alert_indicator},
};

#[derive(Resource, Default)]
pub struct AiTurnTracker {
    last_entity: Option<Entity>,
    consecutive_count: u32,
}

#[derive(Default)]
pub struct AiContext {
    pub detected: Vec<Actor>,
    pub target: Option<Actor>,
}

impl AiContext {
    pub fn nearest_hostile(&self) -> Option<&Actor> {
        self.detected
            .iter()
            .filter(|x| x.relationship < 0)
            .min_by_key(|x| x.distance.round() as u32)
    }
}

#[profiled_system]
pub fn ai_turn(world: &mut World) {
    let Some(turn_state) = world.get_resource::<TurnState>() else {
        return;
    };

    if turn_state.is_players_turn {
        return;
    }

    let Some(current_entity) = turn_state.current_turn_entity else {
        return;
    };

    // Track consecutive turns for the same entity to detect stuck AI
    let (is_stuck, needs_energy_check) = {
        let mut tracker = world.get_resource_or_insert_with(AiTurnTracker::default);
        if tracker.last_entity == Some(current_entity) {
            tracker.consecutive_count += 1;
            if tracker.consecutive_count >= 10 {
                eprintln!(
                    "WARNING: Entity {:?} stuck in AI turn loop ({}x), checking energy",
                    current_entity, tracker.consecutive_count
                );
                (true, true)
            } else {
                (false, false)
            }
        } else {
            tracker.last_entity = Some(current_entity);
            tracker.consecutive_count = 1;
            (false, false)
        }
    };

    if needs_energy_check {
        let has_energy = world.get::<Energy>(current_entity).is_some();
        let energy_value = world
            .get::<Energy>(current_entity)
            .map(|e| e.value)
            .unwrap_or(0);

        if !has_energy {
            eprintln!(
                "ERROR: Entity {:?} has no Energy component! Skipping turn.",
                current_entity
            );
            // Reset tracker to avoid getting stuck on this entity
            let mut tracker = world.get_resource_or_insert_with(AiTurnTracker::default);
            tracker.consecutive_count = 0;
            tracker.last_entity = None;
            return;
        } else {
            // Count other entities and their energy
            let mut other_entities = Vec::new();
            for (entity, energy) in world.query::<(Entity, &Energy)>().iter(world) {
                if entity != current_entity {
                    other_entities.push((entity, energy.value));
                }
            }
            other_entities.sort_by_key(|&(_, e)| -e); // Sort by energy descending

            eprintln!(
                "Entity {:?} energy: {}, top 3 others: {:?}",
                current_entity,
                energy_value,
                &other_entities[..3.min(other_entities.len())]
            );

            // Reset counter since we're forcing a wait
            let mut tracker = world.get_resource_or_insert_with(AiTurnTracker::default);
            tracker.consecutive_count = 0;
        }
    }

    if is_stuck {
        // Check if wait actually consumes energy
        let energy_before = world
            .get::<Energy>(current_entity)
            .map(|e| e.value)
            .unwrap_or(0);
        let wait_result = ai_try_wait(world, current_entity);
        let energy_after = world
            .get::<Energy>(current_entity)
            .map(|e| e.value)
            .unwrap_or(0);

        eprintln!(
            "Wait result: {}, energy: {} → {} (consumed: {})",
            wait_result,
            energy_before,
            energy_after,
            energy_before - energy_after
        );
        return;
    }

    // Check if AI had a target before building context
    let had_target_before = world
        .get::<AiController>(current_entity)
        .and_then(|ai| ai.current_target_id)
        .is_some();

    let mut context = build_ai_context(world, current_entity);

    if try_handle_conditions(world, current_entity, &mut context) {
        return;
    }

    if ai_try_select_target(world, current_entity, &mut context) {
        // Check if AI just acquired a target
        let has_target_now = context.target.is_some();
        if !had_target_before && has_target_now {
            // AI just acquired a target - spawn alert particle
            if let Some(position) = world.get::<Position>(current_entity) {
                let world_pos = position.world();
                spawn_alert_indicator(world, world_pos);
            }
        }

        if let Some(mut ai_controller) = world.get_mut::<AiController>(current_entity) {
            ai_controller.current_target_id = context.target.map(|x| x.stable_id);
        };

        // Try ranged attack first if AI has a ranged weapon and target is not adjacent
        if ai_try_ranged_attack(world, current_entity, &mut context) {
            return;
        }

        // Try melee attack if target is adjacent
        if ai_try_attacking_nearby(world, current_entity, &mut context) {
            return;
        }

        if ai_try_move_toward_target(world, current_entity, &mut context) {
            return;
        }

        // we have a target, but we can't move toward it!
        trace!("AI: Can't reach target!");
        ai_try_wait(world, current_entity);
        return;
    } else {
        // No target - try to wander (30% chance) or wait (70% chance)
        let Some(mut rand) = world.get_resource_mut::<Rand>() else {
            // If no random resource, just wait
            ai_try_wait(world, current_entity);
            return;
        };

        let should_wander = rand.random() < 0.3; // 30% chance to wander

        if should_wander && ai_try_wander(world, current_entity) {
            // Successfully wandered, energy consumed by wander action
            return;
        }
    }

    // Default behavior: wait
    ai_try_wait(world, current_entity);
}

pub fn build_ai_context(world: &mut World, entity: Entity) -> AiContext {
    let detected = detect_actors(world, entity);
    let Some(ai_controller) = world.get::<AiController>(entity) else {
        return AiContext::default();
    };

    let mut target = if let Some(target_id) = ai_controller.current_target_id {
        get_actor(world, entity, target_id)
    } else {
        None
    };

    // Check if this AI has been attacked and doesn't have a current target
    if target.is_none() {
        if let Some(health) = world.get::<Health>(entity) {
            if let Some(attacker_id) = health.last_damage_source {
                // Try to target the attacker, even if they're not in detection range
                if let Some(attacker_actor) = get_actor(world, entity, attacker_id) {
                    target = Some(attacker_actor);
                }
            }
        }
    }

    AiContext { detected, target }
}
