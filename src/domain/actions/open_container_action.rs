use bevy_ecs::prelude::*;
use macroquad::prelude::trace;

use crate::{
    common::Rand,
    domain::{
        InInventory, Inventory, LootTableRegistry, PickupItemAction, Prefab, Prefabs,
        UnopenedContainer,
    },
    engine::{StableId, StableIdRegistry},
    rendering::Position,
    states::{CurrentGameState, GameState},
};

pub struct OpenContainerAction {
    pub player_entity: Entity,
    pub container_entity: Entity,
}

impl Command for OpenContainerAction {
    fn apply(self, world: &mut World) {
        // Check if this is an unopened container and generate loot if needed
        if let Some(unopened) = world.get::<UnopenedContainer>(self.container_entity) {
            let loot_table_id = unopened.0;
            generate_container_loot(world, self.container_entity, loot_table_id);

            // Remove the UnopenedContainer component to mark it as opened
            world
                .entity_mut(self.container_entity)
                .remove::<UnopenedContainer>();
        }

        world.insert_resource(crate::states::ContainerContext {
            player_entity: self.player_entity,
            container_entity: self.container_entity,
        });

        if let Some(mut game_state) = world.get_resource_mut::<CurrentGameState>() {
            game_state.next = GameState::Container;
        }
    }
}

fn generate_container_loot(
    world: &mut World,
    container_entity: Entity,
    loot_table_id: crate::domain::LootTableId,
) {
    // Get container position for seeding randomness
    let container_pos =
        if let Some(position) = world.get::<crate::rendering::Position>(container_entity) {
            (position.x as u32, position.y as u32, position.z as u32)
        } else {
            (0, 0, 0) // fallback
        };

    let mut rand = Rand::seed(container_pos.0 + container_pos.1 + container_pos.2);

    // Get the container's inventory capacity
    let inventory_capacity = if let Some(inventory) = world.get::<Inventory>(container_entity) {
        inventory.capacity.saturating_sub(inventory.item_ids.len())
    } else {
        return; // No inventory component
    };

    // Determine how many items to spawn (1-3 items)
    let item_count = rand.range_n(1, 4) as usize;
    let max_items = item_count.min(inventory_capacity);

    // Generate items from the loot table
    let item_prefabs = {
        let loot_registry = world.resource::<LootTableRegistry>();
        loot_registry.roll_multiple(loot_table_id, max_items, &mut rand)
    };

    for item_prefab_id in item_prefabs {
        trace!("Spawning item in container!");

        // Spawn the item prefab and get the entity it creates
        let item_config = Prefab::new(
            item_prefab_id.clone(),
            (
                container_pos.0 as usize,
                container_pos.1 as usize,
                container_pos.2 as usize,
            ),
        );
        let _ = Prefabs::spawn_in_container(world, item_config, container_entity);
    }
}
