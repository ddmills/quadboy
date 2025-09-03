use bevy_ecs::prelude::*;

use crate::domain::{EquipmentSlots, Equipped, InInventory, Inventory};
use crate::engine::StableIdRegistry;

pub fn update_equipped_items(
    q_equipment: Query<(Entity, &EquipmentSlots)>,
    q_equipped: Query<(Entity, &Equipped)>,
    registry: Res<StableIdRegistry>,
) {
    // This system can be expanded to:
    // - Apply equipment bonuses
    // - Update visual appearance
    // - Check for broken equipment
    // - Validate equipment state consistency
}

pub fn validate_equipment_state(
    mut cmds: Commands,
    q_equipment: Query<(Entity, &EquipmentSlots)>,
    q_equipped: Query<(Entity, &Equipped)>,
    q_inventory: Query<&Inventory>,
    registry: Res<StableIdRegistry>,
) {
    // Ensure equipped items are properly tracked
    for (entity, equipment) in q_equipment.iter() {
        let Some(entity_stable_id) = registry.get_id(entity) else {
            continue;
        };

        // Check each equipped item
        for &item_id in equipment.get_all_equipped().iter() {
            let Some(item_entity) = registry.get_entity(item_id) else {
                // Item doesn't exist anymore, clear from slots
                // This would require mutable access, so we'd need to restructure
                continue;
            };

            // Ensure item has Equipped component
            if !q_equipped.contains(item_entity) {
                cmds.entity(item_entity).insert(Equipped::new(
                    entity_stable_id,
                    vec![], // We'd need to determine the actual slots
                ));
            }
        }
    }

    // Check for orphaned equipped items
    for (item_entity, equipped) in q_equipped.iter() {
        let Some(owner_entity) = registry.get_entity(equipped.owner_id) else {
            // Owner doesn't exist, remove Equipped component
            cmds.entity(item_entity).remove::<Equipped>();
            continue;
        };

        // Verify owner has this item equipped
        if let Ok((_, equipment)) = q_equipment.get(owner_entity) {
            let Some(item_id) = registry.get_id(item_entity) else {
                continue;
            };

            let is_actually_equipped = equipment.get_all_equipped().contains(&item_id);

            if !is_actually_equipped {
                // Item thinks it's equipped but isn't in any slot
                cmds.entity(item_entity).remove::<Equipped>();

                // Try to add back to inventory
                if let Ok(inventory) = q_inventory.get(owner_entity)
                    && !inventory.contains_id(item_id) {
                        cmds.entity(item_entity)
                            .insert(InInventory::new(equipped.owner_id));
                    }
            }
        }
    }
}

pub fn debug_equipment(
    q_equipment: Query<(Entity, &EquipmentSlots)>,
    q_labels: Query<&crate::domain::Label>,
    registry: Res<StableIdRegistry>,
) {
    for (entity, equipment) in q_equipment.iter() {
        let entity_name = q_labels.get(entity).map(|l| l.get()).unwrap_or("Unknown");

        println!("Equipment for {entity_name}:");
        for (slot, item_id) in equipment.slots.iter() {
            if let Some(id) = item_id {
                let item_name = registry
                    .get_entity(*id)
                    .and_then(|e| q_labels.get(e).ok())
                    .map(|l| l.get())
                    .unwrap_or("Unknown Item");
                println!("  {slot:?}: {item_name} (ID: {id})");
            }
        }
    }
}
