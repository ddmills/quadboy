use bevy_ecs::prelude::*;

use crate::{
    domain::{
        ActiveConditions, Condition, ConditionType, Health, Level, StatModifier, StatModifiers,
        StatType, Stats,
    },
    engine::Clock,
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
) {
    let current_tick = clock.current_tick();
    let tick_delta = clock.tick_delta();

    for (entity, mut conditions, mut health, mut stat_modifiers, level, stats) in
        q_entities.iter_mut()
    {
        let mut conditions_to_remove = Vec::new();
        let mut stat_modifiers_to_remove = Vec::new();
        let mut stat_modifiers_to_add = Vec::new();

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

                ConditionType::Regenerating {
                    heal_per_tick,
                    tick_interval,
                } => {
                    condition.accumulated_effect += tick_delta as f32;
                    if condition.accumulated_effect >= *tick_interval as f32 {
                        let heal_cycles =
                            (condition.accumulated_effect / *tick_interval as f32) as i32;
                        let total_heal = heal_cycles * heal_per_tick;
                        health.heal(total_heal, level, stats);
                        condition.accumulated_effect -=
                            (heal_cycles as f32) * (*tick_interval as f32);
                    }
                }

                ConditionType::Strengthened { damage_bonus } => {
                    let condition_id = format!("{:?}_{}", condition.condition_type, index);
                    let damage_bonus_i32 = *damage_bonus as i32;
                    stat_modifiers_to_add.push((
                        StatType::Unarmed,
                        StatModifier::condition(damage_bonus_i32, condition_id.clone()),
                    ));
                    stat_modifiers_to_add.push((
                        StatType::Blade,
                        StatModifier::condition(damage_bonus_i32, condition_id.clone()),
                    ));
                    stat_modifiers_to_add.push((
                        StatType::Cudgel,
                        StatModifier::condition(damage_bonus_i32, condition_id.clone()),
                    ));
                }

                ConditionType::Weakened { damage_reduction } => {
                    let condition_id = format!("{:?}_{}", condition.condition_type, index);
                    let penalty = -(*damage_reduction as i32);
                    stat_modifiers_to_add.push((
                        StatType::Unarmed,
                        StatModifier::condition(penalty, condition_id.clone()),
                    ));
                    stat_modifiers_to_add.push((
                        StatType::Blade,
                        StatModifier::condition(penalty, condition_id.clone()),
                    ));
                    stat_modifiers_to_add.push((
                        StatType::Cudgel,
                        StatModifier::condition(penalty, condition_id.clone()),
                    ));
                }

                ConditionType::Blessed { stat_bonus } => {
                    let condition_id = format!("{:?}_{}", condition.condition_type, index);
                    // Apply bonus to all stats
                    for stat_type in StatType::all() {
                        stat_modifiers_to_add.push((
                            *stat_type,
                            StatModifier::condition(*stat_bonus, condition_id.clone()),
                        ));
                    }
                }

                ConditionType::Cursed { stat_penalty } => {
                    let condition_id = format!("{:?}_{}", condition.condition_type, index);
                    let penalty = -(*stat_penalty);
                    // Apply penalty to all stats
                    for stat_type in StatType::all() {
                        stat_modifiers_to_add.push((
                            *stat_type,
                            StatModifier::condition(penalty, condition_id.clone()),
                        ));
                    }
                }

                ConditionType::Defended { damage_reduction } => {
                    let condition_id = format!("{:?}_{}", condition.condition_type, index);
                    let armor_bonus = (*damage_reduction * 10.0) as i32; // Convert percentage to armor points
                    stat_modifiers_to_add.push((
                        StatType::Armor,
                        StatModifier::condition(armor_bonus, condition_id),
                    ));
                }

                ConditionType::Vulnerable { damage_multiplier } => {
                    let condition_id = format!("{:?}_{}", condition.condition_type, index);
                    let armor_penalty = -((*damage_multiplier - 1.0) * 20.0) as i32; // Convert multiplier to armor penalty
                    stat_modifiers_to_add.push((
                        StatType::Armor,
                        StatModifier::condition(armor_penalty, condition_id),
                    ));
                }

                // Other condition types don't need tick-based processing
                _ => {}
            }
        }

        // Remove expired conditions (in reverse order to maintain indices)
        for &index in conditions_to_remove.iter().rev() {
            conditions.conditions.remove(index);
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

pub fn cleanup_empty_conditions(mut q_conditions: Query<&mut ActiveConditions>) {
    for conditions in q_conditions.iter_mut() {
        if conditions.is_empty() {
            // Could add logic here to remove the component entirely if desired
            // For now, we'll just keep the empty component
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
        if !condition.condition_type.can_stack() {
            // Remove existing condition of this type
            conditions.remove_condition(&condition.condition_type);
        }
        conditions.add_condition(condition);
    } else {
        // Entity doesn't have ActiveConditions, add it
        let mut new_conditions = ActiveConditions::new();
        new_conditions.add_condition(condition);
        world.entity_mut(entity).insert(new_conditions);
    }

    Ok(())
}

// Helper function to remove a specific condition type from an entity
pub fn remove_condition_from_entity(
    entity: Entity,
    condition_type: &ConditionType,
    world: &mut World,
) -> Result<(), String> {
    if let Some(mut conditions) = world.get_mut::<ActiveConditions>(entity) {
        conditions.remove_condition(condition_type);

        // Also remove any associated stat modifiers
        if let Some(mut stat_modifiers) = world.get_mut::<StatModifiers>(entity) {
            // Find and remove condition modifiers for this condition type
            let condition_pattern = format!("{:?}_", condition_type);
            for modifiers in stat_modifiers.modifiers.values_mut() {
                modifiers.retain(|m| {
                    if let crate::domain::ModifierSource::Condition { condition_id } = &m.source {
                        !condition_id.starts_with(&condition_pattern)
                    } else {
                        true
                    }
                });
            }
        }
    }

    Ok(())
}

// Helper function to check if an entity has a specific condition
pub fn entity_has_condition(entity: Entity, condition_type: &ConditionType, world: &World) -> bool {
    world
        .get::<ActiveConditions>(entity)
        .map(|conditions| conditions.has_condition(condition_type))
        .unwrap_or(false)
}
