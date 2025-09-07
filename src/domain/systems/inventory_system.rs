use bevy_ecs::prelude::*;

use crate::{
    domain::{InInventory, Item, PickupItemAction, Player},
    engine::{KeyInput, StableId},
    rendering::Position,
};
use macroquad::input::KeyCode;

pub fn handle_item_pickup(
    mut cmds: Commands,
    mut q_player: Query<(Entity, &Position), With<Player>>,
    q_items: Query<(&Position, &StableId), (With<Item>, Without<InInventory>)>,
    keys: Res<KeyInput>,
) {
    if !keys.is_pressed(KeyCode::G) {
        return;
    }

    let Ok((player_entity, player_pos)) = q_player.single_mut() else {
        return;
    };

    let player_world_pos = player_pos.world();

    for (item_pos, item_stable_id) in q_items.iter() {
        let item_world_pos = item_pos.world();

        if player_world_pos == item_world_pos {
            cmds.queue(PickupItemAction {
                entity: player_entity,
                item_stable_id: item_stable_id.0,
                spend_energy: true,
            });
            return;
        }
    }
}
