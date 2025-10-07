use bevy_ecs::prelude::*;
use macroquad::rand;

use crate::{
    domain::{
        ActiveConditions, AiContext, AiController, ConditionType, Energy, EnergyActionType,
        ai_try_attacking_nearby, ai_try_flee_from, ai_try_move_toward, ai_try_random_move,
        ai_try_wait, get_actor, get_base_energy_cost,
    },
    engine::StableIdRegistry,
    rendering::Position,
};

pub fn try_handle_taunted(world: &mut World, entity: Entity, context: &mut AiContext) -> bool {
    let Some(conditions) = world.get::<ActiveConditions>(entity) else {
        return false;
    };

    let conditions_clone = conditions.conditions.clone();

    for condition in &conditions_clone {
        if let ConditionType::Taunted {
            move_toward,
            force_target,
        } = &condition.condition_type
        {
            let id_registry = world.resource::<StableIdRegistry>();
            if let Some(taunt_entity) = id_registry.get_entity(*move_toward) {
                if let Some(taunt_position) = world.get::<Position>(taunt_entity) {
                    let taunt_pos = taunt_position.world();

                    // Override target selection if force_target is true
                    if *force_target {
                        if let Some(mut ai_controller) = world.get_mut::<AiController>(entity) {
                            ai_controller.current_target_id = Some(*move_toward);
                        }
                        context.target = get_actor(world, entity, *move_toward);
                    }

                    // Try attacking if close enough
                    if ai_try_attacking_nearby(world, entity, context) {
                        return true;
                    }

                    // Otherwise move toward the taunter
                    if ai_try_move_toward(world, entity, taunt_pos) {
                        return true;
                    }

                    // Taunted but can't move or attack - wait instead
                    ai_try_wait(world, entity);
                    return true;
                }
            }
        }
    }

    false
}

pub fn try_handle_feared(world: &mut World, entity: Entity) -> bool {
    let Some(conditions) = world.get::<ActiveConditions>(entity) else {
        return false;
    };

    let conditions_clone = conditions.conditions.clone();

    for condition in &conditions_clone {
        if let ConditionType::Feared { flee_from } = &condition.condition_type {
            let id_registry = world.resource::<StableIdRegistry>();
            if let Some(fear_entity) = id_registry.get_entity(*flee_from) {
                if let Some(fear_position) = world.get::<Position>(fear_entity) {
                    if let Some(current_position) = world.get::<Position>(entity) {
                        let fear_pos = fear_position.world();
                        let current_pos = current_position.world();

                        // Calculate distance to fear source
                        let distance = ((fear_pos.0 as f32 - current_pos.0 as f32).powi(2)
                            + (fear_pos.1 as f32 - current_pos.1 as f32).powi(2)
                            + (fear_pos.2 as f32 - current_pos.2 as f32).powi(2))
                        .sqrt();

                        // If within minimum distance, try to flee
                        if distance < 10. {
                            if ai_try_flee_from(world, entity, fear_pos) {
                                return true;
                            }
                            // Feared but can't flee - wait instead
                            ai_try_wait(world, entity);
                            return true;
                        }
                    }
                }
            }
        }
    }

    false
}

pub fn try_handle_confused(world: &mut World, entity: Entity) -> bool {
    let Some(conditions) = world.get::<ActiveConditions>(entity) else {
        return false;
    };

    let conditions_clone = conditions.conditions.clone();

    for condition in &conditions_clone {
        if ConditionType::Confused == condition.condition_type {
            if rand::gen_range(0.0, 1.0) < 0.5 {
                // Try random movement
                if ai_try_random_move(world, entity) {
                    return true;
                }

                // If random movement fails, skip turn (wait)
                ai_try_wait(world, entity);
                return true;
            }
        }
    }

    false
}

pub fn try_handle_conditions(world: &mut World, entity: Entity, context: &mut AiContext) -> bool {
    // Check conditions in priority order: Taunted → Feared → Confused

    if try_handle_taunted(world, entity, context) {
        return true;
    }

    if try_handle_feared(world, entity) {
        return true;
    }

    if try_handle_confused(world, entity) {
        return true;
    }

    false
}
