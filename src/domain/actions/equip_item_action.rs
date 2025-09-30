use bevy_ecs::prelude::*;

use crate::{
    domain::{
        Energy, EnergyActionType, EquipmentSlots, Equippable, Equipped, Inventory,
        UnequipItemAction, actions::GameAction, get_base_energy_cost,
        inventory::InventoryChangedEvent,
    },
    engine::{StableId, StableIdRegistry},
};

pub struct EquipItemAction {
    pub entity_id: u64, // Who is equipping (stable ID)
    pub item_id: u64,   // What to equip (stable ID)
}

impl GameAction for EquipItemAction {
    fn try_apply(self, world: &mut World) -> bool {
        // Get entities from registry
        let (entity, item_entity) = {
            let Some(registry) = world.get_resource::<StableIdRegistry>() else {
                return false;
            };

            let Some(entity) = registry.get_entity(StableId(self.entity_id)) else {
                return false;
            };

            let Some(item_entity) = registry.get_entity(StableId(self.item_id)) else {
                return false;
            };

            (entity, item_entity)
        };

        // Check if item is equippable and get requirements
        let slot_requirements = {
            let Some(equippable) = world.get::<Equippable>(item_entity) else {
                return false;
            };
            equippable.slot_requirements.clone()
        };

        // Check inventory and remove item
        {
            let Some(inventory) = world.get_mut::<Inventory>(entity) else {
                return false;
            };

            if !inventory.contains_id(self.item_id) {
                return false;
            }

            // inventory.remove_item(self.item_id);
        }

        // Auto-unequip existing items in target slots
        {
            let Some(equipment_slots) = world.get::<EquipmentSlots>(entity) else {
                return false;
            };

            // Collect items to unequip
            let mut items_to_unequip = Vec::new();
            for &slot in &slot_requirements {
                if let Some(existing_item_id) = equipment_slots.get_equipped_item(slot) {
                    items_to_unequip.push(existing_item_id);
                }
            }

            // Unequip existing items
            for existing_item_id in items_to_unequip {
                // Create and apply UnequipItemAction for each existing item
                let unequip_action = UnequipItemAction::new(existing_item_id);
                unequip_action.apply(world);
            }
        }

        // Add to equipment slots
        {
            let Some(mut equipment_slots) = world.get_mut::<EquipmentSlots>(entity) else {
                return false;
            };
            equipment_slots.equip(self.item_id, &slot_requirements);
        }

        // Update item components - keep InInventory, add Equipped
        world
            .entity_mut(item_entity)
            .insert(Equipped::new(self.entity_id, slot_requirements));

        // Consume energy if entity has energy (for player actions)
        if let Some(mut energy) = world.get_mut::<Energy>(entity) {
            let cost = get_base_energy_cost(EnergyActionType::EquipItem);
            energy.consume_energy(cost);
        }

        world.send_event(InventoryChangedEvent);

        true
    }
}

impl Command for EquipItemAction {
    fn apply(self, world: &mut World) {
        self.try_apply(world);
    }
}
