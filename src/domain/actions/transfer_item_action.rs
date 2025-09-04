use bevy_ecs::prelude::*;

use crate::{
    domain::{
        Energy, EnergyActionType, Equipped, InInventory, Inventory, Item, StackCount, Stackable,
        StackableType, UnequipItemAction, get_energy_cost, inventory::InventoryChangedEvent,
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
        // Get item entity, weight, and to_stable_id first
        let (item_entity, item_weight, to_stable_id) = {
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

            let item_weight = world
                .get::<Item>(item_entity)
                .map(|item| item.weight)
                .unwrap_or(1.0);

            let to_stable_id = id_registry.get_id(self.to_entity).unwrap_or(0);
            (item_entity, item_weight, to_stable_id)
        };

        // Check if item is stackable and handle stack merging
        if let Some(stackable) = world.get::<Stackable>(item_entity) {
            let stack_type = stackable.stack_type;
            let transfer_count = world
                .get::<StackCount>(item_entity)
                .map(|sc| sc.count)
                .unwrap_or(1);

            // Check target inventory for existing stack
            let to_inventory = world.get::<Inventory>(self.to_entity);
            if let Some(to_inventory) = to_inventory
                && let Some(existing_entity) =
                    find_existing_stack_transfer(world, &to_inventory.item_ids, stack_type)
                && let Some(mut stack_count) = world.get_mut::<StackCount>(existing_entity)
            {
                let overflow = stack_count.add(transfer_count);

                if overflow == 0 {
                    // All items fit in existing stack - remove from source and despawn
                    let Some(mut from_inventory) = world.get_mut::<Inventory>(self.from_entity)
                    else {
                        return;
                    };
                    from_inventory.remove_item(self.item_stable_id, item_weight);
                    world.entity_mut(item_entity).despawn();

                    // Consume energy
                    if let Some(mut energy) = world.get_mut::<Energy>(self.from_entity) {
                        let cost = get_energy_cost(EnergyActionType::PickUpItem);
                        energy.consume_energy(cost);
                    }
                    return;
                } else {
                    // Partial transfer - update source item with remaining count
                    if let Some(mut source_stack) = world.get_mut::<StackCount>(item_entity) {
                        source_stack.count = overflow;
                    }

                    // Consume energy for partial transfer
                    if let Some(mut energy) = world.get_mut::<Energy>(self.from_entity) {
                        let cost = get_energy_cost(EnergyActionType::PickUpItem);
                        energy.consume_energy(cost);
                    }
                    return;
                }
            }
        }

        // Not stackable or no existing stack - use normal transfer logic
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

            from_inventory.remove_item(self.item_stable_id, item_weight);
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

            if !to_inventory.has_space_for_weight(item_weight) {
                eprintln!(
                    "TransferItemAction: Target inventory cannot hold item weight ({} kg)",
                    item_weight
                );
                // Re-add to source inventory since transfer failed
                if let Some(mut from_inventory) = world.get_mut::<Inventory>(self.from_entity) {
                    from_inventory.add_item(self.item_stable_id, item_weight);
                }
                return;
            }

            to_inventory.add_item(self.item_stable_id, item_weight);
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

        world.send_event(InventoryChangedEvent);
    }
}

fn find_existing_stack_transfer(
    world: &World,
    item_ids: &[u64],
    stack_type: StackableType,
) -> Option<Entity> {
    let id_registry = world.get_resource::<StableIdRegistry>()?;

    for &id in item_ids {
        if let Some(entity) = id_registry.get_entity(id)
            && let Some(stackable) = world.get::<Stackable>(entity)
            && stackable.stack_type == stack_type
        {
            return Some(entity);
        }
    }
    None
}
