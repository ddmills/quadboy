use bevy_ecs::prelude::*;

use crate::{
    domain::{
        ActiveConditions, Condition, ConditionType, Health, Level, PlayerPosition, StatModifiers,
        Stats,
    },
    engine::Clock,
    rendering::{Position, world_to_zone_idx},
};

pub fn process_conditions(
    mut q_entities: Query<(
        Entity,
        &mut ActiveConditions,
        &mut Health,
        &mut StatModifiers,
        &Level,
        &Stats,
    )>,
    clock: Res<Clock>,
    mut cmds: Commands,
) {
    let current_tick = clock.current_tick();
    let tick_delta = clock.tick_delta();

    for (_entity, mut conditions, mut health, mut stat_modifiers, level, stats) in
        q_entities.iter_mut()
    {
        let mut conditions_to_remove = vec![];
        let mut stat_modifiers_to_remove = vec![];
        let mut stat_modifiers_to_add = vec![];

        for (index, condition) in conditions.conditions.iter_mut().enumerate() {
            // Process duration
            let expired = condition.tick(tick_delta);
            if expired {
                conditions_to_remove.push(index);
                // Mark condition's stat modifiers for removal
                let condition_id = format!("{:?}_{}", condition.condition_type, index);
                stat_modifiers_to_remove.push(condition_id.clone());
                continue;
            }

            // Process condition effects
            match &condition.condition_type {
                ConditionType::Poisoned {
                    damage_per_tick,
                    tick_interval,
                } => {
                    condition.accumulated_effect += tick_delta as f32;
                    if condition.accumulated_effect >= *tick_interval as f32 {
                        let damage_cycles =
                            (condition.accumulated_effect / *tick_interval as f32) as i32;
                        let total_damage = damage_cycles * damage_per_tick;
                        let source = condition.source.get_source_id();
                        health.take_damage_from_source(total_damage, current_tick, source);
                        condition.accumulated_effect -=
                            (damage_cycles as f32) * (*tick_interval as f32);
                    }
                }

                ConditionType::Bleeding {
                    damage_per_tick, ..
                } => {
                    condition.accumulated_effect += tick_delta as f32;
                    if condition.accumulated_effect >= 100.0 {
                        // Bleeding ticks every 100 game ticks
                        let damage_cycles = (condition.accumulated_effect / 100.0) as i32;
                        let total_damage = damage_cycles * damage_per_tick;
                        let source = condition.source.get_source_id();
                        health.take_damage_from_source(total_damage, current_tick, source);
                        condition.accumulated_effect -= (damage_cycles as f32) * 100.0;
                    }
                }

                ConditionType::Burning {
                    damage_per_tick, ..
                } => {
                    condition.accumulated_effect += tick_delta as f32;
                    if condition.accumulated_effect >= 80.0 {
                        // Burning ticks every 80 game ticks
                        let damage_cycles = (condition.accumulated_effect / 80.0) as i32;
                        let total_damage = damage_cycles * damage_per_tick;
                        let source = condition.source.get_source_id();
                        health.take_damage_from_source(total_damage, current_tick, source);
                        condition.accumulated_effect -= (damage_cycles as f32) * 80.0;
                    }
                }

                // Other condition types don't need tick-based processing
                _ => {}
            }
        }

        // Remove expired conditions (in reverse order to maintain indices)
        for &index in conditions_to_remove.iter().rev() {
            let condition = conditions.conditions.remove(index);

            // Cleanup associated particle spawner entity
            if let Some(spawner_entity) = condition.particle_spawner_entity {
                cmds.entity(spawner_entity).despawn();
            }
        }

        // Remove old stat modifiers for expired conditions
        for condition_id in stat_modifiers_to_remove {
            stat_modifiers.remove_condition_modifiers(&condition_id);
        }

        // Add new stat modifiers for active conditions
        for (stat_type, modifier) in stat_modifiers_to_add {
            stat_modifiers.add_modifier(stat_type, modifier);
        }
    }
}

// Helper function to apply a condition to an entity
pub fn apply_condition_to_entity(
    entity: Entity,
    condition: Condition,
    world: &mut World,
) -> Result<(), String> {
    // Get or create ActiveConditions component
    if let Some(mut conditions) = world.get_mut::<ActiveConditions>(entity) {
        // Check for stacking logic
        let mut spawners_to_cleanup = Vec::new();
        if !condition.condition_type.can_stack() {
            // Remove existing condition of this type and collect spawner entities to cleanup
            let removed_conditions = conditions.remove_condition(&condition.condition_type);
            for removed_condition in removed_conditions {
                if let Some(spawner_entity) = removed_condition.particle_spawner_entity {
                    spawners_to_cleanup.push(spawner_entity);
                }
            }
        }
        conditions.add_condition(condition);

        // Drop the borrow on conditions before despawning
        drop(conditions);

        // Clean up particle spawner entities
        for spawner_entity in spawners_to_cleanup {
            world.despawn(spawner_entity);
        }
    } else {
        // Entity doesn't have ActiveConditions, add it
        let mut new_conditions = ActiveConditions::new();
        new_conditions.add_condition(condition);
        world.entity_mut(entity).insert(new_conditions);
    }

    Ok(())
}

pub fn spawn_condition_particles(world: &mut World) {
    let mut spawn_requests = Vec::new();

    // Get the active zone from player position
    let active_zone = world
        .get_resource::<PlayerPosition>()
        .map(|player_pos| player_pos.zone_idx());

    // Collect entities and conditions that need particle spawners
    {
        let mut q_entities = world.query::<(Entity, &ActiveConditions, &Position)>();
        for (entity, conditions, position) in q_entities.iter(world) {
            // Check if entity is in the active zone
            if let Some(active_zone_idx) = active_zone {
                let world_pos = position.world();
                let entity_zone_idx = world_to_zone_idx(world_pos.0, world_pos.1, world_pos.2);

                // Only spawn particles for entities in the active zone
                if entity_zone_idx != active_zone_idx {
                    continue;
                }
            }

            for (condition_idx, condition) in conditions.conditions.iter().enumerate() {
                if condition.particle_spawner_entity.is_none() {
                    if let Some(spawner_config) = condition.condition_type.create_particle_spawner()
                    {
                        spawn_requests.push((entity, condition_idx, spawner_config));
                    }
                }
            }
        }
    }

    // Process spawn requests
    let mut updates = Vec::new();
    for (entity, condition_idx, spawner_config) in spawn_requests {
        let spawner_entity = spawner_config.spawn_persistent(world, Some(entity));
        updates.push((entity, condition_idx, spawner_entity));
    }

    // Apply the updates
    for (entity, condition_idx, spawner_entity) in updates {
        if let Some(mut conditions) = world.get_mut::<ActiveConditions>(entity) {
            if let Some(condition) = conditions.conditions.get_mut(condition_idx) {
                condition.particle_spawner_entity = Some(spawner_entity);
            }
        }
    }
}
