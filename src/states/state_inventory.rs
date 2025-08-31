use bevy_ecs::prelude::*;
use macroquad::{input::KeyCode, prelude::trace};

use crate::{
    common::Palette,
    domain::{Inventory, Label, Player},
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
            .on_update(app, handle_inventory_input)
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

    cmds.spawn((
        Text::new("INVENTORY")
            .fg1(Palette::Yellow)
            .bg(Palette::Black)
            .layer(Layer::Ui),
        Position::new_f32(30., 2., 0.),
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
        Position::new_f32(30., 4., 0.),
        CleanupStateInventory,
    ));

    let start_y = 6.0;
    for i in 0..inventory.capacity {
        // Spawn the slot marker
        cmds.spawn((
            InventorySlot {
                index: i,
                item_entity: inventory.items.get(i).copied(),
            },
            Position::new_f32(32., start_y + i as f32, 0.),
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
                    Position::new_f32(34., start_y + i as f32, 0.),
                    CleanupStateInventory,
                ));
            }
        } else {
            cmds.spawn((
                Text::new("(empty)").fg1(Palette::Gray).layer(Layer::Ui),
                Position::new_f32(34., start_y + i as f32, 0.),
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
        Text::new("cursor").fg1(Palette::Cyan).layer(Layer::Ui),
        Position::new_f32(30., start_y, 0.),
        CleanupStateInventory,
    ));

    cmds.spawn((
        Text::new("[ESC] Back   [UP/DOWN] Navigate   [D] Drop")
            .fg1(Palette::Gray)
            .layer(Layer::Ui),
        Position::new_f32(20., 35., 0.),
        CleanupStateInventory,
    ));
}

fn setup_container_screen(mut cmds: Commands, q_player: Query<Entity, With<Player>>) {
    let Ok(_player_entity) = q_player.single() else {
        return;
    };

    cmds.spawn((
        Text::new("CONTAINER VIEW")
            .fg1(Palette::Yellow)
            .bg(Palette::Black)
            .layer(Layer::Ui),
        Position::new_f32(30., 2., 0.),
        CleanupStateContainer,
    ));

    cmds.spawn((
        Text::new("Player Inventory")
            .fg1(Palette::White)
            .layer(Layer::Ui),
        Position::new_f32(10., 4., 0.),
        CleanupStateContainer,
    ));

    cmds.spawn((
        Text::new("Container").fg1(Palette::White).layer(Layer::Ui),
        Position::new_f32(50., 4., 0.),
        CleanupStateContainer,
    ));

    cmds.spawn((
        Text::new("[ESC] Back   [TAB] Switch Side   [ENTER] Transfer")
            .fg1(Palette::Gray)
            .layer(Layer::Ui),
        Position::new_f32(15., 35., 0.),
        CleanupStateContainer,
    ));
}

fn update_container_display(_cmds: Commands) {}

fn handle_inventory_input(
    keys: Res<KeyInput>,
    mut game_state: ResMut<CurrentGameState>,
    mut q_cursor: Query<(&mut InventoryCursor, &mut Position)>,
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
        cursor_pos.y -= 1.0;
    }

    if keys.is_pressed(KeyCode::Down) && cursor.index < cursor.max_index {
        cursor.index += 1;
        cursor_pos.y += 1.0;
    }

    if keys.is_pressed(KeyCode::D) {}
}

fn handle_container_input(keys: Res<KeyInput>, mut game_state: ResMut<CurrentGameState>) {
    if keys.is_pressed(KeyCode::Escape) {
        game_state.next = GameState::Explore;
    }
}
