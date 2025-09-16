use bevy_ecs::{prelude::*, system::ParamSet};

use crate::domain::{
    Attributes, EquipmentSlots, Item, ModifierSource, StatModifier, StatModifiers, StatType, Stats,
};
use crate::engine::StableIdRegistry;
use crate::tracy_span;

pub fn recalculate_stats_system(
    mut q_stats: Query<
        (&mut Stats, &Attributes, &StatModifiers),
        Or<(Changed<StatModifiers>, Changed<Attributes>)>,
    >,
) {
    tracy_span!("recalculate_stats_system");
    for (mut stats, attributes, modifiers) in q_stats.iter_mut() {
        for stat_type in StatType::all() {
            let base_value = stat_type.get_base_value(attributes);
            let modifier_sum = modifiers.get_total_for_stat(*stat_type);
            let new_value = base_value + modifier_sum;

            // Only update if changed
            if stats.values.get(stat_type) != Some(&new_value) {
                stats.values.insert(*stat_type, new_value);
            }
        }
    }
}

pub fn equipment_stat_modifier_system(
    mut param_set: ParamSet<(Query<&mut StatModifiers>, Query<&StatModifiers, With<Item>>)>,
    q_equipment_changed: Query<(Entity, &EquipmentSlots), Changed<EquipmentSlots>>,
    registry: Res<StableIdRegistry>,
) {
    tracy_span!("equipment_stat_modifier_system");
    // Collect all the modifiers to add first
    let mut modifiers_to_add: Vec<(Entity, StatType, StatModifier)> = Vec::new();

    for (entity, equipment_slots) in q_equipment_changed.iter() {
        // First, collect all equipment modifiers for this entity
        for item_id_opt in equipment_slots.slots.values() {
            if let Some(item_id) = item_id_opt
                && let Some(item_entity) = registry.get_entity(*item_id)
                && let Ok(item_modifiers) = param_set.p1().get(item_entity)
            {
                // Collect all stat modifiers from this item
                for (stat_type, modifiers) in &item_modifiers.modifiers {
                    for modifier in modifiers {
                        modifiers_to_add.push((
                            entity,
                            *stat_type,
                            StatModifier::equipment(modifier.value, *item_id),
                        ));
                    }
                }
            }
        }
    }

    // Now apply the changes to entity modifiers
    for (entity, _equipment_slots) in q_equipment_changed.iter() {
        let mut query = param_set.p0();
        let Ok(mut entity_modifiers) = query.get_mut(entity) else {
            continue;
        };

        // Remove all existing equipment modifiers
        for modifiers in entity_modifiers.modifiers.values_mut() {
            modifiers.retain(|m| !matches!(m.source, ModifierSource::Equipment { .. }));
        }

        // Add the collected modifiers for this entity
        for (target_entity, stat_type, modifier) in &modifiers_to_add {
            if *target_entity == entity {
                entity_modifiers.add_modifier(*stat_type, modifier.clone());
            }
        }
    }
}
