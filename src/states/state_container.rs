use bevy_ecs::{prelude::*, schedule::common_conditions::resource_changed, system::SystemId};
use macroquad::input::KeyCode;

use crate::{
    common::Palette,
    domain::{
        Inventory, Label, Player, TransferItemAction, game_loop, inventory::InventoryChangedEvent,
    },
    engine::{App, AudioKey, KeyInput, Plugin, StableId, StableIdRegistry},
    rendering::{Layer, Position, ScreenSize, Text},
    states::{CurrentGameState, GameState, GameStatePlugin, cleanup_system},
    ui::{
        ActivatableBuilder, Dialog, DialogState, List, ListContext, ListItem, ListItemData,
        UiFocus, center_dialogs_on_screen_change, spawn_examine_dialog,
    },
};

#[derive(Resource)]
pub struct ContainerCallbacks {
    pub select_item: SystemId,
    pub examine_item: SystemId,
    pub close_dialog: SystemId,
    pub back_to_explore: SystemId,
    pub direct_transfer_from_player: SystemId,
    pub direct_transfer_from_container: SystemId,
}

#[derive(Component, Clone)]
pub struct CleanupStateContainer;

#[derive(Component)]
pub struct PlayerInventoryList;

#[derive(Component)]
pub struct ContainerInventoryList;

#[derive(Component)]
pub struct PlayerInventoryWeightText;

#[derive(Component)]
pub struct ContainerInventoryWeightText;

#[derive(Resource)]
pub struct ContainerContext {
    pub player_entity: Entity,
    pub container_entity: Entity,
}

pub struct ContainerStatePlugin;

impl ContainerStatePlugin {
    pub fn new() -> Self {
        Self
    }
}

impl Plugin for ContainerStatePlugin {
    fn build(&self, app: &mut App) {
        GameStatePlugin::new(GameState::Container)
            .on_enter(
                app,
                (setup_container_callbacks, setup_container_screen).chain(),
            )
            .on_update(
                app,
                (handle_container_input, refresh_container_display, game_loop).chain(),
            )
            .on_update(
                app,
                center_dialogs_on_screen_change.run_if(resource_changed::<ScreenSize>),
            )
            .on_leave(
                app,
                (
                    cleanup_system::<CleanupStateContainer>,
                    remove_container_callbacks,
                )
                    .chain(),
            );
    }
}

fn setup_container_callbacks(world: &mut World) {
    let callbacks = ContainerCallbacks {
        select_item: world.register_system(select_item),
        examine_item: world.register_system(examine_selected_item),
        close_dialog: world.register_system(close_dialog),
        back_to_explore: world.register_system(back_to_explore),
        direct_transfer_from_player: world.register_system(direct_transfer_from_player),
        direct_transfer_from_container: world.register_system(direct_transfer_from_container),
    };
    world.insert_resource(callbacks);
}

fn remove_container_callbacks(mut cmds: Commands) {
    cmds.remove_resource::<ContainerCallbacks>();
}

fn select_item() {
    // No-op for empty slots
}

fn back_to_explore(mut game_state: ResMut<CurrentGameState>) {
    game_state.next = GameState::Explore;
}

fn transfer_item_from_player(
    mut cmds: Commands,
    list_context: Res<ListContext>,
    context: Res<ContainerContext>,
    q_dialogs: Query<Entity, With<Dialog>>,
    mut dialog_state: ResMut<DialogState>,
) {
    if let Some(item_id) = list_context.context_data {
        cmds.queue(TransferItemAction {
            from_entity: context.player_entity,
            to_entity: context.container_entity,
            item_stable_id: StableId(item_id),
        });

        // Close dialog after transfer
        for dialog_entity in q_dialogs.iter() {
            cmds.entity(dialog_entity).despawn();
        }
        dialog_state.is_open = false;
    }
}

fn transfer_item_from_container(
    mut cmds: Commands,
    list_context: Res<ListContext>,
    context: Res<ContainerContext>,
    q_dialogs: Query<Entity, With<Dialog>>,
    mut dialog_state: ResMut<DialogState>,
) {
    if let Some(item_id) = list_context.context_data {
        cmds.queue(TransferItemAction {
            from_entity: context.container_entity,
            to_entity: context.player_entity,
            item_stable_id: StableId(item_id),
        });

        // Close dialog after transfer
        for dialog_entity in q_dialogs.iter() {
            cmds.entity(dialog_entity).despawn();
        }
        dialog_state.is_open = false;
    }
}

fn direct_transfer_from_player(
    mut cmds: Commands,
    ui_focus: Res<UiFocus>,
    context: Res<ContainerContext>,
    q_player_lists: Query<Entity, With<PlayerInventoryList>>,
    q_list_items: Query<&ListItem>,
    q_inventory: Query<&Inventory>,
    _id_registry: Res<StableIdRegistry>,
) {
    let Some(focused_entity) = ui_focus.focused_element else {
        return;
    };

    let Ok(focused_list_item) = q_list_items.get(focused_entity) else {
        return;
    };

    let Ok(player_list_entity) = q_player_lists.single() else {
        return;
    };

    // Check if the focused list item belongs to the player inventory list
    if focused_list_item.parent_list == player_list_entity {
        let Ok(player_inventory) = q_inventory.get(context.player_entity) else {
            return;
        };

        // Use the focused item's index to get the item ID
        let item_index = focused_list_item.index;
        if item_index < player_inventory.item_ids.len() {
            let item_id = player_inventory.item_ids[item_index];

            cmds.queue(TransferItemAction {
                from_entity: context.player_entity,
                to_entity: context.container_entity,
                item_stable_id: StableId(item_id),
            });
        }
    }
}

fn direct_transfer_from_container(
    mut cmds: Commands,
    ui_focus: Res<UiFocus>,
    context: Res<ContainerContext>,
    q_container_lists: Query<Entity, With<ContainerInventoryList>>,
    q_list_items: Query<&ListItem>,
    q_inventory: Query<&Inventory>,
    _id_registry: Res<StableIdRegistry>,
) {
    let Some(focused_entity) = ui_focus.focused_element else {
        return;
    };

    let Ok(focused_list_item) = q_list_items.get(focused_entity) else {
        return;
    };

    let Ok(container_list_entity) = q_container_lists.single() else {
        return;
    };

    // Check if the focused list item belongs to the container inventory list
    if focused_list_item.parent_list == container_list_entity {
        let Ok(container_inventory) = q_inventory.get(context.container_entity) else {
            return;
        };

        // Use the focused item's index to get the item ID
        let item_index = focused_list_item.index;
        if item_index < container_inventory.item_ids.len() {
            let item_id = container_inventory.item_ids[item_index];

            cmds.queue(TransferItemAction {
                from_entity: context.container_entity,
                to_entity: context.player_entity,
                item_stable_id: StableId(item_id),
            });
        }
    }
}

fn build_player_list_items(
    inventory: &Inventory,
    q_labels: &Query<&Label>,
    id_registry: &StableIdRegistry,
    callbacks: &ContainerCallbacks,
) -> Vec<ListItemData> {
    inventory
        .item_ids
        .iter()
        .map(|item_id| {
            let Some(item_entity) = id_registry.get_entity(StableId(*item_id)) else {
                return ListItemData::new("Unknown", callbacks.select_item);
            };

            let display_text = if let Ok(label) = q_labels.get(item_entity) {
                label.get()
            } else {
                "Unknown"
            };

            ListItemData::new(display_text, callbacks.examine_item)
                .with_hotkey(KeyCode::X)
                .with_context(*item_id)
        })
        .collect()
}

fn build_container_list_items(
    inventory: &Inventory,
    q_labels: &Query<&Label>,
    id_registry: &StableIdRegistry,
    callbacks: &ContainerCallbacks,
) -> Vec<ListItemData> {
    let mut items = Vec::new();

    for &item_id in inventory.item_ids.iter() {
        if let Some(item_entity) = id_registry.get_entity(StableId(item_id)) {
            let display_text = if let Ok(label) = q_labels.get(item_entity) {
                label.get()
            } else {
                "Unknown"
            };

            items.push(
                ListItemData::new(display_text, callbacks.examine_item)
                    .with_hotkey(KeyCode::X)
                    .with_context(item_id),
            );
        }
    }

    items
}

fn examine_selected_item(world: &mut World) {
    let list_context = world.get_resource::<ListContext>().unwrap();
    let Some(item_id) = list_context.context_data else {
        return;
    };

    let id_registry = world.get_resource::<StableIdRegistry>().unwrap();
    let Some(item_entity) = id_registry.get_entity(StableId(item_id)) else {
        return;
    };

    let close_dialog_id = {
        let callbacks = world.get_resource::<ContainerCallbacks>().unwrap();
        callbacks.close_dialog
    };

    let player_entity = {
        let mut q_player = world.query_filtered::<Entity, With<Player>>();
        q_player.single(world).unwrap()
    };

    spawn_examine_dialog(world, item_entity, player_entity, close_dialog_id);

    if let Some(mut dialog_state) = world.get_resource_mut::<DialogState>() {
        dialog_state.is_open = true;
    }
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

fn setup_container_screen(
    mut cmds: Commands,
    callbacks: Res<ContainerCallbacks>,
    q_player: Query<Entity, With<Player>>,
    q_inventory: Query<&Inventory>,
    q_labels: Query<&Label>,
    id_registry: Res<StableIdRegistry>,
    context: Option<Res<ContainerContext>>,
) {
    let Ok(player_entity) = q_player.single() else {
        return;
    };

    let Ok(player_inventory) = q_inventory.get(player_entity) else {
        return;
    };

    let Some(context) = context else {
        return;
    };

    let Ok(container_inventory) = q_inventory.get(context.container_entity) else {
        return;
    };

    let left_x = 2.0;
    let right_x = 21.0;

    cmds.spawn((
        Text::new("PLAYER INVENTORY")
            .fg1(Palette::Yellow)
            .bg(Palette::Black)
            .layer(Layer::Ui),
        Position::new_f32(left_x, 1., 0.),
        CleanupStateContainer,
    ));

    cmds.spawn((
        Text::new(&format!(
            "Weight: {:.1}/{:.1} kg",
            player_inventory.get_total_weight(),
            player_inventory.capacity
        ))
        .fg1(Palette::White)
        .layer(Layer::Ui),
        Position::new_f32(left_x, 2., 0.),
        CleanupStateContainer,
        PlayerInventoryWeightText,
    ));

    let player_list_items =
        build_player_list_items(player_inventory, &q_labels, &id_registry, &callbacks);

    let _player_list_entity = cmds
        .spawn((
            List::new(player_list_items).with_focus_order(1000),
            Position::new_f32(left_x, 3.5, 0.),
            CleanupStateContainer,
            PlayerInventoryList,
        ))
        .id();

    let container_label = if let Ok(label) = q_labels.get(context.container_entity) {
        label.get()
    } else {
        "Unknown Container"
    };

    cmds.spawn((
        Text::new(container_label)
            .fg1(Palette::Yellow)
            .bg(Palette::Black)
            .layer(Layer::Ui),
        Position::new_f32(right_x, 1., 0.),
        CleanupStateContainer,
    ));

    cmds.spawn((
        Text::new(&format!(
            "Weight: {:.1}/{:.1} kg",
            container_inventory.get_total_weight(),
            container_inventory.capacity
        ))
        .fg1(Palette::White)
        .layer(Layer::Ui),
        Position::new_f32(right_x, 2., 0.),
        CleanupStateContainer,
        ContainerInventoryWeightText,
    ));

    let container_list_items =
        build_container_list_items(container_inventory, &q_labels, &id_registry, &callbacks);

    let start_y = 3.5;
    cmds.spawn((
        List::new(container_list_items).with_focus_order(2000),
        Position::new_f32(right_x, start_y, 0.),
        CleanupStateContainer,
        ContainerInventoryList,
    ));

    let help_y = 12.0; // Fixed position near bottom

    // Back button
    cmds.spawn((
        Position::new_f32(left_x, help_y, 0.),
        ActivatableBuilder::new("({Y|I}) BACK", callbacks.back_to_explore)
            .with_hotkey(KeyCode::I)
            .with_hotkey(KeyCode::Escape)
            .with_audio(AudioKey::ButtonBack1)
            .with_focus_order(3000)
            .as_button(Layer::Ui),
        CleanupStateContainer,
    ));

    // Help text for navigation
    cmds.spawn((
        Text::new("  [{Y|TAB}] Switch Side   [{Y|X}] Examine   [{Y|T}] Quick Transfer")
            .fg1(Palette::White)
            .layer(Layer::Ui),
        Position::new_f32(left_x + 8.0, help_y, 0.),
        CleanupStateContainer,
    ));
}

fn refresh_container_display(
    q_inventory: Query<&Inventory>,
    q_labels: Query<&Label>,
    mut q_lists: ParamSet<(
        Query<&mut List, With<PlayerInventoryList>>,
        Query<&mut List, With<ContainerInventoryList>>,
    )>,
    mut q_weight_texts: ParamSet<(
        Query<&mut Text, With<PlayerInventoryWeightText>>,
        Query<&mut Text, With<ContainerInventoryWeightText>>,
    )>,
    context: Res<ContainerContext>,
    id_registry: Res<StableIdRegistry>,
    callbacks: Res<ContainerCallbacks>,
    mut e_inventory_changed: EventReader<InventoryChangedEvent>,
) {
    if e_inventory_changed.is_empty() {
        return;
    }
    e_inventory_changed.clear();

    let Ok(player_inventory) = q_inventory.get(context.player_entity) else {
        return;
    };

    let Ok(container_inventory) = q_inventory.get(context.container_entity) else {
        return;
    };

    if let Ok(mut player_list) = q_lists.p0().single_mut() {
        let player_list_items =
            build_player_list_items(player_inventory, &q_labels, &id_registry, &callbacks);
        player_list.items = player_list_items;
    }

    if let Ok(mut container_list) = q_lists.p1().single_mut() {
        let container_list_items =
            build_container_list_items(container_inventory, &q_labels, &id_registry, &callbacks);
        container_list.items = container_list_items;
    }
    if let Ok(mut text) = q_weight_texts.p0().single_mut() {
        text.value = format!(
            "Weight: {:.1}/{:.1} kg",
            player_inventory.get_total_weight(),
            player_inventory.capacity
        );
    }

    if let Ok(mut text) = q_weight_texts.p1().single_mut() {
        text.value = format!(
            "Weight: {:.1}/{:.1} kg",
            container_inventory.get_total_weight(),
            container_inventory.capacity
        );
    }
}

fn handle_container_input(
    keys: Res<KeyInput>,
    mut game_state: ResMut<CurrentGameState>,
    ui_focus: Res<UiFocus>,
    q_player_lists: Query<Entity, With<PlayerInventoryList>>,
    q_container_lists: Query<Entity, With<ContainerInventoryList>>,
    q_list_items: Query<&ListItem>,
    callbacks: Res<ContainerCallbacks>,
    mut commands: Commands,
) {
    if keys.is_pressed(KeyCode::I) {
        game_state.next = GameState::Explore;
        return;
    }

    if keys.is_pressed(KeyCode::T)
        && let Some(focused_entity) = ui_focus.focused_element
        && let Ok(focused_list_item) = q_list_items.get(focused_entity)
    {
        let Ok(player_list_entity) = q_player_lists.single() else {
            return;
        };
        let Ok(container_list_entity) = q_container_lists.single() else {
            return;
        };

        if focused_list_item.parent_list == player_list_entity {
            commands.run_system(callbacks.direct_transfer_from_player);
        } else if focused_list_item.parent_list == container_list_entity {
            commands.run_system(callbacks.direct_transfer_from_container);
        }
    }
}
