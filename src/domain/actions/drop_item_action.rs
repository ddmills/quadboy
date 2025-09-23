use bevy_ecs::prelude::*;
use macroquad::prelude::trace;

use crate::{
    domain::{
        Collider, DynamicEntity, Energy, EnergyActionType, Equipped, InInventory, Inventory, Item,
        StaticEntity, StaticEntitySpawnedEvent, UnequipItemAction, get_base_energy_cost, inventory::InventoryChangedEvent,
    },
    engine::StableIdRegistry,
    rendering::Position,
};

pub struct DropItemAction {
    pub entity: Entity,
    pub item_stable_id: u64,
    pub drop_position: (usize, usize, usize),
}

impl Command for DropItemAction {
    fn apply(self, world: &mut World) {
        let Some(id_registry) = world.get_resource::<StableIdRegistry>() else {
            eprintln!("DropItemAction: StableIdRegistry not found");
            return;
        };

        let Some(item_entity) = id_registry.get_entity(self.item_stable_id) else {
            eprintln!(
                "DropItemAction: Item entity not found for id {}",
                self.item_stable_id
            );
            return;
        };

        let mut q_equipped = world.query::<&Equipped>();

        if q_equipped.get(world, item_entity).is_ok() {
            trace!("eq? {}", self.item_stable_id);
            UnequipItemAction::new(self.item_stable_id).apply(world);
        }

        // Get item weight before removing from inventory
        let item_weight = world
            .get::<Item>(item_entity)
            .map(|item| item.weight)
            .unwrap_or(1.0);

        let Some(mut inventory) = world.get_mut::<Inventory>(self.entity) else {
            eprintln!("DropItemAction: Entity {:?} has no inventory", self.entity);
            return;
        };

        // Use the inventory's remove_item method to properly handle weight tracking
        if !inventory.remove_item(self.item_stable_id, item_weight) {
            eprintln!(
                "DropItemAction: Item {} not found in inventory",
                self.item_stable_id
            );
            return;
        };

        let position = Position::new_world(self.drop_position);
        world
            .entity_mut(item_entity)
            .insert(position.clone())
            .remove::<InInventory>();

        // Note: Stackable items keep their StackCount component when dropped

        // Check if item has StaticEntity or DynamicEntity component
        let has_static_entity = world.get::<StaticEntity>(item_entity).is_some();
        let has_dynamic_entity = world.get::<DynamicEntity>(item_entity).is_some();
        let collider_flags = world.get::<Collider>(item_entity).map(|c| c.flags);

        if has_static_entity {
            // Fire StaticEntitySpawnedEvent for proper static entity placement
            world.send_event(StaticEntitySpawnedEvent {
                entity: item_entity,
                position,
                collider_flags,
            });
        } else if has_dynamic_entity {
            // Dynamic entities are handled by update_dynamic_entity_pos system
            // Just adding Position component triggers the system
        } else {
            // Item doesn't have tracking component (created in inventory), add StaticEntity
            world.entity_mut(item_entity).insert(StaticEntity);
            world.send_event(StaticEntitySpawnedEvent {
                entity: item_entity,
                position,
                collider_flags,
            });
        }

        if let Some(mut energy) = world.get_mut::<Energy>(self.entity) {
            let cost = get_base_energy_cost(EnergyActionType::DropItem);
            energy.consume_energy(cost);
        }

        world.send_event(InventoryChangedEvent);
    }
}
