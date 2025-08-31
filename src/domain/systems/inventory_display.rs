use bevy_ecs::prelude::*;

use crate::{
    common::Palette,
    domain::{DropEvent, Inventory, PickupEvent, Player},
    engine::Time,
    rendering::{Position, Text},
    states::CleanupStateExplore,
};

#[derive(Component)]
pub struct PickupMessage {
    pub lifetime: f64,
}

pub fn display_pickup_messages(
    mut cmds: Commands,
    mut e_pickup: EventReader<PickupEvent>,
    time: Res<Time>,
) {
    for event in e_pickup.read() {
        let message = format!("Picked up {}", event.item_name);

        cmds.spawn((
            Text::new(&message).fg1(Palette::Yellow).bg(Palette::Black),
            Position::new_f32(2., 2., 0.),
            PickupMessage {
                lifetime: time.fixed_t + 2.0, // Show for 2 seconds
            },
            CleanupStateExplore,
        ));
    }
}

pub fn display_drop_messages(
    mut cmds: Commands,
    mut e_drop: EventReader<DropEvent>,
    time: Res<Time>,
) {
    for event in e_drop.read() {
        let message = if event.count == 1 {
            "Dropped 1 item".to_string()
        } else {
            format!("Dropped {} items", event.count)
        };

        cmds.spawn((
            Text::new(&message).fg1(Palette::Cyan).bg(Palette::Black),
            Position::new_f32(2., 2.5, 0.),
            PickupMessage {
                lifetime: time.fixed_t + 2.0, // Show for 2 seconds
            },
            CleanupStateExplore,
        ));
    }
}

pub fn cleanup_old_messages(
    mut cmds: Commands,
    q_messages: Query<(Entity, &PickupMessage)>,
    time: Res<Time>,
) {
    for (entity, message) in q_messages.iter() {
        if time.fixed_t > message.lifetime {
            cmds.entity(entity).despawn();
        }
    }
}

pub fn display_inventory_count(
    mut q_text: Query<&mut Text, With<InventoryDisplay>>,
    q_inventory: Query<&Inventory, With<Player>>,
) {
    let Ok(inventory) = q_inventory.single() else {
        return;
    };

    let Ok(mut text) = q_text.single_mut() else {
        return;
    };

    text.value = format!("Inventory: {}/{}", inventory.count(), inventory.capacity);
}

#[derive(Component)]
pub struct InventoryDisplay;
