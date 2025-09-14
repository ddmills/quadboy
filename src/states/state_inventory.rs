use bevy_ecs::{prelude::*, system::SystemId};
use macroquad::input::{KeyCode, is_key_pressed};

use crate::{
    common::Palette,
    domain::{
        DropItemAction, EquipItemAction, EquipmentSlot, Equippable, Equipped, Inventory, Item,
        Label, Player, PlayerPosition, StackCount, UnequipItemAction, Weapon, game_loop,
        inventory::InventoryChangedEvent,
    },
    engine::{App, AudioKey, Plugin, StableIdRegistry},
    rendering::{Glyph, Layer, Position, Text},
    states::{CurrentGameState, GameState, GameStatePlugin, cleanup_system},
    ui::{
        ActivatableBuilder, Dialog, DialogState, ItemDialogBuilder, ItemDialogButton, List,
        ListContext, ListItemData, render_dialog_content, setup_buttons, setup_dialogs,
        spawn_item_dialog,
    },
};

#[derive(Resource)]
struct InventoryCallbacks {
    back_to_explore: SystemId,
    drop_item: SystemId,
    toggle_equip_item: SystemId,
    examine_item: SystemId,
    close_dialog: SystemId,
}

#[derive(Component, Clone)]
pub struct CleanupStateInventory;

#[derive(Component)]
pub struct CleanupStateEquipSlotSelect;

#[derive(Component)]
pub struct InventoryWeightText;

#[derive(Resource)]
pub struct InventoryContext {
    pub player_entity: Entity,
    pub selected_item_id: Option<u64>,
    pub available_slots: Vec<EquipmentSlot>,
}

fn back_to_explore(mut game_state: ResMut<CurrentGameState>) {
    game_state.next = GameState::Explore;
}

fn build_inventory_list_items(
    inventory: &Inventory,
    q_labels: &Query<&Label>,
    q_equipped: &Query<&Equipped>,
    q_stack_counts: &Query<&StackCount>,
    id_registry: &StableIdRegistry,
    callbacks: &InventoryCallbacks,
) -> Vec<ListItemData> {
    let mut items = Vec::new();

    for &item_id in inventory.item_ids.iter() {
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
                let slot_name = equipped
                    .slots
                    .first()
                    .map(|slot| slot.display_name())
                    .unwrap_or("Unknown");
                format!("{} {{G|[{}]}}", display_text, slot_name)
            } else {
                display_text
            };

            items
                .push(ListItemData::new(&final_text, callbacks.examine_item).with_context(item_id));
        }
    }

    items
}

fn setup_equip_slot_screen(
    mut cmds: Commands,
    ctx: Res<InventoryContext>,
    registry: Res<StableIdRegistry>,
    q_item: Query<&Label>,
) {
    let Some(item_id) = ctx.selected_item_id else {
        return;
    };

    let Some(item_entity) = registry.get_entity(item_id) else {
        return;
    };

    let Ok(label) = q_item.get(item_entity) else {
        return;
    };

    cmds.spawn((
        Text::new(&format!("Equip {} to:", label.get()))
            .fg1(Palette::White)
            .layer(Layer::Ui),
        Position::new_f32(2., 2., 0.),
        CleanupStateEquipSlotSelect,
    ));

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
        drop_item: world.register_system(drop_selected_item),
        toggle_equip_item: world.register_system(toggle_equip_selected_item),
        examine_item: world.register_system(examine_selected_item),
        close_dialog: world.register_system(close_dialog),
    };

    world.insert_resource(callbacks);
}

fn drop_selected_item(
    mut cmds: Commands,
    list_context: Res<ListContext>,
    context: Res<InventoryContext>,
    player_pos: Res<PlayerPosition>,
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

fn toggle_equip_selected_item(
    mut cmds: Commands,
    mut game_state: ResMut<CurrentGameState>,
    list_context: Res<ListContext>,
    q_equippable: Query<&Equippable>,
    q_equipped: Query<&Equipped>,
    mut context: ResMut<InventoryContext>,
    id_registry: Res<StableIdRegistry>,
) {
    let Some(item_id) = list_context.context_data else {
        return;
    };

    let Some(item_entity) = id_registry.get_entity(item_id) else {
        return;
    };

    if q_equipped.get(item_entity).is_ok() {
        cmds.queue(UnequipItemAction::new(item_id));
        return;
    }

    let Ok(equippable) = q_equippable.get(item_entity) else {
        return;
    };

    if equippable.slot_requirements.len() == 1 {
        let Some(player_id) = id_registry.get_id(context.player_entity) else {
            return;
        };

        cmds.queue(EquipItemAction {
            entity_id: player_id,
            item_id,
        });
        return;
    }

    context.selected_item_id = Some(item_id);
    context.available_slots = equippable.slot_requirements.clone();
    game_state.next = GameState::EquipSlotSelect;
}

fn examine_selected_item(
    mut cmds: Commands,
    list_context: Res<ListContext>,
    callbacks: Res<InventoryCallbacks>,
    id_registry: Res<StableIdRegistry>,
    q_labels: Query<&Label>,
    q_glyphs: Query<&Glyph>,
    q_items: Query<&Item>,
    q_equippable: Query<&Equippable>,
    q_weapons: Query<&Weapon>,
    q_stack_counts: Query<&StackCount>,
) {
    let Some(item_id) = list_context.context_data else {
        return;
    };

    let Some(item_entity) = id_registry.get_entity(item_id) else {
        return;
    };

    let mut builder = ItemDialogBuilder::new(item_entity)
        .with_position(5.0, 3.0)
        .with_size(20.0, 8.0)
        .with_close_callback(callbacks.close_dialog);

    // Add Drop button
    builder = builder.add_button(
        ItemDialogButton::new("[{Y|U}] Drop", callbacks.drop_item)
            .with_hotkey(KeyCode::U)
            .with_focus_order(1000),
    );

    // Add Equip button if item is equippable
    if q_equippable.get(item_entity).is_ok() {
        builder = builder.add_button(
            ItemDialogButton::new("[{Y|E}] Equip", callbacks.toggle_equip_item)
                .with_hotkey(KeyCode::E)
                .with_focus_order(2000),
        );
    }

    spawn_item_dialog(
        &mut cmds,
        item_id,
        builder,
        &id_registry,
        &q_labels,
        &q_glyphs,
        &q_items,
        &q_equippable,
        &q_weapons,
        &q_stack_counts,
        CleanupStateInventory,
    );
}

fn close_dialog(
    mut cmds: Commands,
    q_dialogs: Query<Entity, With<Dialog>>,
    mut dialog_state: ResMut<DialogState>,
) {
    for dialog_entity in q_dialogs.iter() {
        cmds.entity(dialog_entity).despawn();
    }
    dialog_state.is_open = false;
}

fn remove_inventory_callbacks(mut cmds: Commands) {
    cmds.remove_resource::<InventoryCallbacks>();
}

pub struct InventoryStatePlugin;

impl Plugin for InventoryStatePlugin {
    fn build(&self, app: &mut App) {
        GameStatePlugin::new(GameState::Inventory)
            .on_enter(
                app,
                (setup_inventory_callbacks, setup_inventory_screen).chain(),
            )
            .on_update(
                app,
                (
                    game_loop,
                    refresh_inventory_display,
                    setup_dialogs,
                    render_dialog_content,
                    setup_buttons,
                )
                    .chain(),
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
    q_equipped: Query<&Equipped>,
    q_stack_counts: Query<&StackCount>,
    id_registry: Res<StableIdRegistry>,
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

    let left_x = 2.0;

    cmds.spawn((
        Text::new("PLAYER INVENTORY")
            .fg1(Palette::Yellow)
            .bg(Palette::Black)
            .layer(Layer::Ui),
        Position::new_f32(left_x, 1., 0.),
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

    let mut list_items = Vec::new();

    for &item_id in inventory.item_ids.iter() {
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
                let slot_name = equipped
                    .slots
                    .first()
                    .map(|slot| slot.display_name())
                    .unwrap_or("Unknown");
                format!("{} {{G|[{}]}}", display_text, slot_name)
            } else {
                display_text
            };

            list_items
                .push(ListItemData::new(&final_text, callbacks.examine_item).with_context(item_id));
        }
    }

    cmds.spawn((
        List::new(list_items).with_focus_order(1000).height(10),
        Position::new_f32(left_x + 1., 3., 0.),
        CleanupStateInventory,
    ));

    let help_y = 3.5 + (inventory.count() as f32 * 0.5) + 1.0;

    cmds.spawn((
        Position::new_f32(left_x, help_y.min(18.), 0.),
        ActivatableBuilder::new("({Y|I}) BACK", callbacks.back_to_explore)
            .with_hotkey(KeyCode::I)
            .with_hotkey(KeyCode::Escape)
            .with_audio(AudioKey::ButtonBack1)
            .with_focus_order(2000)
            .as_button(Layer::Ui),
        CleanupStateInventory,
    ));

    cmds.spawn((
        Position::new_f32(left_x + 4.5, help_y.min(18.), 0.),
        ActivatableBuilder::new("({Y|U}) DROP", callbacks.drop_item)
            .with_hotkey(KeyCode::U)
            .with_focus_order(2100)
            .as_button(Layer::Ui),
        CleanupStateInventory,
    ));

    cmds.spawn((
        Position::new_f32(left_x + 9., help_y.min(18.), 0.),
        ActivatableBuilder::new("({Y|E}) TOGGLE EQUIP", callbacks.toggle_equip_item)
            .with_hotkey(KeyCode::E)
            .with_focus_order(2200)
            .as_button(Layer::Ui),
        CleanupStateInventory,
    ));
}

fn handle_equip_slot_input(mut game_state: ResMut<CurrentGameState>) {
    if is_key_pressed(KeyCode::Escape) {
        game_state.next = GameState::Inventory;
    }
}

fn refresh_inventory_display(
    mut q_list: Query<&mut List, With<CleanupStateInventory>>,
    mut q_weight_text: Query<&mut Text, With<InventoryWeightText>>,
    context: Res<InventoryContext>,
    q_inventory: Query<&Inventory>,
    q_labels: Query<&Label>,
    q_equipped: Query<&Equipped>,
    q_stack_counts: Query<&StackCount>,
    id_registry: Res<StableIdRegistry>,
    callbacks: Res<InventoryCallbacks>,
    mut e_inventory_changed: EventReader<InventoryChangedEvent>,
) {
    let Ok(player_inventory) = q_inventory.get(context.player_entity) else {
        return;
    };

    let Ok(mut list) = q_list.single_mut() else {
        return;
    };

    if !e_inventory_changed.is_empty() {
        e_inventory_changed.clear();

        let list_items = build_inventory_list_items(
            player_inventory,
            &q_labels,
            &q_equipped,
            &q_stack_counts,
            &id_registry,
            &callbacks,
        );

        list.items = list_items;

        if let Ok(mut text) = q_weight_text.single_mut() {
            text.value = format!(
                "Weight: {:.1}/{:.1} kg",
                player_inventory.get_total_weight(),
                player_inventory.capacity
            );
        }
    }
}
