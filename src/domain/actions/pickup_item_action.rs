use bevy_ecs::prelude::*;

use crate::{
    domain::{
        Energy, EnergyActionType, InInventory, Inventory, Item, StackCount, Stackable,
        StackableType, Zone, get_energy_cost, inventory::InventoryChangedEvent,
    },
    engine::StableIdRegistry,
    rendering::Position,
};

pub struct PickupItemAction {
    pub entity: Entity,
    pub item_stable_id: u64,
    pub spend_energy: bool,
}

impl Command for PickupItemAction {
    fn apply(self, world: &mut World) {
        let Some(id_registry) = world.get_resource::<StableIdRegistry>() else {
            eprintln!("PickupItemAction: StableIdRegistry not found");
            return;
        };

        let Some(item_entity) = id_registry.get_entity(self.item_stable_id) else {
            return;
        };
        let Some(entity_stable_id) = id_registry.get_id(self.entity) else {
            return;
        };

        // Check if item is stackable first (before any mutable borrows)
        let stackable_info = world.get::<Stackable>(item_entity).map(|stackable| {
            let stack_type = stackable.stack_type;
            let pickup_count = world
                .get::<StackCount>(item_entity)
                .map(|sc| sc.count)
                .unwrap_or(1);
            (stack_type, pickup_count)
        });

        let mut q_inventory = world.query::<&mut Inventory>();

        let Ok(inventory) = q_inventory.get_mut(world, self.entity) else {
            return;
        };

        if let Some((stack_type, pickup_count)) = stackable_info {
            // Collect inventory items to avoid borrow conflicts
            let item_ids = inventory.item_ids.clone();

            // Find existing stack of same type in inventory
            if let Some(existing_entity) = find_existing_stack(world, &item_ids, stack_type)
                && let Some(mut stack_count) = world.get_mut::<StackCount>(existing_entity)
            {
                let overflow = stack_count.add(pickup_count);

                if overflow == 0 {
                    // All items fit in existing stack - despawn picked up item
                    remove_item_from_zone(world, item_entity);
                    world.entity_mut(item_entity).despawn();

                    // Consume energy
                    if self.spend_energy
                        && let Some(mut energy) = world.get_mut::<Energy>(self.entity)
                    {
                        let cost = get_energy_cost(EnergyActionType::PickUpItem);
                        energy.consume_energy(cost);
                    }
                    return;
                } else {
                    // Partial pickup - update the item on ground with remaining count
                    if let Some(mut ground_stack) = world.get_mut::<StackCount>(item_entity) {
                        ground_stack.count = overflow;
                    }

                    // Consume energy for partial pickup
                    if self.spend_energy
                        && let Some(mut energy) = world.get_mut::<Energy>(self.entity)
                    {
                        let cost = get_energy_cost(EnergyActionType::PickUpItem);
                        energy.consume_energy(cost);
                    }
                    return;
                }
            }
        }

        // Not stackable or no existing stack - use normal pickup logic
        // Re-get inventory and queries if we dropped them for stackable handling
        let mut q_inventory = world.query::<&mut Inventory>();
        let mut q_items = world.query::<&Position>();
        let mut q_zones = world.query::<&mut Zone>();

        // Get item weight before borrowing inventory
        let item_weight = if let Some(item) = world.get::<Item>(item_entity) {
            item.weight
        } else {
            1.0 // Default weight if Item component missing
        };

        let Ok(mut inventory) = q_inventory.get_mut(world, self.entity) else {
            return;
        };

        if !inventory.add_item(self.item_stable_id, item_weight) {
            return;
        }

        if let Ok(position) = q_items.get(world, item_entity) {
            let zone_idx = position.zone_idx();

            for mut zone in q_zones.iter_mut(world) {
                if zone.idx == zone_idx {
                    zone.entities.remove(&item_entity);
                    break;
                }
            }
        }

        world
            .entity_mut(item_entity)
            .remove::<Position>()
            .remove::<ChildOf>()
            .insert(InInventory::new(entity_stable_id));

        if self.spend_energy
            && let Some(mut energy) = world.get_mut::<Energy>(self.entity)
        {
            let cost = get_energy_cost(EnergyActionType::PickUpItem);
            energy.consume_energy(cost);
        }

        world.send_event(InventoryChangedEvent);
    }
}

fn find_existing_stack(
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

fn remove_item_from_zone(world: &mut World, item_entity: Entity) {
    if let Ok(position) = world.query::<&Position>().get(world, item_entity) {
        let zone_idx = position.zone_idx();

        for mut zone in world.query::<&mut Zone>().iter_mut(world) {
            if zone.idx == zone_idx {
                zone.entities.remove(&item_entity);
                break;
            }
        }
    }
}
