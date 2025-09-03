use bevy_ecs::{prelude::*, system::SystemId};
use macroquad::input::{KeyCode, is_key_pressed};

use crate::{
    common::Palette,
    domain::{
        DropItemAction, EquipItemAction, EquipmentSlot, EquipmentSlots, Equippable, Equipped,
        Inventory, Label, Player, PlayerPosition, StackCount, UnequipItemAction, game_loop,
    },
    engine::{App, KeyInput, Plugin, StableIdRegistry},
    rendering::{Glyph, Layer, Position, Text},
    states::{CurrentGameState, GameState, GameStatePlugin, cleanup_system},
    ui::{Button, List, ListContext, ListFocus, ListItemData, ListState},
};

#[derive(Event)]
pub struct InventoryChangedEvent;

#[derive(Resource)]
struct InventoryCallbacks {
    back_to_explore: SystemId,
    select_item: SystemId,
    drop_item: SystemId,
    equip_item: SystemId,
    unequip_item: SystemId,
    toggle_equip_item: SystemId,
}

#[derive(Component)]
pub struct CleanupStateInventory;

#[derive(Component)]
pub struct CleanupStateEquipSlotSelect;

#[derive(Component)]
pub struct InventoryWeightText;

#[derive(Component)]
pub struct InventoryItemDisplay;

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
    pub selected_item_id: Option<u64>,
    pub available_slots: Vec<EquipmentSlot>,
}

fn back_to_explore(mut game_state: ResMut<CurrentGameState>) {
    game_state.next = GameState::Explore;
}

fn select_item() {
    // No-op for empty slots
}

fn build_inventory_list_items(
    inventory: &Inventory,
    q_labels: &Query<&Label>,
    q_glyphs: &Query<&Glyph>,
    q_equipped: &Query<&Equipped>,
    q_stack_counts: &Query<&StackCount>,
    id_registry: &StableIdRegistry,
    callbacks: &InventoryCallbacks,
) -> Vec<ListItemData> {
    let mut items = Vec::new();

    for (i, &item_id) in inventory.item_ids.iter().enumerate() {
        if let Some(item_entity) = id_registry.get_entity(item_id) {
            let text = if let Ok(label) = q_labels.get(item_entity) {
                label.get().to_string()
            } else {
                "Unknown Item".to_string()
            };

            let display_text = if let Ok(stack_count) = q_stack_counts.get(item_entity) {
                if stack_count.count > 1 {
                    format!("{} x{}", text, stack_count.count)
                } else {
                    text
                }
            } else {
                text
            };

            let final_text = if let Ok(equipped) = q_equipped.get(item_entity) {
                // Get the first slot name (most items only use one slot)
                let slot_name = equipped
                    .slots
                    .first()
                    .map(|slot| slot.display_name())
                    .unwrap_or("Unknown");
                format!("{} {{G|[{}]}}", display_text, slot_name)
            } else {
                display_text
            };

            items.push(ListItemData {
                label: final_text,
                callback: callbacks.drop_item,
                hotkey: None,
                context_data: Some(item_id),
            });
        }
    }

    items
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

fn setup_inventory_callbacks(world: &mut World) {
    let callbacks = InventoryCallbacks {
        back_to_explore: world.register_system(back_to_explore),
        select_item: world.register_system(select_item),
        drop_item: world.register_system(drop_selected_item),
        equip_item: world.register_system(equip_selected_item),
        unequip_item: world.register_system(unequip_selected_item),
        toggle_equip_item: world.register_system(toggle_equip_selected_item),
    };

    world.insert_resource(callbacks);
}

fn drop_selected_item(
    mut cmds: Commands,
    list_context: Res<ListContext>,
    context: Res<InventoryContext>,
    player_pos: Res<PlayerPosition>,
    mut e_inventory_changed: EventWriter<InventoryChangedEvent>,
) {
    if let Some(item_id) = list_context.context_data {
        let world_pos = player_pos.world();
        cmds.queue(DropItemAction {
            entity: context.player_entity,
            item_stable_id: item_id,
            drop_position: world_pos,
        });
    }
}

fn equip_selected_item(
    mut cmds: Commands,
    mut game_state: ResMut<CurrentGameState>,
    list_context: Res<ListContext>,
    q_equippable: Query<&Equippable>,
    mut context: ResMut<InventoryContext>,
    id_registry: Res<StableIdRegistry>,
    mut e_inventory_changed: EventWriter<InventoryChangedEvent>,
) {
    if let Some(item_id) = list_context.context_data {
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
    list_context: Res<ListContext>,
    q_equipped: Query<&Equipped>,
    id_registry: Res<StableIdRegistry>,
    mut e_inventory_changed: EventWriter<InventoryChangedEvent>,
) {
    if let Some(item_id) = list_context.context_data {
        // Check if item is equipped
        if let Some(item_entity) = id_registry.get_entity(item_id) {
            if q_equipped.get(item_entity).is_ok() {
                cmds.queue(UnequipItemAction::new(item_id));
                e_inventory_changed.write(InventoryChangedEvent);
            }
        }
    }
}

fn toggle_equip_selected_item(
    mut cmds: Commands,
    mut game_state: ResMut<CurrentGameState>,
    list_context: Res<ListContext>,
    q_equippable: Query<&Equippable>,
    q_equipped: Query<&Equipped>,
    mut context: ResMut<InventoryContext>,
    id_registry: Res<StableIdRegistry>,
    mut e_inventory_changed: EventWriter<InventoryChangedEvent>,
) {
    if let Some(item_id) = list_context.context_data {
        if let Some(item_entity) = id_registry.get_entity(item_id) {
            // Check if item is already equipped
            if q_equipped.get(item_entity).is_ok() {
                // Item is equipped, unequip it
                cmds.queue(UnequipItemAction::new(item_id));
                e_inventory_changed.write(InventoryChangedEvent);
            } else {
                // Item is not equipped, try to equip it
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
    mut list_focus: ResMut<ListFocus>,
) {
    let Ok(player_entity) = q_player.single() else {
        return;
    };

    cmds.insert_resource(InventoryContext {
        player_entity,
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
            "Weight: {:.1}/{:.1} kg",
            inventory.get_total_weight(),
            inventory.capacity
        ))
        .fg1(Palette::White)
        .layer(Layer::Ui),
        Position::new_f32(left_x, 2., 0.),
        CleanupStateInventory,
        InventoryWeightText,
    ));

    // Build list items from inventory - just show actual items (no empty slots for weight-based)
    let mut list_items = Vec::new();

    for (i, &item_id) in inventory.item_ids.iter().enumerate() {
        if let Some(item_entity) = id_registry.get_entity(item_id) {
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
            let final_text = if let Ok(equipped) = q_equipped.get(item_entity) {
                let slot_name = equipped
                    .slots
                    .first()
                    .map(|slot| slot.display_name())
                    .unwrap_or("Unknown");
                format!("{} {{G|[{}]}}", display_text, slot_name)
            } else {
                display_text
            };

            list_items.push(ListItemData {
                label: final_text,
                callback: callbacks.select_item,
                hotkey: None,                    // Could add number keys 1-9 for quick selection
                context_data: Some(item_id),
            });
        }
    }

    // Spawn the inventory list
    let list_entity = cmds
        .spawn((
            List::new(list_items),
            ListState::new().with_focus(true),
            Position::new_f32(left_x + 1., 3., 0.),
            CleanupStateInventory,
        ))
        .id();

    // Set list focus
    list_focus.active_list = Some(list_entity);

    // Position action buttons based on number of items in inventory
    let help_y = 3.5 + (inventory.count() as f32 * 0.5) + 1.0;

    cmds.spawn((
        Position::new_f32(left_x, help_y.min(18.), 0.),
        Button::new("({Y|I}) BACK", callbacks.back_to_explore).hotkey(KeyCode::I),
        CleanupStateInventory,
    ));

    cmds.spawn((
        Position::new_f32(left_x + 6., help_y.min(18.), 0.),
        Button::new("({Y|U}) DROP", callbacks.drop_item).hotkey(KeyCode::U),
        CleanupStateInventory,
    ));

    cmds.spawn((
        Position::new_f32(left_x + 12., help_y.min(18.), 0.),
        Button::new("({Y|E}) TOGGLE EQUIP", callbacks.toggle_equip_item).hotkey(KeyCode::E),
        CleanupStateInventory,
    ));
}

fn handle_equip_slot_input(mut game_state: ResMut<CurrentGameState>) {
    if is_key_pressed(KeyCode::Escape) {
        game_state.next = GameState::Inventory;
    }
}

fn refresh_inventory_display(
    mut list_focus: ResMut<ListFocus>,
    mut q_list: Query<(&mut List, Entity), With<CleanupStateInventory>>,
    mut q_weight_text: Query<&mut Text, With<InventoryWeightText>>,
    context: Res<InventoryContext>,
    q_inventory: Query<&Inventory>,
    q_labels: Query<&Label>,
    q_glyphs: Query<&Glyph>,
    q_equipped: Query<&Equipped>,
    q_stack_counts: Query<&StackCount>,
    id_registry: Res<StableIdRegistry>,
    callbacks: Res<InventoryCallbacks>,
) {
    let Ok(player_inventory) = q_inventory.get(context.player_entity) else {
        return;
    };

    let Ok((mut list, list_entity)) = q_list.single_mut() else {
        return;
    };

    // Set focus to this list
    list_focus.active_list = Some(list_entity);

    let list_items = build_inventory_list_items(
        player_inventory,
        &q_labels,
        &q_glyphs,
        &q_equipped,
        &q_stack_counts,
        &id_registry,
        &callbacks,
    );

    // Only update if items have changed to prevent flickering
    if list.items.len() != list_items.len()
        || list
            .items
            .iter()
            .zip(list_items.iter())
            .any(|(a, b)| a.label != b.label)
    {
        list.items = list_items;
    }

    // Update weight text display
    if let Ok(mut text) = q_weight_text.single_mut() {
        text.value = format!(
            "Weight: {:.1}/{:.1} kg",
            player_inventory.get_total_weight(),
            player_inventory.capacity
        );
    }
}

fn handle_inventory_input(keys: Res<KeyInput>, mut game_state: ResMut<CurrentGameState>) {
    if keys.is_pressed(KeyCode::I) {
        game_state.next = GameState::Explore;
    }
}
