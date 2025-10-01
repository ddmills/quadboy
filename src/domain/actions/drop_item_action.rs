use bevy_ecs::prelude::*;
use macroquad::prelude::trace;

use crate::{
    domain::{
        Collider, DynamicEntity, Energy, EnergyActionType, Equipped, InInventory, Inventory, Item,
        Player, StaticEntity, StaticEntitySpawnedEvent, UnequipItemAction,
        actions::GameAction,
        get_base_energy_cost,
        inventory::InventoryChangedEvent,
        systems::game_log_system::{GameLogEvent, KnowledgeLevel, LogMessage},
    },
    engine::{Clock, StableId, StableIdRegistry},
    rendering::Position,
};

pub struct DropItemAction {
    pub entity: Entity,
    pub item_stable_id: StableId,
    pub drop_position: (usize, usize, usize),
}

impl GameAction for DropItemAction {
    fn try_apply(self, world: &mut World) -> bool {
        let Some(id_registry) = world.get_resource::<StableIdRegistry>() else {
            return false;
        };

        let Some(item_entity) = id_registry.get_entity(self.item_stable_id) else {
            return false;
        };

        let mut q_equipped = world.query::<&Equipped>();

        if q_equipped.get(world, item_entity).is_ok() {
            trace!("eq? {}", self.item_stable_id.0);
            UnequipItemAction::new(self.item_stable_id.0).apply(world);
        }

        // Get item weight before removing from inventory
        let item_weight = world
            .get::<Item>(item_entity)
            .map(|item| item.weight)
            .unwrap_or(1.0);

        let Some(mut inventory) = world.get_mut::<Inventory>(self.entity) else {
            return false;
        };

        // Use the inventory's remove_item method to properly handle weight tracking
        if !inventory.remove_item(self.item_stable_id.0, item_weight) {
            return false;
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

        // Send drop log event
        let knowledge = if world.get::<Player>(self.entity).is_some() {
            KnowledgeLevel::Player
        } else {
            KnowledgeLevel::Action {
                actor: self.entity,
                location: self.drop_position,
            }
        };

        world.send_event(GameLogEvent {
            message: LogMessage::ItemDrop {
                dropper: self.entity,
                item: item_entity,
                quantity: None, // TODO: Add stack count support if needed
            },
            tick: world.resource::<Clock>().current_tick(),
            knowledge,
        });

        world.send_event(InventoryChangedEvent);

        true
    }
}

impl Command for DropItemAction {
    fn apply(self, world: &mut World) {
        self.try_apply(world);
    }
}
