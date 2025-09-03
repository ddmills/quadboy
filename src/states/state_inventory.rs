use bevy_ecs::{prelude::*, system::SystemId};
use macroquad::input::{KeyCode, is_key_pressed};

use crate::{
    common::Palette,
    domain::{
        DropItemAction, EquipItemAction, EquipmentSlot, EquipmentSlots, Equippable, Equipped,
        Inventory, Label, Player, PlayerPosition, StackCount, TransferItemAction,
        UnequipItemAction, game_loop,
    },
    engine::{App, KeyInput, Plugin, StableIdRegistry},
    rendering::{Glyph, Layer, Position, Text},
    states::{CurrentGameState, GameState, GameStatePlugin, cleanup_system},
    ui::Button,
};

#[derive(Event)]
pub struct InventoryChangedEvent;

#[derive(Resource)]
struct InventoryCallbacks {
    back_to_explore: SystemId,
    drop_item: SystemId,
    equip_item: SystemId,
    unequip_item: SystemId,
}

#[derive(Component)]
pub struct CleanupStateInventory;

#[derive(Component)]
pub struct CleanupStateEquipSlotSelect;

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

#[derive(Component)]
pub struct EquipSlotCursor {
    pub index: usize,
    pub max_index: usize,
}

#[derive(Resource)]
pub struct InventoryContext {
    pub player_entity: Entity,
    pub container_entity: Option<Entity>,
    pub selected_item_id: Option<u64>,
    pub available_slots: Vec<EquipmentSlot>,
}

fn setup_inventory_callbacks(world: &mut World) {
    let callbacks = InventoryCallbacks {
        back_to_explore: world.register_system(back_to_explore),
        drop_item: world.register_system(drop_selected_item),
        equip_item: world.register_system(equip_selected_item),
        unequip_item: world.register_system(unequip_selected_item),
    };

    world.insert_resource(callbacks);
}

fn back_to_explore(mut game_state: ResMut<CurrentGameState>) {
    game_state.next = GameState::Explore;
}

fn drop_selected_item(
    mut cmds: Commands,
    q_cursor: Query<&InventoryCursor>,
    q_inventory: Query<&Inventory>,
    context: Res<InventoryContext>,
    player_pos: Res<PlayerPosition>,
    mut e_inventory_changed: EventWriter<InventoryChangedEvent>,
) {
    let Ok(cursor) = q_cursor.single() else {
        return;
    };

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

fn equip_selected_item(
    mut cmds: Commands,
    mut game_state: ResMut<CurrentGameState>,
    q_cursor: Query<&InventoryCursor>,
    q_inventory: Query<&Inventory>,
    q_equippable: Query<&Equippable>,
    mut context: ResMut<InventoryContext>,
    id_registry: Res<StableIdRegistry>,
    mut e_inventory_changed: EventWriter<InventoryChangedEvent>,
) {
    let Ok(cursor) = q_cursor.single() else {
        return;
    };

    let Ok(inventory) = q_inventory.get(context.player_entity) else {
        return;
    };

    if let Some(item_id) = inventory.item_ids.get(cursor.index).copied() {
        // Check if item is equippable
        if let Some(item_entity) = id_registry.get_entity(item_id) {
            if let Ok(equippable) = q_equippable.get(item_entity) {
                // If item only has one slot requirement, equip directly
                if equippable.slot_requirements.len() == 1 {
                    if let Some(player_id) = id_registry.get_id(context.player_entity) {
                        cmds.queue(EquipItemAction {
                            entity_id: player_id,
                            item_id,
                        });
                        e_inventory_changed.write(InventoryChangedEvent);
                    }
                } else {
                    // Set up context for equipment slot selection
                    context.selected_item_id = Some(item_id);
                    context.available_slots = equippable.slot_requirements.clone();

                    // Transition to equipment slot selection
                    game_state.next = GameState::EquipSlotSelect;
                }
            }
        }
    }
}

fn unequip_selected_item(
    mut cmds: Commands,
    q_cursor: Query<&InventoryCursor>,
    q_inventory: Query<&Inventory>,
    q_equipped: Query<&Equipped>,
    context: Res<InventoryContext>,
    id_registry: Res<StableIdRegistry>,
    mut e_inventory_changed: EventWriter<InventoryChangedEvent>,
) {
    let Ok(cursor) = q_cursor.single() else {
        return;
    };

    let Ok(inventory) = q_inventory.get(context.player_entity) else {
        return;
    };

    if let Some(item_id) = inventory.item_ids.get(cursor.index).copied() {
        // Check if item is equipped
        if let Some(item_entity) = id_registry.get_entity(item_id) {
            if q_equipped.get(item_entity).is_ok() {
                cmds.queue(UnequipItemAction::new(item_id));
                e_inventory_changed.write(InventoryChangedEvent);
            }
        }
    }
}

fn remove_inventory_callbacks(mut cmds: Commands) {
    cmds.remove_resource::<InventoryCallbacks>();
}

pub struct InventoryStatePlugin;

impl Plugin for InventoryStatePlugin {
    fn build(&self, app: &mut App) {
        app.register_event::<InventoryChangedEvent>();

        GameStatePlugin::new(GameState::Inventory)
            .on_enter(
                app,
                (setup_inventory_callbacks, setup_inventory_screen).chain(),
            )
            .on_update(
                app,
                (handle_inventory_input, refresh_inventory_display, game_loop),
            )
            .on_leave(
                app,
                (
                    cleanup_system::<CleanupStateInventory>,
                    remove_inventory_callbacks,
                )
                    .chain(),
            );

        GameStatePlugin::new(GameState::Container)
            .on_enter(app, setup_container_screen)
            .on_update(
                app,
                (handle_container_input, refresh_container_display, game_loop),
            )
            .on_leave(app, cleanup_system::<CleanupStateContainer>);

        GameStatePlugin::new(GameState::EquipSlotSelect)
            .on_enter(app, setup_equip_slot_screen)
            .on_update(app, handle_equip_slot_input)
            .on_leave(app, cleanup_system::<CleanupStateEquipSlotSelect>);
    }
}

fn setup_inventory_screen(
    mut cmds: Commands,
    callbacks: Res<InventoryCallbacks>,
    q_player: Query<Entity, With<Player>>,
    q_inventory: Query<&Inventory>,
    q_labels: Query<&Label>,
    q_glyphs: Query<&Glyph>,
    q_equipped: Query<&Equipped>,
    q_stack_counts: Query<&StackCount>,
    id_registry: Res<StableIdRegistry>,
) {
    let Ok(player_entity) = q_player.single() else {
        return;
    };

    cmds.insert_resource(InventoryContext {
        player_entity,
        container_entity: None,
        selected_item_id: None,
        available_slots: Vec::new(),
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
                    let mut item_glyph = Glyph::idx(glyph.idx).layer(Layer::Ui).bg(Palette::Black);
                    item_glyph.fg1 = glyph.fg1;
                    item_glyph.fg2 = glyph.fg2;
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

                // Check for stack count
                let display_text = if let Ok(stack_count) = q_stack_counts.get(item_entity) {
                    if stack_count.count > 1 {
                        format!("{} x{}", text, stack_count.count)
                    } else {
                        text
                    }
                } else {
                    text
                };

                // Check if item is equipped
                let is_equipped = q_equipped.get(item_entity).is_ok();
                let final_text = if is_equipped {
                    format!("{} {{G|[E]}}", display_text)
                } else {
                    display_text
                };

                cmds.spawn((
                    Text::new(&final_text).layer(Layer::Ui).bg(Palette::Black),
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

    // Position action buttons based on inventory size
    let help_y = start_y + (inventory.capacity as f32 * 1.0) + 1.0;

    // Back button
    cmds.spawn((
        Position::new_f32(left_x, help_y.min(18.), 0.),
        Button::new("({Y|I}) BACK", callbacks.back_to_explore).hotkey(KeyCode::I),
        CleanupStateInventory,
    ));

    // Drop button
    cmds.spawn((
        Position::new_f32(left_x + 6., help_y.min(18.), 0.),
        Button::new("({Y|D}) DROP", callbacks.drop_item).hotkey(KeyCode::D),
        CleanupStateInventory,
    ));

    // Equip button
    cmds.spawn((
        Position::new_f32(left_x + 12., help_y.min(18.), 0.),
        Button::new("({Y|E}) EQUIP", callbacks.equip_item).hotkey(KeyCode::E),
        CleanupStateInventory,
    ));

    // Unequip button
    cmds.spawn((
        Position::new_f32(left_x + 19., help_y.min(18.), 0.),
        Button::new("({Y|U}) UNEQUIP", callbacks.unequip_item).hotkey(KeyCode::U),
        CleanupStateInventory,
    ));

    // Navigation help text
    cmds.spawn((
        Text::new("[{Y|UP}/{Y|DOWN}] Navigate")
            .fg1(Palette::White)
            .layer(Layer::Ui),
        Position::new_f32(left_x, help_y.min(18.) + 1., 0.),
        CleanupStateInventory,
    ));
}

fn setup_container_screen(
    mut cmds: Commands,
    q_player: Query<Entity, With<Player>>,
    q_inventory: Query<&Inventory>,
    q_labels: Query<&Label>,
    q_glyphs: Query<&Glyph>,
    q_equipped: Query<&Equipped>,
    q_stack_counts: Query<&StackCount>,
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
                    let mut item_glyph = Glyph::idx(glyph.idx).layer(Layer::Ui).bg(Palette::Black);
                    item_glyph.fg1 = glyph.fg1;
                    item_glyph.fg2 = glyph.fg2;
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

                // Check for stack count
                let display_text = if let Ok(stack_count) = q_stack_counts.get(item_entity) {
                    if stack_count.count > 1 {
                        format!("{} x{}", text, stack_count.count)
                    } else {
                        text
                    }
                } else {
                    text
                };

                // Check if item is equipped
                let is_equipped = q_equipped.get(item_entity).is_ok();
                let final_text = if is_equipped {
                    format!("{} {{G|[E]}}", display_text)
                } else {
                    display_text
                };

                cmds.spawn((
                    Text::new(&final_text).layer(Layer::Ui).bg(Palette::Black),
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
                    let mut item_glyph = Glyph::idx(glyph.idx).layer(Layer::Ui).bg(Palette::Black);
                    item_glyph.fg1 = glyph.fg1;
                    item_glyph.fg2 = glyph.fg2;
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

                // Check for stack count
                let display_text = if let Ok(stack_count) = q_stack_counts.get(item_entity) {
                    if stack_count.count > 1 {
                        format!("{} x{}", text, stack_count.count)
                    } else {
                        text
                    }
                } else {
                    text
                };

                // Check if item is equipped
                let is_equipped = q_equipped.get(item_entity).is_ok();
                let final_text = if is_equipped {
                    format!("{} {{G|[E]}}", display_text)
                } else {
                    display_text
                };

                cmds.spawn((
                    Text::new(&final_text).layer(Layer::Ui).bg(Palette::Black),
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
    _q_equipment: Query<&EquipmentSlots>,
    q_labels: Query<&Label>,
    q_glyphs: Query<&Glyph>,
    q_equipped: Query<&Equipped>,
    q_stack_counts: Query<&StackCount>,
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
                    let mut item_glyph = Glyph::idx(glyph.idx).layer(Layer::Ui).bg(Palette::Black);
                    item_glyph.fg1 = glyph.fg1;
                    item_glyph.fg2 = glyph.fg2;
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

                // Check for stack count
                let display_text = if let Ok(stack_count) = q_stack_counts.get(item_entity) {
                    if stack_count.count > 1 {
                        format!("{} x{}", text, stack_count.count)
                    } else {
                        text
                    }
                } else {
                    text
                };

                // Check if item is equipped
                let is_equipped = q_equipped.get(item_entity).is_ok();
                let final_text = if is_equipped {
                    format!("{} {{G|[E]}}", display_text)
                } else {
                    display_text
                };

                cmds.spawn((
                    Text::new(&final_text).layer(Layer::Ui).bg(Palette::Black),
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

fn setup_equip_slot_screen(
    mut cmds: Commands,
    ctx: Res<InventoryContext>,
    registry: Res<StableIdRegistry>,
    q_player: Query<(&Inventory, &EquipmentSlots), With<Player>>,
    q_item: Query<(&Label, &Equippable)>,
) {
    // Get player components
    let Ok((inventory, equipment_slots)) = q_player.get(ctx.player_entity) else {
        return;
    };

    // Get selected item
    let Some(item_id) = ctx.selected_item_id else {
        return;
    };

    let Some(item_entity) = registry.get_entity(item_id) else {
        return;
    };

    let Ok((label, equippable)) = q_item.get(item_entity) else {
        return;
    };

    // Title
    cmds.spawn((
        Text::new(&format!("Equip {} to:", label.get()))
            .fg1(Palette::White)
            .layer(Layer::Ui),
        Position::new_f32(2., 2., 0.),
        CleanupStateEquipSlotSelect,
    ));

    // List available slots
    let mut y_offset = 4.;
    for (i, slot) in ctx.available_slots.iter().enumerate() {
        let is_selected = i == 0; // First slot selected by default
        let color = if is_selected {
            Palette::Yellow
        } else {
            Palette::White
        };

        let slot_name = format!("{:?}", slot);
        let prefix = if is_selected { ">" } else { " " };

        cmds.spawn((
            Text::new(&format!("{} {}", prefix, slot_name))
                .fg1(color)
                .layer(Layer::Ui),
            Position::new_f32(3., y_offset, 0.),
            CleanupStateEquipSlotSelect,
        ));

        y_offset += 1.;
    }

    // Help text
    cmds.spawn((
        Text::new("Up/Down: Select | Enter: Equip | ESC: Cancel")
            .fg1(Palette::Gray)
            .layer(Layer::Ui),
        Position::new_f32(2., y_offset + 2., 0.),
        CleanupStateEquipSlotSelect,
    ));
}

fn handle_equip_slot_input(
    mut cmds: Commands,
    mut game_state: ResMut<CurrentGameState>,
    mut ctx: ResMut<InventoryContext>,
    q_player: Query<Entity, With<Player>>,
    registry: Res<StableIdRegistry>,
) {
    if is_key_pressed(KeyCode::Escape) {
        game_state.next = GameState::Inventory;
        return;
    }

    let Ok(player_entity) = q_player.single() else {
        return;
    };

    let Some(player_id) = registry.get_id(player_entity) else {
        return;
    };

    let Some(item_id) = ctx.selected_item_id else {
        return;
    };

    // For now, just equip to first available slot on Enter
    if is_key_pressed(KeyCode::Enter) && !ctx.available_slots.is_empty() {
        // Equip new item (EquipItemAction should handle slot conflicts internally)
        cmds.queue(EquipItemAction {
            entity_id: player_id,
            item_id,
        });

        game_state.next = GameState::Inventory;
    }
}

fn refresh_container_display(
    mut cmds: Commands,
    mut e_inventory_changed: EventReader<InventoryChangedEvent>,
    q_inventory: Query<&Inventory>,
    q_labels: Query<&Label>,
    q_glyphs: Query<&Glyph>,
    q_equipped: Query<&Equipped>,
    q_stack_counts: Query<&StackCount>,
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
                    let mut item_glyph = Glyph::idx(glyph.idx).layer(Layer::Ui).bg(Palette::Black);
                    item_glyph.fg1 = glyph.fg1;
                    item_glyph.fg2 = glyph.fg2;
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

                // Check for stack count
                let display_text = if let Ok(stack_count) = q_stack_counts.get(item_entity) {
                    if stack_count.count > 1 {
                        format!("{} x{}", text, stack_count.count)
                    } else {
                        text
                    }
                } else {
                    text
                };

                // Check if item is equipped
                let is_equipped = q_equipped.get(item_entity).is_ok();
                let final_text = if is_equipped {
                    format!("{} {{G|[E]}}", display_text)
                } else {
                    display_text
                };

                cmds.spawn((
                    Text::new(&final_text).layer(Layer::Ui).bg(Palette::Black),
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
                    let mut item_glyph = Glyph::idx(glyph.idx).layer(Layer::Ui).bg(Palette::Black);
                    item_glyph.fg1 = glyph.fg1;
                    item_glyph.fg2 = glyph.fg2;
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

                // Check for stack count
                let display_text = if let Ok(stack_count) = q_stack_counts.get(item_entity) {
                    if stack_count.count > 1 {
                        format!("{} x{}", text, stack_count.count)
                    } else {
                        text
                    }
                } else {
                    text
                };

                // Check if item is equipped
                let is_equipped = q_equipped.get(item_entity).is_ok();
                let final_text = if is_equipped {
                    format!("{} {{G|[E]}}", display_text)
                } else {
                    display_text
                };

                cmds.spawn((
                    Text::new(&final_text).layer(Layer::Ui).bg(Palette::Black),
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
    q_equippable: Query<&Equippable>,
    q_equipped: Query<&Equipped>,
    player_pos: Res<PlayerPosition>,
    mut context: ResMut<InventoryContext>,
    id_registry: Res<StableIdRegistry>,
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

    if keys.is_pressed(KeyCode::E) {
        let Ok(inventory) = q_inventory.get(context.player_entity) else {
            return;
        };

        if let Some(item_id) = inventory.item_ids.get(cursor.index).copied() {
            // Check if item is equippable
            if let Some(item_entity) = id_registry.get_entity(item_id) {
                if let Ok(equippable) = q_equippable.get(item_entity) {
                    // If item only has one slot requirement, equip directly
                    if equippable.slot_requirements.len() == 1 {
                        if let Some(player_id) = id_registry.get_id(context.player_entity) {
                            cmds.queue(EquipItemAction {
                                entity_id: player_id,
                                item_id,
                            });
                            e_inventory_changed.write(InventoryChangedEvent);
                        }
                    } else {
                        // Set up context for equipment slot selection
                        context.selected_item_id = Some(item_id);
                        context.available_slots = equippable.slot_requirements.clone();

                        // Transition to equipment slot selection
                        game_state.next = GameState::EquipSlotSelect;
                    }
                }
            }
        }
    }

    if keys.is_pressed(KeyCode::U) {
        let Ok(inventory) = q_inventory.get(context.player_entity) else {
            return;
        };

        if let Some(item_id) = inventory.item_ids.get(cursor.index).copied() {
            // Check if item is equipped
            if let Some(item_entity) = id_registry.get_entity(item_id) {
                if q_equipped.get(item_entity).is_ok() {
                    cmds.queue(UnequipItemAction::new(item_id));
                    e_inventory_changed.write(InventoryChangedEvent);
                }
            }
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
