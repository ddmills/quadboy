use bevy_ecs::prelude::*;

use crate::{
    domain::{
        Consumable, ConsumableEffect, Energy, EnergyActionType, Health, InInventory, Inventory,
        Item, Level, StackCount, Stackable, Stats, get_base_energy_cost,
        inventory::InventoryChangedEvent,
    },
    engine::{StableId, StableIdRegistry},
};

pub struct EatAction {
    pub item_id: u64,  // Stable ID of the item to eat
    pub eater_id: u64, // Stable ID of the entity eating (usually player)
}

impl EatAction {
    pub fn new(item_id: u64, eater_id: u64) -> Self {
        Self { item_id, eater_id }
    }
}

impl Command for EatAction {
    fn apply(self, world: &mut World) {
        // Get entities from registry
        let (eater_entity, item_entity) = {
            let Some(registry) = world.get_resource::<StableIdRegistry>() else {
                return;
            };

            let Some(eater_entity) = registry.get_entity(StableId(self.eater_id)) else {
                return;
            };

            let Some(item_entity) = registry.get_entity(StableId(self.item_id)) else {
                return;
            };

            (eater_entity, item_entity)
        };

        // Check if item is consumable
        let consumable = {
            let Some(consumable) = world.get::<Consumable>(item_entity) else {
                return;
            };
            consumable.clone()
        };

        // Check if eater has the item in inventory
        {
            let Some(inventory) = world.get::<Inventory>(eater_entity) else {
                return;
            };

            if !inventory.contains_id(self.item_id) {
                return;
            }
        }

        // Apply the consumable effect
        match consumable.effect {
            ConsumableEffect::Heal(amount) => {
                // Check if we have all required components first
                if world.get::<Health>(eater_entity).is_some()
                    && world.get::<Level>(eater_entity).is_some()
                    && world.get::<Stats>(eater_entity).is_some()
                {
                    let level = world.get::<Level>(eater_entity).unwrap().clone();
                    let stats = world.get::<Stats>(eater_entity).unwrap().clone();
                    let mut health = world.get_mut::<Health>(eater_entity).unwrap();
                    health.heal(amount, &level, &stats);
                }
            }
            ConsumableEffect::RestoreEnergy(amount) => {
                if let Some(mut energy) = world.get_mut::<Energy>(eater_entity) {
                    energy.value += amount;
                }
            }
            ConsumableEffect::HealAndEnergy(heal_amount, energy_amount) => {
                // Handle healing first
                if world.get::<Health>(eater_entity).is_some()
                    && world.get::<Level>(eater_entity).is_some()
                    && world.get::<Stats>(eater_entity).is_some()
                {
                    let level = world.get::<Level>(eater_entity).unwrap().clone();
                    let stats = world.get::<Stats>(eater_entity).unwrap().clone();
                    let mut health = world.get_mut::<Health>(eater_entity).unwrap();
                    health.heal(heal_amount, &level, &stats);
                }
                // Handle energy separately
                if let Some(mut energy) = world.get_mut::<Energy>(eater_entity) {
                    energy.value += energy_amount;
                }
            }
            // Other effects can be implemented as needed
            ConsumableEffect::Poison(_damage, _duration) => {
                // TODO: Implement poison system when status effects are added
            }
            ConsumableEffect::Buff(_stat, _amount, _duration) => {
                // TODO: Implement buff system when temporary effects are added
            }
            ConsumableEffect::Cure => {
                // TODO: Implement cure system when status effects are added
            }
        }

        // Handle item consumption
        if consumable.consume_on_use {
            // Get item weight before removing from inventory
            let item_weight = world
                .get::<Item>(item_entity)
                .map(|item| item.weight)
                .unwrap_or(1.0);

            // Check if item is stackable
            if world.get::<Stackable>(item_entity).is_some()
                && world.get::<StackCount>(item_entity).is_some()
            {
                let mut stack_count = world.get_mut::<StackCount>(item_entity).unwrap();
                // Reduce stack count
                stack_count.count = stack_count.count.saturating_sub(1);

                // If stack is empty, remove item from inventory
                if stack_count.count == 0 {
                    if let Some(mut inventory) = world.get_mut::<Inventory>(eater_entity) {
                        inventory.remove_item(self.item_id, item_weight);
                    }
                    // Remove from world
                    world.entity_mut(item_entity).remove::<InInventory>();
                    world.entity_mut(item_entity).despawn();
                }
            } else {
                // Non-stackable item, remove completely
                if let Some(mut inventory) = world.get_mut::<Inventory>(eater_entity) {
                    inventory.remove_item(self.item_id, item_weight);
                }
                // Remove from world
                world.entity_mut(item_entity).remove::<InInventory>();
                world.entity_mut(item_entity).despawn();
            }
        }

        // Consume energy
        if let Some(mut energy) = world.get_mut::<Energy>(eater_entity) {
            let cost = get_base_energy_cost(EnergyActionType::Eat);
            energy.consume_energy(cost);
        }

        // Send inventory changed event
        world.send_event(InventoryChangedEvent);
    }
}
