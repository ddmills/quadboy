use bevy_ecs::prelude::*;

use crate::{
    domain::{Energy, EnergyActionType, EquipmentSlots, Equipped, get_energy_cost},
    engine::StableIdRegistry,
};

pub struct UnequipItemAction {
    pub item_id: u64,
}

impl UnequipItemAction {
    pub fn new(item_id: u64) -> Self {
        Self { item_id }
    }
}

impl Command for UnequipItemAction {
    fn apply(self, world: &mut World) {
        // Get item entity from registry
        let item_entity = {
            let Some(registry) = world.get_resource::<StableIdRegistry>() else {
                return;
            };

            let Some(item_entity) = registry.get_entity(self.item_id) else {
                return;
            };

            item_entity
        };

        // Get equipped info from the item to find owner and slots
        let (owner_id, slots) = {
            let Some(equipped) = world.get::<Equipped>(item_entity) else {
                return; // Item is not equipped
            };
            (equipped.owner_id, equipped.slots.clone())
        };

        // Get owner entity from registry
        let owner_entity = {
            let Some(registry) = world.get_resource::<StableIdRegistry>() else {
                return;
            };

            let Some(owner_entity) = registry.get_entity(owner_id) else {
                return;
            };

            owner_entity
        };

        // Remove item from owner's equipment slots
        {
            let Some(mut equipment_slots) = world.get_mut::<EquipmentSlots>(owner_entity) else {
                return;
            };

            for slot in slots {
                equipment_slots.unequip(slot);
            }
        }

        // Remove Equipped component from item
        world.entity_mut(item_entity).remove::<Equipped>();

        // Consume energy if owner entity has energy (for player actions)
        if let Some(mut energy) = world.get_mut::<Energy>(owner_entity) {
            let cost = get_energy_cost(EnergyActionType::UnequipItem);
            energy.consume_energy(cost);
        }
    }
}
