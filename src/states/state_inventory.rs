use bevy_ecs::{prelude::*, schedule::common_conditions::resource_changed, system::SystemId};
use macroquad::input::{KeyCode, is_key_pressed};

use crate::{
    common::Palette,
    domain::{
        Consumable, DropItemAction, EatAction, EquipItemAction, EquipmentSlot, Equippable,
        Equipped, Inventory, Item, Label, LightSource, LightStateChangedEvent, Lightable, Player,
        PlayerPosition, StackCount, ToggleLightAction, UnequipItemAction, Weapon, game_loop,
        inventory::InventoryChangedEvent,
    },
    engine::{App, AudioKey, Plugin, StableIdRegistry},
    rendering::{Glyph, Layer, Position, ScreenSize, Text},
    states::{CurrentGameState, GameState, GameStatePlugin, cleanup_system},
    ui::{
        ActivatableBuilder, Dialog, DialogContent, DialogState, List, ListContext, ListItem,
        ListItemData, UiFocus, center_dialogs_on_screen_change, spawn_examine_dialog,
    },
};

#[derive(Resource)]
struct InventoryCallbacks {
    back_to_explore: SystemId,
    drop_item: SystemId,
    toggle_equip_item: SystemId,
    toggle_light: SystemId,
    eat_item: SystemId,
    show_actions: SystemId,
    close_dialog: SystemId,
    open_item_actions: SystemId,
    examine_item: SystemId,
    close_examine_dialog: SystemId,
}

#[derive(Component, Clone)]
pub struct CleanupStateInventory;

#[derive(Component)]
pub struct CleanupStateEquipSlotSelect;

#[derive(Component)]
pub struct InventoryWeightText;

#[derive(Component)]
pub struct ItemDetailPanel;

#[derive(Component)]
pub struct ItemDetailIcon;

#[derive(Component)]
pub struct ItemDetailName;

#[derive(Component)]
pub struct ItemDetailProperties;

#[derive(Component)]
pub struct ItemActionDialog {
    pub item_id: u64,
}

#[derive(Resource, Default)]
struct ActionDialogRefreshTimer {
    frames_to_wait: u32,
    needs_refresh: bool,
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
                .push(ListItemData::new(&final_text, callbacks.show_actions).with_context(item_id));
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
    let drop_item = world.register_system(drop_selected_item_from_dialog);
    let toggle_equip_item = world.register_system(toggle_equip_selected_item_from_dialog);
    let toggle_light = world.register_system(toggle_light_selected_item_from_dialog);
    let eat_item = world.register_system(eat_selected_item_from_dialog);

    let callbacks = InventoryCallbacks {
        back_to_explore: world.register_system(back_to_explore),
        drop_item,
        toggle_equip_item,
        toggle_light,
        eat_item,
        show_actions: world.register_system(handle_item_click),
        close_dialog: world.register_system(close_dialog),
        open_item_actions: world.register_system(handle_item_click),
        examine_item: world.register_system(examine_item),
        close_examine_dialog: world.register_system(close_examine_dialog),
    };

    world.insert_resource(callbacks);
}

fn handle_item_click(
    cmds: Commands,
    list_context: Res<ListContext>,
    id_registry: Res<StableIdRegistry>,
    q_labels: Query<&Label>,
    q_equippable: Query<&Equippable>,
    q_equipped: Query<&Equipped>,
    q_lightable: Query<&Lightable>,
    q_light_source: Query<&LightSource>,
    q_consumable: Query<&Consumable>,
    dialog_state: ResMut<DialogState>,
    callbacks: Res<InventoryCallbacks>,
    screen: Res<ScreenSize>,
) {
    if let Some(item_id) = list_context.context_data {
        spawn_item_actions_dialog(
            cmds,
            item_id,
            &id_registry,
            &q_labels,
            &q_equippable,
            &q_equipped,
            &q_lightable,
            &q_light_source,
            &q_consumable,
            dialog_state,
            &callbacks,
            &screen,
        );
    }
}

fn build_item_action_list(
    item_id: u64,
    id_registry: &StableIdRegistry,
    q_equippable: &Query<&Equippable>,
    q_equipped: &Query<&Equipped>,
    q_lightable: &Query<&Lightable>,
    q_light_source: &Query<&LightSource>,
    q_consumable: &Query<&Consumable>,
    callbacks: &InventoryCallbacks,
) -> Vec<ListItemData> {
    let Some(item_entity) = id_registry.get_entity(item_id) else {
        return Vec::new();
    };

    let mut list_items = Vec::new();

    list_items.push(ListItemData::new("({Y|D}) Drop", callbacks.drop_item).with_hotkey(KeyCode::D));

    if q_equippable.get(item_entity).is_ok() {
        let label = if q_equipped.get(item_entity).is_ok() {
            "({Y|E}) Unequip"
        } else {
            "({Y|E}) Equip"
        };
        list_items
            .push(ListItemData::new(label, callbacks.toggle_equip_item).with_hotkey(KeyCode::E));
    }

    if q_lightable.get(item_entity).is_ok() && q_light_source.get(item_entity).is_ok() {
        let label = if let Ok(light_source) = q_light_source.get(item_entity) {
            if light_source.is_enabled {
                "({Y|L}) Extinguish"
            } else {
                "({Y|L}) Light"
            }
        } else {
            "({Y|L}) Toggle Light"
        };
        list_items.push(ListItemData::new(label, callbacks.toggle_light).with_hotkey(KeyCode::L));
    }

    if q_consumable.get(item_entity).is_ok() {
        list_items
            .push(ListItemData::new("({Y|C}) Eat", callbacks.eat_item).with_hotkey(KeyCode::C));
    }

    list_items
        .push(ListItemData::new("({Y|X}) Examine", callbacks.examine_item).with_hotkey(KeyCode::X));

    list_items.push(
        ListItemData::new("({Y|ESC}) Close", callbacks.close_dialog).with_hotkey(KeyCode::Escape),
    );

    list_items
}

fn spawn_item_actions_dialog(
    mut cmds: Commands,
    item_id: u64,
    id_registry: &StableIdRegistry,
    q_labels: &Query<&Label>,
    q_equippable: &Query<&Equippable>,
    q_equipped: &Query<&Equipped>,
    q_lightable: &Query<&Lightable>,
    q_light_source: &Query<&LightSource>,
    q_consumable: &Query<&Consumable>,
    mut dialog_state: ResMut<DialogState>,
    callbacks: &InventoryCallbacks,
    screen: &ScreenSize,
) {
    let Some(item_entity) = id_registry.get_entity(item_id) else {
        return;
    };

    let item_name = q_labels
        .get(item_entity)
        .map(|l| l.get())
        .unwrap_or("Unknown Item");

    let list_items = build_item_action_list(
        item_id,
        id_registry,
        q_equippable,
        q_equipped,
        q_lightable,
        q_light_source,
        q_consumable,
        callbacks,
    );

    // Calculate dialog height based on number of actions
    let dialog_width = 24.0;
    let dialog_height = (list_items.len() as f32 + 2.5).min(12.0);
    let dialog_x = ((screen.tile_w as f32 - dialog_width) / 2.0).round();
    let dialog_y = ((screen.tile_h as f32 - dialog_height) / 2.0).round();

    let dialog_entity = cmds
        .spawn((
            Dialog::new(item_name, dialog_width, dialog_height),
            Position::new_f32(dialog_x, dialog_y, 0.0),
            ItemActionDialog { item_id },
            CleanupStateInventory,
        ))
        .id();

    // Spawn the list
    cmds.spawn((
        List::new(list_items).with_focus_order(10000),
        Position::new_f32(dialog_x + 1.0, dialog_y + 1.5, 0.0),
        DialogContent {
            parent_dialog: dialog_entity,
            order: 1,
        },
        CleanupStateInventory,
    ));

    dialog_state.is_open = true;
}

fn close_dialog(
    mut cmds: Commands,
    q_dialogs: Query<Entity, With<Dialog>>,
    q_dialog_content: Query<Entity, With<DialogContent>>,
    mut dialog_state: ResMut<DialogState>,
) {
    for dialog_entity in q_dialogs.iter() {
        cmds.entity(dialog_entity).despawn_recursive();
    }
    for content_entity in q_dialog_content.iter() {
        cmds.entity(content_entity).despawn_recursive();
    }
    dialog_state.is_open = false;
}

fn drop_selected_item_from_dialog(
    mut cmds: Commands,
    context: Res<InventoryContext>,
    player_pos: Res<PlayerPosition>,
    q_action_dialog: Query<&ItemActionDialog>,
    q_dialogs: Query<Entity, With<Dialog>>,
    q_dialog_content: Query<Entity, With<DialogContent>>,
    mut dialog_state: ResMut<DialogState>,
) {
    if let Ok(action_dialog) = q_action_dialog.single() {
        let world_pos = player_pos.world();
        cmds.queue(DropItemAction {
            entity: context.player_entity,
            item_stable_id: action_dialog.item_id,
            drop_position: world_pos,
        });

        // Close dialog
        for dialog_entity in q_dialogs.iter() {
            cmds.entity(dialog_entity).despawn_recursive();
        }
        for content_entity in q_dialog_content.iter() {
            cmds.entity(content_entity).despawn_recursive();
        }
        dialog_state.is_open = false;
    }
}

fn toggle_equip_selected_item_from_dialog(
    mut cmds: Commands,
    mut game_state: ResMut<CurrentGameState>,
    q_equippable: Query<&Equippable>,
    q_equipped: Query<&Equipped>,
    mut context: ResMut<InventoryContext>,
    id_registry: Res<StableIdRegistry>,
    q_action_dialog: Query<&ItemActionDialog>,
    q_dialogs: Query<Entity, With<Dialog>>,
    q_dialog_content: Query<Entity, With<DialogContent>>,
    mut dialog_state: ResMut<DialogState>,
) {
    if let Ok(action_dialog) = q_action_dialog.single() {
        let Some(item_entity) = id_registry.get_entity(action_dialog.item_id) else {
            return;
        };

        if q_equipped.get(item_entity).is_ok() {
            cmds.queue(UnequipItemAction::new(action_dialog.item_id));
        } else if let Ok(equippable) = q_equippable.get(item_entity) {
            if equippable.slot_requirements.len() == 1 {
                let Some(player_id) = id_registry.get_id(context.player_entity) else {
                    return;
                };

                cmds.queue(EquipItemAction {
                    entity_id: player_id,
                    item_id: action_dialog.item_id,
                });
            } else {
                context.selected_item_id = Some(action_dialog.item_id);
                context.available_slots = equippable.slot_requirements.clone();
                game_state.next = GameState::EquipSlotSelect;
                // Close dialog only when going to slot selection
                for dialog_entity in q_dialogs.iter() {
                    cmds.entity(dialog_entity).despawn();
                }
                for content_entity in q_dialog_content.iter() {
                    cmds.entity(content_entity).despawn();
                }
                dialog_state.is_open = false;
            }
        }
    }
}

fn toggle_light_selected_item_from_dialog(
    mut cmds: Commands,
    context: Res<InventoryContext>,
    id_registry: Res<StableIdRegistry>,
    q_lightable: Query<&Lightable>,
    q_light_source: Query<&LightSource>,
    q_action_dialog: Query<&ItemActionDialog>,
) {
    if let Ok(action_dialog) = q_action_dialog.single() {
        let Some(item_entity) = id_registry.get_entity(action_dialog.item_id) else {
            return;
        };

        // Check if item can be lit/extinguished
        if q_lightable.get(item_entity).is_ok() && q_light_source.get(item_entity).is_ok() {
            cmds.queue(ToggleLightAction::new(
                action_dialog.item_id,
                context.player_entity,
            ));
        }
    }
}

fn eat_selected_item_from_dialog(
    mut cmds: Commands,
    context: Res<InventoryContext>,
    id_registry: Res<StableIdRegistry>,
    q_consumable: Query<&Consumable>,
    q_action_dialog: Query<&ItemActionDialog>,
) {
    if let Ok(action_dialog) = q_action_dialog.single() {
        let Some(item_entity) = id_registry.get_entity(action_dialog.item_id) else {
            return;
        };

        // Check if item is consumable
        if q_consumable.get(item_entity).is_ok() {
            let Some(player_id) = id_registry.get_id(context.player_entity) else {
                return;
            };

            cmds.queue(EatAction::new(action_dialog.item_id, player_id));
        }
    }
}

fn examine_item(world: &mut World) {
    let action_dialog = {
        let mut q_action_dialog = world.query::<&ItemActionDialog>();
        q_action_dialog.single(world).ok()
    };

    if let Some(action_dialog) = action_dialog {
        let id_registry = world.get_resource::<StableIdRegistry>().unwrap();
        let Some(item_entity) = id_registry.get_entity(action_dialog.item_id) else {
            return;
        };

        let close_examine_dialog_id = {
            let callbacks = world.get_resource::<InventoryCallbacks>().unwrap();
            callbacks.close_examine_dialog
        };

        // Close action dialog first
        let dialog_entities: Vec<Entity> = {
            let mut q_dialogs = world.query_filtered::<Entity, With<Dialog>>();
            q_dialogs.iter(world).collect()
        };

        let content_entities: Vec<Entity> = {
            let mut q_dialog_content = world.query_filtered::<Entity, With<DialogContent>>();
            q_dialog_content.iter(world).collect()
        };

        for dialog_entity in dialog_entities {
            world.despawn(dialog_entity);
        }
        for content_entity in content_entities {
            world.despawn(content_entity);
        }

        let player_entity = {
            let mut q_player = world.query_filtered::<Entity, With<Player>>();
            q_player.single(world).unwrap()
        };

        // Create examine dialog using the new system
        spawn_examine_dialog(world, item_entity, player_entity, close_examine_dialog_id);

        if let Some(mut dialog_state) = world.get_resource_mut::<DialogState>() {
            dialog_state.is_open = true;
        }
    }
}

fn close_examine_dialog(
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
                    refresh_action_dialog_on_events,
                    refresh_action_dialog_with_timer,
                    update_item_detail_panel,
                )
                    .chain(),
            )
            .on_update(
                app,
                center_dialogs_on_screen_change.run_if(resource_changed::<ScreenSize>),
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

    cmds.init_resource::<ActionDialogRefreshTimer>();

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
                .push(ListItemData::new(&final_text, callbacks.show_actions).with_context(item_id));
        }
    }

    cmds.spawn((
        List::new(list_items).with_focus_order(1000).height(10),
        Position::new_f32(left_x + 1., 3., 0.),
        CleanupStateInventory,
    ));

    // Add item detail panel on the right
    let detail_x = left_x + 20.0;
    let detail_y = 3.0;

    // Panel header
    cmds.spawn((
        Text::new("ITEM DETAILS")
            .fg1(Palette::Yellow)
            .layer(Layer::Ui),
        Position::new_f32(detail_x, detail_y - 1.0, 0.),
        CleanupStateInventory,
        ItemDetailPanel,
    ));

    // Item icon (will be updated when item is focused)
    cmds.spawn((
        Glyph::idx(0)
            .scale((2.0, 2.0))
            .layer(Layer::Ui)
            .fg1(Palette::White),
        Position::new_f32(detail_x + 1.0, detail_y + 1.0, 0.),
        CleanupStateInventory,
        ItemDetailIcon,
    ));

    // Item name
    cmds.spawn((
        Text::new("Select an item")
            .fg1(Palette::Gray)
            .layer(Layer::Ui),
        Position::new_f32(detail_x + 5.0, detail_y + 1.0, 0.),
        CleanupStateInventory,
        ItemDetailName,
    ));

    // Item properties
    cmds.spawn((
        Text::new("").fg1(Palette::White).layer(Layer::Ui),
        Position::new_f32(detail_x + 1.0, detail_y + 4.0, 0.),
        CleanupStateInventory,
        ItemDetailProperties,
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

fn update_item_detail_panel(
    ui_focus: Res<UiFocus>,
    id_registry: Res<StableIdRegistry>,
    q_list_items: Query<&ListItem>,
    q_lists: Query<&List>,
    q_labels: Query<&Label>,
    q_items: Query<&Item>,
    q_equipped: Query<&Equipped>,
    q_stack_counts: Query<&StackCount>,
    q_weapons: Query<&Weapon>,
    mut glyph_queries: ParamSet<(Query<&Glyph>, Query<&mut Glyph, With<ItemDetailIcon>>)>,
    mut q_detail_name: Query<&mut Text, (With<ItemDetailName>, Without<ItemDetailProperties>)>,
    mut q_detail_props: Query<&mut Text, With<ItemDetailProperties>>,
) {
    // Get the focused list item
    let Some(focused_entity) = ui_focus.focused_element else {
        return;
    };

    let Ok(list_item) = q_list_items.get(focused_entity) else {
        return;
    };

    let Ok(list) = q_lists.get(list_item.parent_list) else {
        return;
    };

    // Get the item ID from the list data
    if list_item.index >= list.items.len() {
        return;
    }

    let Some(item_id) = list.items[list_item.index].context_data else {
        return;
    };

    let Some(item_entity) = id_registry.get_entity(item_id) else {
        return;
    };

    // Update icon - first get the item glyph data
    let item_glyph_data = glyph_queries
        .p0()
        .get(item_entity)
        .ok()
        .map(|g| (g.idx, g.fg1, g.fg2, g.texture_id));

    // Then update the detail icon
    if let Some((idx, fg1, fg2, texture_id)) = item_glyph_data {
        for mut detail_glyph in glyph_queries.p1().iter_mut() {
            detail_glyph.idx = idx;
            detail_glyph.fg1 = fg1;
            detail_glyph.fg2 = fg2;
            detail_glyph.texture_id = texture_id;
        }
    }

    // Update name
    for mut detail_text in q_detail_name.iter_mut() {
        if let Ok(label) = q_labels.get(item_entity) {
            detail_text.value = label.get().to_string();
        }
    }

    // Update properties
    for mut detail_text in q_detail_props.iter_mut() {
        let mut props = Vec::new();

        // Weight
        if let Ok(item) = q_items.get(item_entity) {
            props.push(format!("Weight: {:.1} kg", item.weight));
        }

        // Stack count
        if let Ok(stack) = q_stack_counts.get(item_entity)
            && stack.count > 1
        {
            props.push(format!("Quantity: {}", stack.count));
        }

        // Equipped status
        if let Ok(equipped) = q_equipped.get(item_entity) {
            let slots = equipped
                .slots
                .iter()
                .map(|s| s.display_name())
                .collect::<Vec<_>>()
                .join(", ");
            props.push(format!("Equipped: {}", slots));
        }

        // Weapon stats
        if let Ok(weapon) = q_weapons.get(item_entity) {
            props.push(format!("Damage: {}", weapon.damage_dice));
            if let Some(range) = weapon.range {
                props.push(format!("Range: {}", range));
            }
            if let Some(ammo) = weapon.current_ammo
                && let Some(clip) = weapon.clip_size
            {
                props.push(format!("Ammo: {}/{}", ammo, clip));
            }
        }

        detail_text.value = props.join("\n");
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

fn refresh_action_dialog_on_events(
    mut refresh_timer: ResMut<ActionDialogRefreshTimer>,
    mut e_inventory_changed: EventReader<InventoryChangedEvent>,
    mut e_light_changed: EventReader<LightStateChangedEvent>,
) {
    // Check if we have any events to process
    let has_inventory_event = !e_inventory_changed.is_empty();
    let has_light_event = !e_light_changed.is_empty();

    if has_inventory_event || has_light_event {
        // Clear the events
        e_inventory_changed.clear();
        e_light_changed.clear();

        // Set frame counter to wait 2 frames for component changes to be fully applied
        refresh_timer.needs_refresh = true;
        refresh_timer.frames_to_wait = 2;
    }
}

fn refresh_action_dialog_with_timer(
    mut cmds: Commands,
    mut refresh_timer: ResMut<ActionDialogRefreshTimer>,
    q_equippable: Query<&Equippable>,
    q_equipped: Query<&Equipped>,
    q_lightable: Query<&Lightable>,
    q_light_source: Query<&LightSource>,
    q_consumable: Query<&Consumable>,
    mut q_lists: Query<&mut List, With<DialogContent>>,
    id_registry: Res<StableIdRegistry>,
    q_action_dialog: Query<&ItemActionDialog>,
    q_dialogs: Query<Entity, With<Dialog>>,
    q_dialog_content: Query<Entity, With<DialogContent>>,
    mut dialog_state: ResMut<DialogState>,
    callbacks: Res<InventoryCallbacks>,
) {
    if !refresh_timer.needs_refresh {
        return;
    }

    // Decrement frame counter
    if refresh_timer.frames_to_wait > 0 {
        refresh_timer.frames_to_wait -= 1;
        return; // Still waiting
    }

    // Reset the refresh state
    refresh_timer.needs_refresh = false;

    // Only proceed if there's an active action dialog
    let Ok(action_dialog) = q_action_dialog.single() else {
        return;
    };

    // Check if the item still exists (it may have been consumed)
    if id_registry.get_entity(action_dialog.item_id).is_none() {
        // Item no longer exists, close the dialog
        for dialog_entity in q_dialogs.iter() {
            cmds.entity(dialog_entity).despawn_recursive();
        }
        for content_entity in q_dialog_content.iter() {
            cmds.entity(content_entity).despawn_recursive();
        }
        dialog_state.is_open = false;
        return;
    }

    // Find the List entity with DialogContent component (it's the action list in the dialog)
    if let Some(mut list) = q_lists.iter_mut().next() {
        // Build new list items with updated labels
        let new_items = build_item_action_list(
            action_dialog.item_id,
            &id_registry,
            &q_equippable,
            &q_equipped,
            &q_lightable,
            &q_light_source,
            &q_consumable,
            &callbacks,
        );

        // Directly mutate the List's items
        list.items = new_items;
        // Manually trigger change detection so setup_lists will update the UI
        list.set_changed(); // Found and updated, exit early
    }
}
