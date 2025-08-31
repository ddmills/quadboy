use bevy_ecs::prelude::*;

use crate::{
    domain::{InInventory, Inventory, Item, Label, Player, Zone},
    engine::{KeyInput, StableId},
    rendering::{Position, world_to_zone_idx, world_to_zone_local},
};
use macroquad::input::KeyCode;

#[derive(Event)]
pub struct PickupEvent {
    pub item_name: String,
}

#[derive(Event)]
pub struct DropEvent {
    pub count: usize,
}

pub fn handle_item_pickup(
    mut cmds: Commands,
    mut q_player: Query<(Entity, &Position, &mut Inventory, &StableId), With<Player>>,
    q_items: Query<
        (Entity, &Position, &StableId, Option<&Label>),
        (With<Item>, Without<InInventory>),
    >,
    keys: Res<KeyInput>,
    mut e_pickup: EventWriter<PickupEvent>,
) {
    if !keys.is_pressed(KeyCode::F) {
        return;
    }

    let Ok((_player_entity, player_pos, mut inventory, player_stable_id)) = q_player.single_mut()
    else {
        return;
    };

    let player_world_pos = player_pos.world();

    for (item_entity, item_pos, item_stable_id, item_label) in q_items.iter() {
        let item_world_pos = item_pos.world();

        if player_world_pos == item_world_pos {
            if inventory.is_full() {
                // TODO: Show "Inventory is full!" message
                return;
            }

            if inventory.add_item(item_entity, item_stable_id.0) {
                let item_name = item_label
                    .map(|l| l.get().to_string())
                    .unwrap_or_else(|| "item".to_string());

                cmds.entity(item_entity)
                    .remove::<Position>()
                    .insert(InInventory::new(player_stable_id.0));

                e_pickup.write(PickupEvent { item_name });
                return; // Only pick up one item per key press
            }
        }
    }
}

pub fn handle_drop_all_items(
    mut cmds: Commands,
    mut q_player: Query<(&Position, &mut Inventory), With<Player>>,
    mut q_zones: Query<&mut Zone>,
    keys: Res<KeyInput>,
    mut e_drop: EventWriter<DropEvent>,
) {
    if !keys.is_pressed(KeyCode::R) {
        return;
    }

    let Ok((player_pos, mut inventory)) = q_player.single_mut() else {
        return;
    };

    if inventory.items.is_empty() {
        return;
    }

    let drop_pos = player_pos.clone();
    let world_pos = drop_pos.world();
    let zone_idx = world_to_zone_idx(world_pos.0, world_pos.1, world_pos.2);
    let (local_x, local_y) = world_to_zone_local(world_pos.0, world_pos.1);

    // Find the zone to add items back to
    let mut zone_found = false;
    for mut zone in q_zones.iter_mut() {
        if zone.idx == zone_idx {
            // Drop all items at player's position
            let items_to_drop = inventory.items.clone();
            let count = items_to_drop.len();

            for item_entity in items_to_drop {
                // Restore Position component
                cmds.entity(item_entity)
                    .insert(drop_pos.clone())
                    .remove::<InInventory>();

                // Add back to zone's entity grid
                zone.entities.insert(local_x, local_y, item_entity);
            }

            zone_found = true;

            // Clear the inventory
            inventory.items.clear();
            inventory.item_ids.clear();

            // Send drop event
            e_drop.write(DropEvent { count });
            break;
        }
    }

    if !zone_found {
        println!(
            "Warning: Could not find zone {} to drop items into",
            zone_idx
        );
    }
}
