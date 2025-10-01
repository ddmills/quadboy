use bevy_ecs::prelude::*;
use quadboy_macros::profiled_system;

use crate::{
    domain::{ActiveConditions, ConditionBlink, HitBlink},
    engine::Time,
    rendering::Glyph,
};

#[profiled_system]
pub fn condition_blink_system(
    mut cmds: Commands,
    mut q_condition_blink: Query<(Entity, &mut ConditionBlink, &mut Glyph, Option<&HitBlink>)>,
    mut q_orphaned_glyphs: Query<
        (Entity, &mut Glyph, Option<&HitBlink>),
        (Without<ConditionBlink>, Without<ActiveConditions>),
    >,
    time: Res<Time>,
) {
    let mut entities_to_remove = Vec::new();

    for (entity, mut condition_blink, mut glyph, hit_blink) in q_condition_blink.iter_mut() {
        // HitBlink takes priority - skip if active
        if hit_blink.is_some() {
            continue;
        }

        // Remove if no conditions remain
        if condition_blink.is_empty() {
            glyph.outline_override = None;
            entities_to_remove.push(entity);
            continue;
        }

        // Update timers
        condition_blink.update_timers(time.dt);

        // Set outline color based on current condition and blink state
        glyph.outline_override = if condition_blink.blink_on {
            condition_blink.get_current_color()
        } else {
            None
        };
    }

    // Clean up orphaned glyphs that have outline overrides but no conditions
    for (_entity, mut glyph, hit_blink) in q_orphaned_glyphs.iter_mut() {
        // Only clear if HitBlink is not active (HitBlink manages its own outline)
        if hit_blink.is_none() && glyph.outline_override.is_some() {
            glyph.outline_override = None;
        }
    }

    // Remove ConditionBlink components from entities with no conditions
    for entity in entities_to_remove {
        cmds.entity(entity).remove::<ConditionBlink>();
    }
}

/// System to sync ConditionBlink with ActiveConditions
/// This ensures ConditionBlink stays in sync with actual conditions
#[profiled_system]
pub fn sync_condition_blink_system(
    mut cmds: Commands,
    q_entities_with_conditions: Query<(Entity, &ActiveConditions), Without<ConditionBlink>>,
    mut q_entities_with_both: Query<(Entity, &ActiveConditions, &mut ConditionBlink, &mut Glyph)>,
    mut q_entities_with_blink_only: Query<
        (Entity, &mut Glyph),
        (With<ConditionBlink>, Without<ActiveConditions>),
    >,
) {
    // Handle entities that have conditions but no ConditionBlink - create new ones
    for (entity, active_conditions) in q_entities_with_conditions.iter() {
        if !active_conditions.is_empty() {
            let mut condition_blink = ConditionBlink::new();
            sync_condition_blink_with_active(&mut condition_blink, active_conditions);
            cmds.entity(entity).insert(condition_blink);
        }
    }

    // Handle entities that have both - sync existing ones or remove if empty
    for (entity, active_conditions, mut condition_blink, mut glyph) in
        q_entities_with_both.iter_mut()
    {
        if active_conditions.is_empty() {
            // No conditions - clear outline and remove ConditionBlink
            glyph.outline_override = None;
            cmds.entity(entity).remove::<ConditionBlink>();
        } else {
            // Has conditions - sync existing ConditionBlink
            sync_condition_blink_with_active(&mut condition_blink, active_conditions);
        }
    }

    // Remove ConditionBlink from entities that no longer have ActiveConditions
    for (entity, mut glyph) in q_entities_with_blink_only.iter_mut() {
        // Clear outline and remove component
        glyph.outline_override = None;
        cmds.entity(entity).remove::<ConditionBlink>();
    }
}

fn sync_condition_blink_with_active(
    condition_blink: &mut ConditionBlink,
    active_conditions: &ActiveConditions,
) {
    // Remove conditions that are no longer active
    let active_condition_types: Vec<_> = active_conditions
        .conditions
        .iter()
        .map(|c| &c.condition_type)
        .collect();

    condition_blink
        .conditions
        .retain(|blink_data| active_condition_types.contains(&&blink_data.condition_type));

    // Add new conditions
    for condition in &active_conditions.conditions {
        let color = condition.condition_type.get_blink_color();
        condition_blink.add_condition(condition.condition_type.clone(), color);
    }

    // Reset index if it's out of bounds
    if condition_blink.current_condition_index >= condition_blink.conditions.len()
        && !condition_blink.conditions.is_empty()
    {
        condition_blink.current_condition_index = 0;
    }
}
