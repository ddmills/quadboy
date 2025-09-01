use bevy_ecs::prelude::*;
use macroquad::{input::KeyCode, prelude::trace};

use crate::{
    common::Palette,
    domain::{
        DropItemAction, Inventory, Label, Player, PlayerPosition, TransferItemAction, game_loop,
    },
    engine::{App, KeyInput, Plugin, StableIdRegistry},
    rendering::{Glyph, Layer, Position, Text},
    states::{CurrentGameState, GameState, GameStatePlugin, cleanup_system},
};

#[derive(Event)]
pub struct InventoryChangedEvent;

#[derive(Component)]
pub struct CleanupStateInventory;

#[derive(Component)]
pub struct CleanupStateContainer;

#[derive(Component)]
pub struct InventoryItemDisplay;

#[derive(Component)]
pub struct ContainerItemDisplay;

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
        app.register_event::<InventoryChangedEvent>();

        GameStatePlugin::new(GameState::Inventory)
            .on_enter(app, setup_inventory_screen)
            .on_update(
                app,
                (handle_inventory_input, refresh_inventory_display, game_loop),
            )
            .on_leave(app, cleanup_system::<CleanupStateInventory>);

        GameStatePlugin::new(GameState::Container)
            .on_enter(app, setup_container_screen)
            .on_update(
                app,
                (handle_container_input, refresh_container_display, game_loop),
            )
            .on_leave(app, cleanup_system::<CleanupStateContainer>);
    }
}

fn setup_inventory_screen(
    mut cmds: Commands,
    q_player: Query<Entity, With<Player>>,
    q_inventory: Query<&Inventory>,
    q_labels: Query<&Label>,
    q_glyphs: Query<&Glyph>,
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
        let y_pos = start_y + (i as f32 * 1.0);

        if let Some(item_id) = inventory.item_ids.get(i) {
            if let Some(item_entity) = id_registry.get_entity(*item_id) {
                if let Ok(glyph) = q_glyphs.get(item_entity) {
                    let mut item_glyph = Glyph::idx(glyph.idx).layer(Layer::Ui);
                    item_glyph.fg1 = glyph.fg1;
                    item_glyph.fg2 = glyph.fg2;
                    item_glyph.bg = glyph.bg;
                    item_glyph.outline = glyph.outline;

                    cmds.spawn((
                        item_glyph,
                        Position::new_f32(left_x + 1.5, y_pos - 0.5, 0.),
                        CleanupStateInventory,
                        InventoryItemDisplay,
                    ));
                }

                let text = if let Ok(label) = q_labels.get(item_entity) {
                    label.get().to_string()
                } else {
                    "Unknown Item".to_string()
                };

                cmds.spawn((
                    Text::new(&text).fg1(Palette::White).layer(Layer::Ui),
                    Position::new_f32(left_x + 3., y_pos, 0.),
                    CleanupStateInventory,
                    InventoryItemDisplay,
                ));
            }
        } else {
            cmds.spawn((
                Text::new("(empty)").fg1(Palette::Gray).layer(Layer::Ui),
                Position::new_f32(left_x + 3., y_pos, 0.),
                CleanupStateInventory,
                InventoryItemDisplay,
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
    let help_y = start_y + (inventory.capacity as f32 * 1.0) + 1.0;
    cmds.spawn((
        Text::new("[{Y|I}] Back   [{Y|UP}/{Y|DOWN}] Navigate   [{Y|D}] Drop")
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
    q_glyphs: Query<&Glyph>,
    id_registry: Res<StableIdRegistry>,
    context: Option<Res<InventoryContext>>,
) {
    let Ok(player_entity) = q_player.single() else {
        return;
    };

    let Ok(player_inventory) = q_inventory.get(player_entity) else {
        return;
    };

    // Get container entity from existing context if available
    let container_entity = context.and_then(|ctx| ctx.container_entity);

    // Re-insert the context resource to ensure it persists
    if container_entity.is_none() {
        // If no container entity, return to explore state
        return;
    }

    let container = container_entity.unwrap();
    let Ok(container_inventory) = q_inventory.get(container) else {
        return;
    };

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
            player_inventory.count(),
            player_inventory.capacity
        ))
        .fg1(Palette::White)
        .layer(Layer::Ui),
        Position::new_f32(left_x, 2., 0.),
        CleanupStateContainer,
    ));

    for i in 0..player_inventory.capacity {
        let y_pos = start_y + (i as f32 * 1.0);

        if let Some(item_id) = player_inventory.item_ids.get(i) {
            if let Some(item_entity) = id_registry.get_entity(*item_id) {
                // Spawn glyph if the item has one
                if let Ok(glyph) = q_glyphs.get(item_entity) {
                    let mut item_glyph = Glyph::idx(glyph.idx).layer(Layer::Ui);
                    item_glyph.fg1 = glyph.fg1;
                    item_glyph.fg2 = glyph.fg2;
                    item_glyph.bg = glyph.bg;
                    item_glyph.outline = glyph.outline;

                    cmds.spawn((
                        item_glyph,
                        Position::new_f32(left_x + 1.5, y_pos - 0.5, 0.),
                        CleanupStateContainer,
                        InventoryItemDisplay,
                    ));
                }

                let text = if let Ok(label) = q_labels.get(item_entity) {
                    label.get().to_string()
                } else {
                    "Unknown Item".to_string()
                };

                cmds.spawn((
                    Text::new(&text).fg1(Palette::White).layer(Layer::Ui),
                    Position::new_f32(left_x + 3., y_pos, 0.),
                    CleanupStateContainer,
                    InventoryItemDisplay,
                ));
            }
        } else {
            cmds.spawn((
                Text::new("(empty)").fg1(Palette::Gray).layer(Layer::Ui),
                Position::new_f32(left_x + 3., y_pos, 0.),
                CleanupStateContainer,
                InventoryItemDisplay,
            ));
        }
    }

    // Right side - Container inventory
    let container_label = if let Ok(label) = q_labels.get(container) {
        label.get().to_string()
    } else {
        "CONTAINER".to_string()
    };

    cmds.spawn((
        Text::new(&container_label)
            .fg1(Palette::Yellow)
            .layer(Layer::Ui),
        Position::new_f32(right_x, 1., 0.),
        CleanupStateContainer,
    ));

    cmds.spawn((
        Text::new(&format!(
            "Items: {}/{}",
            container_inventory.count(),
            container_inventory.capacity
        ))
        .fg1(Palette::White)
        .layer(Layer::Ui),
        Position::new_f32(right_x, 2., 0.),
        CleanupStateContainer,
    ));

    // Display container inventory items
    for i in 0..container_inventory.capacity {
        let y_pos = start_y + (i as f32 * 1.0);

        if let Some(item_id) = container_inventory.item_ids.get(i) {
            if let Some(item_entity) = id_registry.get_entity(*item_id) {
                // Spawn glyph if the item has one
                if let Ok(glyph) = q_glyphs.get(item_entity) {
                    let mut item_glyph = Glyph::idx(glyph.idx).layer(Layer::Ui);
                    item_glyph.fg1 = glyph.fg1;
                    item_glyph.fg2 = glyph.fg2;
                    item_glyph.bg = glyph.bg;
                    item_glyph.outline = glyph.outline;

                    cmds.spawn((
                        item_glyph,
                        Position::new_f32(right_x + 1.5, y_pos - 0.5, 0.),
                        CleanupStateContainer,
                        ContainerItemDisplay,
                    ));
                }

                let text = if let Ok(label) = q_labels.get(item_entity) {
                    label.get().to_string()
                } else {
                    "Unknown Item".to_string()
                };

                cmds.spawn((
                    Text::new(&text).fg1(Palette::White).layer(Layer::Ui),
                    Position::new_f32(right_x + 3., y_pos, 0.),
                    CleanupStateContainer,
                    ContainerItemDisplay,
                ));
            }
        } else {
            cmds.spawn((
                Text::new("(empty)").fg1(Palette::Gray).layer(Layer::Ui),
                Position::new_f32(right_x + 3., y_pos, 0.),
                CleanupStateContainer,
                ContainerItemDisplay,
            ));
        }
    }

    // Cursor starts on player side
    cmds.spawn((
        InventoryCursor {
            index: 0,
            max_index: player_inventory.capacity.saturating_sub(1),
            is_player_side: true,
        },
        Text::new(">").fg1(Palette::Cyan).layer(Layer::Ui),
        Position::new_f32(left_x, start_y, 0.),
        CleanupStateContainer,
    ));

    // Help text
    let max_capacity = player_inventory.capacity.max(container_inventory.capacity);
    let help_y = start_y + (max_capacity as f32 * 1.0) + 1.0;
    cmds.spawn((
        Text::new("[{Y|I}] Back   [{Y|TAB}] Switch Side   [{Y|ENTER}] Transfer")
            .fg1(Palette::White)
            .layer(Layer::Ui),
        Position::new_f32(left_x, help_y.min(18.), 0.),
        CleanupStateContainer,
    ));
}

fn refresh_inventory_display(
    mut cmds: Commands,
    mut e_inventory_changed: EventReader<InventoryChangedEvent>,
    q_inventory: Query<&Inventory>,
    q_labels: Query<&Label>,
    q_glyphs: Query<&Glyph>,
    q_item_displays: Query<Entity, With<InventoryItemDisplay>>,
    q_player: Query<Entity, With<Player>>,
    id_registry: Res<StableIdRegistry>,
) {
    if e_inventory_changed.is_empty() {
        return;
    }
    e_inventory_changed.clear();

    // Remove old item displays
    for entity in q_item_displays.iter() {
        cmds.entity(entity).despawn();
    }

    // Rebuild player inventory display
    let Ok(player_entity) = q_player.single() else {
        return;
    };

    let Ok(inventory) = q_inventory.get(player_entity) else {
        return;
    };

    let left_x = 2.0;
    let start_y = 3.5;

    for i in 0..inventory.capacity {
        let y_pos = start_y + (i as f32 * 1.0);

        if let Some(item_id) = inventory.item_ids.get(i) {
            if let Some(item_entity) = id_registry.get_entity(*item_id) {
                // Spawn glyph if the item has one
                if let Ok(glyph) = q_glyphs.get(item_entity) {
                    let mut item_glyph = Glyph::idx(glyph.idx).layer(Layer::Ui);
                    item_glyph.fg1 = glyph.fg1;
                    item_glyph.fg2 = glyph.fg2;
                    item_glyph.bg = glyph.bg;
                    item_glyph.outline = glyph.outline;

                    cmds.spawn((
                        item_glyph,
                        Position::new_f32(left_x + 1.5, y_pos - 0.5, 0.),
                        CleanupStateInventory,
                        InventoryItemDisplay,
                    ));
                }

                let text = if let Ok(label) = q_labels.get(item_entity) {
                    label.get().to_string()
                } else {
                    "Unknown Item".to_string()
                };

                cmds.spawn((
                    Text::new(&text).fg1(Palette::White).layer(Layer::Ui),
                    Position::new_f32(left_x + 3., y_pos, 0.),
                    CleanupStateInventory,
                    InventoryItemDisplay,
                ));
            }
        } else {
            cmds.spawn((
                Text::new("(empty)").fg1(Palette::Gray).layer(Layer::Ui),
                Position::new_f32(left_x + 3., y_pos, 0.),
                CleanupStateInventory,
                InventoryItemDisplay,
            ));
        }
    }
}

fn refresh_container_display(
    mut cmds: Commands,
    mut e_inventory_changed: EventReader<InventoryChangedEvent>,
    q_inventory: Query<&Inventory>,
    q_labels: Query<&Label>,
    q_glyphs: Query<&Glyph>,
    q_player_displays: Query<Entity, With<InventoryItemDisplay>>,
    q_container_displays: Query<Entity, With<ContainerItemDisplay>>,
    context: Res<InventoryContext>,
    id_registry: Res<StableIdRegistry>,
) {
    if e_inventory_changed.is_empty() {
        return;
    }
    e_inventory_changed.clear();

    let Some(container_entity) = context.container_entity else {
        return;
    };

    // Remove old displays
    for entity in q_player_displays.iter() {
        cmds.entity(entity).despawn();
    }
    for entity in q_container_displays.iter() {
        cmds.entity(entity).despawn();
    }

    let left_x = 2.0;
    let right_x = 15.0;
    let start_y = 3.5;

    // Rebuild player inventory display
    let Ok(player_inventory) = q_inventory.get(context.player_entity) else {
        return;
    };

    for i in 0..player_inventory.capacity {
        let y_pos = start_y + (i as f32 * 1.0);

        if let Some(item_id) = player_inventory.item_ids.get(i) {
            if let Some(item_entity) = id_registry.get_entity(*item_id) {
                // Spawn glyph if the item has one
                if let Ok(glyph) = q_glyphs.get(item_entity) {
                    let mut item_glyph = Glyph::idx(glyph.idx).layer(Layer::Ui);
                    item_glyph.fg1 = glyph.fg1;
                    item_glyph.fg2 = glyph.fg2;
                    item_glyph.bg = glyph.bg;
                    item_glyph.outline = glyph.outline;

                    cmds.spawn((
                        item_glyph,
                        Position::new_f32(left_x + 1.5, y_pos - 0.5, 0.),
                        CleanupStateContainer,
                        InventoryItemDisplay,
                    ));
                }

                let text = if let Ok(label) = q_labels.get(item_entity) {
                    label.get().to_string()
                } else {
                    "Unknown Item".to_string()
                };

                cmds.spawn((
                    Text::new(&text).fg1(Palette::White).layer(Layer::Ui),
                    Position::new_f32(left_x + 3., y_pos, 0.),
                    CleanupStateContainer,
                    InventoryItemDisplay,
                ));
            }
        } else {
            cmds.spawn((
                Text::new("(empty)").fg1(Palette::Gray).layer(Layer::Ui),
                Position::new_f32(left_x + 3., y_pos, 0.),
                CleanupStateContainer,
                InventoryItemDisplay,
            ));
        }
    }

    // Rebuild container inventory display
    let Ok(container_inventory) = q_inventory.get(container_entity) else {
        return;
    };

    for i in 0..container_inventory.capacity {
        let y_pos = start_y + (i as f32 * 1.0);

        if let Some(item_id) = container_inventory.item_ids.get(i) {
            if let Some(item_entity) = id_registry.get_entity(*item_id) {
                // Spawn glyph if the item has one
                if let Ok(glyph) = q_glyphs.get(item_entity) {
                    let mut item_glyph = Glyph::idx(glyph.idx).layer(Layer::Ui);
                    item_glyph.fg1 = glyph.fg1;
                    item_glyph.fg2 = glyph.fg2;
                    item_glyph.bg = glyph.bg;
                    item_glyph.outline = glyph.outline;

                    cmds.spawn((
                        item_glyph,
                        Position::new_f32(right_x + 1.5, y_pos - 0.5, 0.),
                        CleanupStateContainer,
                        ContainerItemDisplay,
                    ));
                }

                let text = if let Ok(label) = q_labels.get(item_entity) {
                    label.get().to_string()
                } else {
                    "Unknown Item".to_string()
                };

                cmds.spawn((
                    Text::new(&text).fg1(Palette::White).layer(Layer::Ui),
                    Position::new_f32(right_x + 3., y_pos, 0.),
                    CleanupStateContainer,
                    ContainerItemDisplay,
                ));
            }
        } else {
            cmds.spawn((
                Text::new("(empty)").fg1(Palette::Gray).layer(Layer::Ui),
                Position::new_f32(right_x + 3., y_pos, 0.),
                CleanupStateContainer,
                ContainerItemDisplay,
            ));
        }
    }
}

fn handle_inventory_input(
    mut cmds: Commands,
    keys: Res<KeyInput>,
    mut game_state: ResMut<CurrentGameState>,
    mut q_cursor: Query<(&mut InventoryCursor, &mut Position)>,
    q_inventory: Query<&Inventory>,
    player_pos: Res<PlayerPosition>,
    context: Res<InventoryContext>,
    mut e_inventory_changed: EventWriter<InventoryChangedEvent>,
) {
    if keys.is_pressed(KeyCode::I) {
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
            e_inventory_changed.write(InventoryChangedEvent);
        }
    }
}

fn handle_container_input(
    mut cmds: Commands,
    keys: Res<KeyInput>,
    mut game_state: ResMut<CurrentGameState>,
    mut q_cursor: Query<(&mut InventoryCursor, &mut Position, &mut Text)>,
    q_inventory: Query<&Inventory>,
    context: Res<InventoryContext>,
    mut e_inventory_changed: EventWriter<InventoryChangedEvent>,
) {
    if keys.is_pressed(KeyCode::I) {
        game_state.next = GameState::Explore;
        return;
    }

    let Ok((mut cursor, mut cursor_pos, mut cursor_text)) = q_cursor.single_mut() else {
        return;
    };

    let container_entity = match context.container_entity {
        Some(e) => e,
        None => {
            game_state.next = GameState::Explore;
            return;
        }
    };

    let Ok(player_inventory) = q_inventory.get(context.player_entity) else {
        return;
    };
    let Ok(container_inventory) = q_inventory.get(container_entity) else {
        return;
    };

    // Handle TAB to switch between player and container sides
    if keys.is_pressed(KeyCode::Tab) {
        cursor.is_player_side = !cursor.is_player_side;
        cursor.index = 0;

        if cursor.is_player_side {
            cursor.max_index = player_inventory.capacity.saturating_sub(1);
            cursor_pos.x = 2.0; // left_x
            cursor_text.value = ">".to_string();
            cursor_text.fg1 = Some(Palette::Cyan.into());
        } else {
            cursor.max_index = container_inventory.capacity.saturating_sub(1);
            cursor_pos.x = 15.0; // right_x
            cursor_text.value = "<".to_string();
            cursor_text.fg1 = Some(Palette::Orange.into());
        }
        cursor_pos.y = 3.5; // start_y
    }

    // Handle UP/DOWN navigation
    if keys.is_pressed(KeyCode::Up) && cursor.index > 0 {
        cursor.index -= 1;
        cursor_pos.y -= 1.0;
    }

    if keys.is_pressed(KeyCode::Down) && cursor.index < cursor.max_index {
        cursor.index += 1;
        cursor_pos.y += 1.0;
    }

    // Handle ENTER to transfer items
    if keys.is_pressed(KeyCode::Enter) {
        if cursor.is_player_side {
            // Transfer from player to container
            if let Some(item_id) = player_inventory.item_ids.get(cursor.index).copied() {
                cmds.queue(TransferItemAction {
                    from_entity: context.player_entity,
                    to_entity: container_entity,
                    item_stable_id: item_id,
                });
                e_inventory_changed.write(InventoryChangedEvent);
            }
        } else {
            // Transfer from container to player
            if let Some(item_id) = container_inventory.item_ids.get(cursor.index).copied() {
                cmds.queue(TransferItemAction {
                    from_entity: container_entity,
                    to_entity: context.player_entity,
                    item_stable_id: item_id,
                });
                e_inventory_changed.write(InventoryChangedEvent);
            }
        }
    }
}
