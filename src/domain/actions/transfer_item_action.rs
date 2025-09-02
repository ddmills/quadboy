use bevy_ecs::prelude::*;

use crate::{
    domain::{
        Energy, EnergyActionType, Equipped, InInventory, Inventory, UnequipItemAction,
        get_energy_cost,
    },
    engine::StableIdRegistry,
};

pub struct TransferItemAction {
    pub from_entity: Entity,
    pub to_entity: Entity,
    pub item_stable_id: u64,
}

impl Command for TransferItemAction {
    fn apply(self, world: &mut World) {
        // Get item entity and to_stable_id first
        let (item_entity, to_stable_id) = {
            let Some(id_registry) = world.get_resource::<StableIdRegistry>() else {
                eprintln!("TransferItemAction: StableIdRegistry not found");
                return;
            };

            let Some(item_entity) = id_registry.get_entity(self.item_stable_id) else {
                eprintln!(
                    "TransferItemAction: Item entity not found for id {}",
                    self.item_stable_id
                );
                return;
            };

            let to_stable_id = id_registry.get_id(self.to_entity).unwrap_or(0);
            (item_entity, to_stable_id)
        };

        // Check and perform the transfer
        {
            let Some(mut from_inventory) = world.get_mut::<Inventory>(self.from_entity) else {
                eprintln!(
                    "TransferItemAction: Entity {:?} has no inventory",
                    self.from_entity
                );
                return;
            };

            if !from_inventory.contains_id(self.item_stable_id) {
                eprintln!(
                    "TransferItemAction: Item {} not found in source inventory",
                    self.item_stable_id
                );
                return;
            }

            from_inventory.remove_item(self.item_stable_id);
        }

        {
            let mut q_equipped = world.query::<&Equipped>();
            if q_equipped.get(world, item_entity).is_ok() {
                UnequipItemAction::new(self.item_stable_id).apply(world);
            }

            let Some(mut to_inventory) = world.get_mut::<Inventory>(self.to_entity) else {
                eprintln!(
                    "TransferItemAction: Entity {:?} has no inventory",
                    self.to_entity
                );
                return;
            };

            if to_inventory.is_full() {
                eprintln!("TransferItemAction: Target inventory is full");
                // Re-add to source inventory since transfer failed
                if let Some(mut from_inventory) = world.get_mut::<Inventory>(self.from_entity) {
                    from_inventory.add_item(self.item_stable_id);
                }
                return;
            }

            to_inventory.add_item(self.item_stable_id);
        }

        // Update the InInventory component
        if let Some(mut in_inventory) = world.get_mut::<InInventory>(item_entity) {
            in_inventory.owner_id = to_stable_id;
        }

        // Consume energy if from_entity has energy (for player actions)
        if let Some(mut energy) = world.get_mut::<Energy>(self.from_entity) {
            let cost = get_energy_cost(EnergyActionType::TransferItem);
            energy.consume_energy(cost);
        }
    }
}
