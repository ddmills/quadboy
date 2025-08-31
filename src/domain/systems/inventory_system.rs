use bevy_ecs::prelude::*;

use crate::{
    domain::{InInventory, Inventory, Item, Label, Player},
    engine::{KeyInput, StableId},
    rendering::Position,
};
use macroquad::input::KeyCode;

#[derive(Event)]
pub struct PickupEvent {
    pub item_name: String,
}

pub fn handle_item_pickup(
    mut cmds: Commands,
    mut q_player: Query<(Entity, &Position, &mut Inventory, &StableId), With<Player>>,
    q_items: Query<(Entity, &Position, &StableId, Option<&Label>), (With<Item>, Without<InInventory>)>,
    keys: Res<KeyInput>,
    mut e_pickup: EventWriter<PickupEvent>,
) {
    if !keys.is_pressed(KeyCode::F) {
        return;
    }

    let Ok((_player_entity, player_pos, mut inventory, player_stable_id)) = q_player.single_mut() else {
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