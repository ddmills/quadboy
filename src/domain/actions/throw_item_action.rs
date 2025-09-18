use bevy_ecs::prelude::*;

use crate::{
    Rand,
    domain::{
        Collider, Energy, EnergyActionType, Equipped, InInventory, Inventory, Item, Prefab,
        PrefabId, Prefabs, StackCount, Stackable, Throwable, UnequipItemAction, Zone,
        get_base_energy_cost, inventory::InventoryChangedEvent,
    },
    engine::{StableId, StableIdRegistry},
    rendering::{Position, spawn_throw_trail_in_world, world_to_zone_idx, world_to_zone_local},
};

pub struct ThrowItemAction {
    pub thrower_entity: Entity,
    pub item_stable_id: u64,
    pub target_position: (usize, usize, usize),
}

impl Command for ThrowItemAction {
    fn apply(self, world: &mut World) {
        let Some(id_registry) = world.get_resource::<StableIdRegistry>() else {
            eprintln!("ThrowItemAction: StableIdRegistry not found");
            return;
        };

        let Some(item_entity) = id_registry.get_entity(self.item_stable_id) else {
            eprintln!(
                "ThrowItemAction: Item entity not found for id {}",
                self.item_stable_id
            );
            return;
        };

        // Unequip item if it's equipped
        let mut q_equipped = world.query::<&Equipped>();
        if q_equipped.get(world, item_entity).is_ok() {
            UnequipItemAction::new(self.item_stable_id).apply(world);
        }

        // Get item weight before processing
        let item_weight = world
            .get::<Item>(item_entity)
            .map(|item| item.weight)
            .unwrap_or(1.0);

        // Check if this is a stackable item
        let stack_count = world
            .get::<StackCount>(item_entity)
            .map(|sc| sc.count)
            .unwrap_or(1);

        let thrown_item_entity = if stack_count > 1 {
            // Decrement the original stack
            if let Some(mut original_stack) = world.get_mut::<StackCount>(item_entity) {
                original_stack.count -= 1;
            }

            // Create a new single item to throw
            let stackable = world.get::<Stackable>(item_entity).cloned();

            if let Some(stackable) = stackable {
                // Determine the PrefabId based on stackable type
                let prefab_id = match stackable.stack_type {
                    crate::domain::StackableType::Dynamite => PrefabId::Dynamite,
                    crate::domain::StackableType::Apple => PrefabId::Apple,
                    crate::domain::StackableType::GoldNugget => PrefabId::GoldNugget,
                };

                // Create new single item at the target position
                let config = Prefab::new(prefab_id, self.target_position);
                let new_entity = Prefabs::spawn_world(world, config);

                // Set the new item's stack count to 1
                if let Some(mut new_stack) = world.get_mut::<StackCount>(new_entity) {
                    new_stack.count = 1;
                }

                new_entity
            } else {
                // Fallback to original behavior if not stackable
                item_entity
            }
        } else {
            // Single item or last item in stack - remove from inventory and throw original
            let Some(mut inventory) = world.get_mut::<Inventory>(self.thrower_entity) else {
                eprintln!(
                    "ThrowItemAction: Entity {:?} has no inventory",
                    self.thrower_entity
                );
                return;
            };

            if !inventory.remove_item(self.item_stable_id, item_weight) {
                eprintln!(
                    "ThrowItemAction: Item {} not found in inventory",
                    self.item_stable_id
                );
                return;
            }

            item_entity
        };

        // Get thrower position and item glyph for trail effect
        let thrower_position = world
            .get::<Position>(self.thrower_entity)
            .map(|pos| pos.world())
            .unwrap_or((0, 0, 0));

        let (item_glyph, item_color) = world
            .get::<Throwable>(thrown_item_entity)
            .map(|throwable| (throwable.particle_char, throwable.throwable_fg1))
            .unwrap_or(('?', 0xFFFFFF));

        // Spawn throw trail effect
        {
            let mut temp_rand = Rand::default();
            spawn_throw_trail_in_world(
                world,
                thrower_position,
                self.target_position,
                15.0, // throw speed
                item_glyph,
                item_color,
                &mut temp_rand,
            );
        }

        // Place thrown item at target position
        let position = Position::new_world(self.target_position);
        world
            .entity_mut(thrown_item_entity)
            .insert(position)
            .remove::<InInventory>();

        // Add item to target zone's entity grid
        let zone_idx = world_to_zone_idx(
            self.target_position.0,
            self.target_position.1,
            self.target_position.2,
        );
        let (local_x, local_y) =
            world_to_zone_local(self.target_position.0, self.target_position.1);

        let has_collider = world.get::<Collider>(thrown_item_entity).is_some();
        let mut zone_found = false;
        let mut zones = world.query::<&mut Zone>();
        for mut zone in zones.iter_mut(world) {
            if zone.idx == zone_idx {
                zone.entities.insert(local_x, local_y, thrown_item_entity);

                if has_collider {
                    zone.entities.insert(local_x, local_y, thrown_item_entity);
                }

                zone_found = true;
                break;
            }
        }

        if !zone_found {
            eprintln!(
                "ThrowItemAction: Could not find zone {} to throw item into",
                zone_idx
            );
        }

        // Consume energy from thrower
        if let Some(mut energy) = world.get_mut::<Energy>(self.thrower_entity) {
            let cost = get_base_energy_cost(EnergyActionType::Throw);
            energy.consume_energy(cost);
        }

        // Send inventory changed event
        world.send_event(InventoryChangedEvent);
    }
}
