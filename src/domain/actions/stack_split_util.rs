use bevy_ecs::prelude::*;

use crate::{
    domain::{
        Inventory, Item, Prefab, PrefabId, Prefabs, StackCount, Stackable,
        inventory::InventoryChangedEvent,
    },
    engine::{StableId, StableIdRegistry},
    rendering::Position,
};

/// Splits a single item from a stack, creating a new entity if the stack size > 1
/// Returns the entity that should be used for the action (new single item or original if stack size was 1)
pub fn split_item_from_stack(
    world: &mut World,
    item_entity: Entity,
    inventory_owner: Entity,
) -> Option<Entity> {
    // Check if item has a stack count
    let stack_count = world.get::<StackCount>(item_entity)?;

    if stack_count.count <= 1 {
        // Already a single item, return as-is
        return Some(item_entity);
    }

    // Get necessary components from the original item
    let stackable = world.get::<Stackable>(item_entity)?.clone();
    let item_weight = world.get::<Item>(item_entity)?.weight;
    let position = world.get::<Position>(inventory_owner).map(|p| p.world());

    // Decrement the original stack
    {
        let mut original_stack = world.get_mut::<StackCount>(item_entity)?;
        original_stack.count -= 1;
    }

    // Determine the PrefabId based on stackable type
    let prefab_id = match stackable.stack_type {
        crate::domain::StackableType::Dynamite => PrefabId::Dynamite,
        crate::domain::StackableType::Apple => PrefabId::Apple,
        crate::domain::StackableType::GoldNugget => PrefabId::GoldNugget,
    };

    // Create new single item at the inventory owner's position (for safety)
    let spawn_position = position.unwrap_or((0, 0, 0));
    let config = Prefab::new(prefab_id, spawn_position);
    let new_entity = Prefabs::spawn_world(world, config);

    // Set the new item's stack count to 1
    if let Some(mut new_stack) = world.get_mut::<StackCount>(new_entity) {
        new_stack.count = 1;
    }

    // Manually assign a StableId immediately since the auto-assign system runs later
    let stable_id = {
        let Some(mut registry) = world.get_resource_mut::<StableIdRegistry>() else {
            return None;
        };
        let id = registry.generate_id();
        registry.register(new_entity, id);
        world.entity_mut(new_entity).insert(StableId::new(id));
        id
    };

    // Add the new item to inventory
    if let Some(mut inventory) = world.get_mut::<Inventory>(inventory_owner) {
        inventory.add_item(stable_id, item_weight);
    }

    // Send inventory changed event
    world.send_event(InventoryChangedEvent);

    Some(new_entity)
}
