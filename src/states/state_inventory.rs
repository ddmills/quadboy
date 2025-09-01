use bevy_ecs::prelude::*;
use macroquad::input::KeyCode;

use crate::{
    common::Palette,
    domain::{DropItemAction, Inventory, Label, Player, PlayerPosition, game_loop},
    engine::{App, KeyInput, Plugin, StableIdRegistry},
    rendering::{Layer, Position, Text},
    states::{CurrentGameState, GameState, GameStatePlugin, cleanup_system},
};

#[derive(Component)]
pub struct CleanupStateInventory;

#[derive(Component)]
pub struct CleanupStateContainer;

#[derive(Component)]
pub struct InventorySlot {
    pub index: usize,
    pub item_entity: Option<Entity>,
}

#[derive(Component)]
pub struct InventoryCursor {
    pub index: usize,
    pub max_index: usize,
    pub is_player_side: bool,
}

#[derive(Resource)]
pub struct InventoryContext {
    pub player_entity: Entity,
    pub container_entity: Option<Entity>,
}

pub struct InventoryStatePlugin;

impl Plugin for InventoryStatePlugin {
    fn build(&self, app: &mut App) {
        GameStatePlugin::new(GameState::Inventory)
            .on_enter(app, setup_inventory_screen)
            .on_update(app, (handle_inventory_input, game_loop))
            .on_leave(app, cleanup_system::<CleanupStateInventory>);

        GameStatePlugin::new(GameState::Container)
            .on_enter(app, setup_container_screen)
            .on_update(app, (update_container_display, handle_container_input))
            .on_leave(app, cleanup_system::<CleanupStateContainer>);
    }
}

fn setup_inventory_screen(
    mut cmds: Commands,
    q_player: Query<Entity, With<Player>>,
    q_inventory: Query<&Inventory>,
    q_labels: Query<&Label>,
    id_registry: Res<StableIdRegistry>,
) {
    let Ok(player_entity) = q_player.single() else {
        return;
    };

    cmds.insert_resource(InventoryContext {
        player_entity,
        container_entity: None,
    });

    let Ok(inventory) = q_inventory.get(player_entity) else {
        return;
    };

    // Left side - Player inventory
    let left_x = 2.0;
    let right_x = 15.0;

    cmds.spawn((
        Text::new("PLAYER INVENTORY")
            .fg1(Palette::Yellow)
            .bg(Palette::Black)
            .layer(Layer::Ui),
        Position::new_f32(left_x, 1., 0.),
        CleanupStateInventory,
    ));

    // Right side placeholder
    cmds.spawn((
        Text::new("CONTAINER").fg1(Palette::Gray).layer(Layer::Ui),
        Position::new_f32(right_x, 1., 0.),
        CleanupStateInventory,
    ));

    cmds.spawn((
        Text::new("No container open")
            .fg1(Palette::Gray)
            .layer(Layer::Ui),
        Position::new_f32(right_x, 2., 0.),
        CleanupStateInventory,
    ));

    cmds.spawn((
        Text::new(&format!(
            "Items: {}/{}",
            inventory.count(),
            inventory.capacity
        ))
        .fg1(Palette::White)
        .layer(Layer::Ui),
        Position::new_f32(left_x, 2., 0.),
        CleanupStateInventory,
    ));

    let start_y = 3.5;
    for i in 0..inventory.capacity {
        let y_pos = start_y + (i as f32 * 0.5);

        // Spawn the slot marker
        cmds.spawn((
            InventorySlot {
                index: i,
                item_entity: inventory.items.get(i).copied(),
            },
            Position::new_f32(left_x + 2., y_pos, 0.),
            CleanupStateInventory,
        ));

        // Spawn the item text
        if let Some(item_id) = inventory.item_ids.get(i) {
            if let Some(item_entity) = id_registry.get_entity(*item_id) {
                let text = if let Ok(label) = q_labels.get(item_entity) {
                    label.get().to_string()
                } else {
                    "Unknown Item".to_string()
                };

                cmds.spawn((
                    Text::new(&text).fg1(Palette::White).layer(Layer::Ui),
                    Position::new_f32(left_x + 4., y_pos, 0.),
                    CleanupStateInventory,
                ));
            }
        } else {
            cmds.spawn((
                Text::new("(empty)").fg1(Palette::Gray).layer(Layer::Ui),
                Position::new_f32(left_x + 4., y_pos, 0.),
                CleanupStateInventory,
            ));
        }
    }

    cmds.spawn((
        InventoryCursor {
            index: 0,
            max_index: inventory.capacity.saturating_sub(1),
            is_player_side: true,
        },
        Text::new(">").fg1(Palette::Cyan).layer(Layer::Ui),
        Position::new_f32(left_x, start_y, 0.),
        CleanupStateInventory,
    ));

    // Position help text based on inventory size
    let help_y = start_y + (inventory.capacity as f32 * 0.5) + 1.0;
    cmds.spawn((
        Text::new("[{Y|ESC}] Back   [{Y|UP}/{Y|DOWN}] Navigate   [{Y|D}] Drop")
            .fg1(Palette::White)
            .layer(Layer::Ui),
        Position::new_f32(left_x, help_y.min(18.), 0.),
        CleanupStateInventory,
    ));
}

fn setup_container_screen(
    mut cmds: Commands,
    q_player: Query<Entity, With<Player>>,
    q_inventory: Query<&Inventory>,
    q_labels: Query<&Label>,
    id_registry: Res<StableIdRegistry>,
) {
    let Ok(player_entity) = q_player.single() else {
        return;
    };

    let Ok(inventory) = q_inventory.get(player_entity) else {
        return;
    };

    cmds.insert_resource(InventoryContext {
        player_entity,
        container_entity: None, // TODO: Set this to the actual container entity
    });

    let left_x = 2.0;
    let right_x = 15.0;
    let start_y = 3.5;

    // Left side - Player inventory
    cmds.spawn((
        Text::new("PLAYER INVENTORY")
            .fg1(Palette::Yellow)
            .layer(Layer::Ui),
        Position::new_f32(left_x, 1., 0.),
        CleanupStateContainer,
    ));

    cmds.spawn((
        Text::new(&format!(
            "Items: {}/{}",
            inventory.count(),
            inventory.capacity
        ))
        .fg1(Palette::White)
        .layer(Layer::Ui),
        Position::new_f32(left_x, 2., 0.),
        CleanupStateContainer,
    ));

    // Display player inventory items
    for i in 0..inventory.capacity {
        let y_pos = start_y + (i as f32 * 0.5);

        cmds.spawn((
            InventorySlot {
                index: i,
                item_entity: inventory.items.get(i).copied(),
            },
            Position::new_f32(left_x + 2., y_pos, 0.),
            CleanupStateContainer,
        ));

        if let Some(item_id) = inventory.item_ids.get(i) {
            if let Some(item_entity) = id_registry.get_entity(*item_id) {
                let text = if let Ok(label) = q_labels.get(item_entity) {
                    label.get().to_string()
                } else {
                    "Unknown Item".to_string()
                };

                cmds.spawn((
                    Text::new(&text).fg1(Palette::White).layer(Layer::Ui),
                    Position::new_f32(left_x + 4., y_pos, 0.),
                    CleanupStateContainer,
                ));
            }
        } else {
            cmds.spawn((
                Text::new("(empty)").fg1(Palette::Gray).layer(Layer::Ui),
                Position::new_f32(left_x + 4., y_pos, 0.),
                CleanupStateContainer,
            ));
        }
    }

    // Right side - Container inventory
    cmds.spawn((
        Text::new("CONTAINER").fg1(Palette::Yellow).layer(Layer::Ui),
        Position::new_f32(right_x, 1., 0.),
        CleanupStateContainer,
    ));

    cmds.spawn((
        Text::new("Empty").fg1(Palette::Gray).layer(Layer::Ui),
        Position::new_f32(right_x, 2., 0.),
        CleanupStateContainer,
    ));

    // TODO: Display container inventory items when we have a container entity

    // Cursor starts on player side
    cmds.spawn((
        InventoryCursor {
            index: 0,
            max_index: inventory.capacity.saturating_sub(1),
            is_player_side: true,
        },
        Text::new(">").fg1(Palette::Yellow).layer(Layer::Ui),
        Position::new_f32(left_x, start_y, 0.),
        CleanupStateContainer,
    ));

    // Help text
    let help_y = start_y + (inventory.capacity.max(10) as f32 * 0.5) + 1.0;
    cmds.spawn((
        Text::new("[{Y|ESC}] Back   [{Y|TAB}] Switch Side   [{Y|ENTER}] Transfer")
            .fg1(Palette::White)
            .layer(Layer::Ui),
        Position::new_f32(left_x, help_y.min(18.), 0.),
        CleanupStateContainer,
    ));
}

fn update_container_display(_cmds: Commands) {}

fn handle_inventory_input(
    mut cmds: Commands,
    keys: Res<KeyInput>,
    mut game_state: ResMut<CurrentGameState>,
    mut q_cursor: Query<(&mut InventoryCursor, &mut Position)>,
    q_inventory: Query<&Inventory>,
    player_pos: Res<PlayerPosition>,
    context: Res<InventoryContext>,
) {
    if keys.is_pressed(KeyCode::Escape) {
        game_state.next = GameState::Explore;
        return;
    }

    let Ok((mut cursor, mut cursor_pos)) = q_cursor.single_mut() else {
        return;
    };

    if keys.is_pressed(KeyCode::Up) && cursor.index > 0 {
        cursor.index -= 1;
        cursor_pos.y -= 0.5;
    }

    if keys.is_pressed(KeyCode::Down) && cursor.index < cursor.max_index {
        cursor.index += 1;
        cursor_pos.y += 0.5;
    }

    if keys.is_pressed(KeyCode::D) {
        let Ok(inventory) = q_inventory.get(context.player_entity) else {
            return;
        };

        if let Some(item_id) = inventory.item_ids.get(cursor.index).copied() {
            let world_pos = player_pos.world();
            cmds.queue(DropItemAction {
                entity: context.player_entity,
                item_stable_id: item_id,
                drop_position: world_pos,
            });
        }
    }
}

fn handle_container_input(keys: Res<KeyInput>, mut game_state: ResMut<CurrentGameState>) {
    if keys.is_pressed(KeyCode::Escape) {
        game_state.next = GameState::Explore;
    }
}
