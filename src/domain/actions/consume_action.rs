use bevy_ecs::prelude::*;

use crate::{
    domain::{
        Consumable, ConsumableEffect, Energy, EnergyActionType, Health, InInventory, Inventory,
        Item, Level, StackCount, Stackable, Stats,
        actions::GameAction,
        get_base_energy_cost,
        inventory::InventoryChangedEvent,
        systems::{
            destruction_system::{DestructionCause, EntityDestroyedEvent},
            game_log_system::{GameLogEvent, KnowledgeLevel, LogMessage},
        },
    },
    engine::{Clock, StableId, StableIdRegistry},
    rendering::Position,
};

use super::stack_split_util::split_item_from_stack;

pub struct ConsumeAction {
    pub item_id: u64,
    pub consumer_id: u64,
}

impl ConsumeAction {
    pub fn new(item_id: u64, consumer_id: u64) -> Self {
        Self {
            item_id,
            consumer_id,
        }
    }
}

impl GameAction for ConsumeAction {
    fn try_apply(self, world: &mut World) -> bool {
        // Get entities from registry
        let (consumer_entity, item_entity) = {
            let Some(registry) = world.get_resource::<StableIdRegistry>() else {
                return false;
            };

            let Some(consumer_entity) = registry.get_entity(StableId(self.consumer_id)) else {
                return false;
            };

            let Some(item_entity) = registry.get_entity(StableId(self.item_id)) else {
                return false;
            };

            (consumer_entity, item_entity)
        };

        // Check if item is consumable
        let consumable = {
            let Some(consumable) = world.get::<Consumable>(item_entity) else {
                return false;
            };
            consumable.clone()
        };

        // Check if consumer has the item in inventory
        {
            let Some(inventory) = world.get::<Inventory>(consumer_entity) else {
                return false;
            };

            if !inventory.contains_id(self.item_id) {
                return false;
            }
        }

        // Handle item consumption
        if consumable.consume_on_use {
            let item_to_destroy = if world.get::<Stackable>(item_entity).is_some()
                && world.get::<StackCount>(item_entity).is_some()
            {
                split_item_from_stack(world, item_entity, consumer_entity)
            } else {
                Some(item_entity)
            };

            if let Some(entity_to_destroy) = item_to_destroy {
                let item_weight = world
                    .get::<Item>(entity_to_destroy)
                    .map(|item| item.weight)
                    .unwrap_or(1.0);

                let item_stable_id = world
                    .get::<StableId>(entity_to_destroy)
                    .map(|id| id.0)
                    .unwrap_or(self.item_id);

                let position = world
                    .get::<Position>(consumer_entity)
                    .map(|p| p.world())
                    .unwrap_or((0, 0, 0));

                if let Some(mut inventory) = world.get_mut::<Inventory>(consumer_entity) {
                    inventory.remove_item(item_stable_id, item_weight);
                }

                world.entity_mut(entity_to_destroy).remove::<InInventory>();

                world.send_event(EntityDestroyedEvent::new(
                    entity_to_destroy,
                    position,
                    DestructionCause::Consumed,
                ));
            }
        }

        // Apply the consumable effect and generate description
        let effect_desc = match consumable.effect {
            ConsumableEffect::Heal(amount) => {
                // Check if we have all required components first
                if world.get::<Health>(consumer_entity).is_some()
                    && world.get::<Level>(consumer_entity).is_some()
                    && world.get::<Stats>(consumer_entity).is_some()
                {
                    let level = world.get::<Level>(consumer_entity).unwrap().clone();
                    let stats = world.get::<Stats>(consumer_entity).unwrap().clone();
                    let mut health = world.get_mut::<Health>(consumer_entity).unwrap();
                    health.heal(amount, &level, &stats);
                }
                format!("restored {} health", amount)
            }
            ConsumableEffect::RestoreArmor(amount) => {
                if world.get::<Health>(consumer_entity).is_some()
                    && world.get::<Stats>(consumer_entity).is_some()
                {
                    let stats = world.get::<Stats>(consumer_entity).unwrap().clone();
                    let mut health = world.get_mut::<Health>(consumer_entity).unwrap();
                    let (_, max_armor) = health.get_current_max_armor(&stats);
                    health.current_armor = (health.current_armor + amount).min(max_armor);
                }
                format!("restored {} armor", amount)
            }
            ConsumableEffect::Poison(_damage, _duration) => {
                // TODO: Implement poison system when status effects are added
                "poisoned".to_string()
            }
            ConsumableEffect::Buff(_stat, _amount, _duration) => {
                // TODO: Implement buff system when temporary effects are added
                "gained buff".to_string()
            }
            ConsumableEffect::Cure => {
                // TODO: Implement cure system when status effects are added
                "cured status effects".to_string()
            }
        };

        // Send game log event
        let current_tick = world
            .get_resource::<Clock>()
            .map(|c| c.get_tick())
            .unwrap_or(0);
        world.send_event(GameLogEvent {
            message: LogMessage::ItemConsumed {
                consumer: consumer_entity,
                item: item_entity,
                effect_desc,
            },
            tick: current_tick,
            knowledge: KnowledgeLevel::Player,
        });

        // Consume energy
        if let Some(mut energy) = world.get_mut::<Energy>(consumer_entity) {
            let cost = get_base_energy_cost(EnergyActionType::Eat);
            energy.consume_energy(cost);
        }

        // Send inventory changed event
        world.send_event(InventoryChangedEvent);

        true
    }
}

impl Command for ConsumeAction {
    fn apply(self, world: &mut World) {
        self.try_apply(world);
    }
}
