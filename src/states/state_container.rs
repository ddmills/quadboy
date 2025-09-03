use bevy_ecs::{prelude::*, system::SystemId};
use macroquad::input::KeyCode;

use crate::{
    common::Palette,
    domain::{Equipped, Inventory, Label, Player, StackCount, TransferItemAction, game_loop},
    engine::{App, KeyInput, Plugin, StableIdRegistry},
    rendering::{Glyph, Layer, Position, Text},
    states::{CurrentGameState, GameState, GameStatePlugin, cleanup_system},
    ui::{List, ListContext, ListFocus, ListItemData, ListState},
};

#[derive(Resource)]
pub struct ContainerCallbacks {
    pub select_item: SystemId,
    pub transfer_from_player: SystemId,
    pub transfer_from_container: SystemId,
}

#[derive(Component)]
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
                (handle_container_input, refresh_container_display, game_loop),
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
        transfer_from_player: world.register_system(transfer_item_from_player),
        transfer_from_container: world.register_system(transfer_item_from_container),
    };
    world.insert_resource(callbacks);
}

fn remove_container_callbacks(mut cmds: Commands) {
    cmds.remove_resource::<ContainerCallbacks>();
}

fn select_item() {
    // No-op for empty slots
}

fn transfer_item_from_player(
    mut cmds: Commands,
    list_context: Res<ListContext>,
    context: Res<ContainerContext>,
) {
    if let Some(item_id) = list_context.context_data {
        cmds.queue(TransferItemAction {
            from_entity: context.player_entity,
            to_entity: context.container_entity,
            item_stable_id: item_id,
        });
    }
}

fn transfer_item_from_container(
    mut cmds: Commands,
    list_context: Res<ListContext>,
    context: Res<ContainerContext>,
) {
    if let Some(item_id) = list_context.context_data {
        cmds.queue(TransferItemAction {
            from_entity: context.container_entity,
            to_entity: context.player_entity,
            item_stable_id: item_id,
        });
    }
}

fn build_player_list_items(
    inventory: &Inventory,
    q_labels: &Query<&Label>,
    q_glyphs: &Query<&Glyph>,
    q_equipped: &Query<&Equipped>,
    q_stack_counts: &Query<&StackCount>,
    id_registry: &StableIdRegistry,
    callbacks: &ContainerCallbacks,
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
                callback: callbacks.transfer_from_player,
                hotkey: None,
                icon: None,
                context_data: Some(item_id),
            });
        }
    }

    items
}

fn build_container_list_items(
    inventory: &Inventory,
    q_labels: &Query<&Label>,
    q_glyphs: &Query<&Glyph>,
    q_equipped: &Query<&Equipped>,
    q_stack_counts: &Query<&StackCount>,
    id_registry: &StableIdRegistry,
    callbacks: &ContainerCallbacks,
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
                callback: callbacks.transfer_from_container,
                hotkey: None,
                icon: None,
                context_data: Some(item_id),
            });
        }
    }

    items
}

fn setup_container_screen(
    mut cmds: Commands,
    callbacks: Res<ContainerCallbacks>,
    q_player: Query<Entity, With<Player>>,
    q_inventory: Query<&Inventory>,
    q_labels: Query<&Label>,
    q_glyphs: Query<&Glyph>,
    q_equipped: Query<&Equipped>,
    q_stack_counts: Query<&StackCount>,
    id_registry: Res<StableIdRegistry>,
    mut list_focus: ResMut<ListFocus>,
    context: Option<Res<ContainerContext>>,
) {
    let Ok(player_entity) = q_player.single() else {
        return;
    };

    let Ok(player_inventory) = q_inventory.get(player_entity) else {
        return;
    };

    // Get container entity from existing context
    let Some(context) = context else {
        return;
    };

    let Ok(container_inventory) = q_inventory.get(context.container_entity) else {
        return;
    };

    let left_x = 2.0;
    let right_x = 15.0;

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

    // Build player inventory list
    let player_list_items = build_player_list_items(
        player_inventory,
        &q_labels,
        &q_glyphs,
        &q_equipped,
        &q_stack_counts,
        &id_registry,
        &callbacks,
    );

    // Spawn player inventory list
    let player_list_entity = cmds
        .spawn((
            List::new(player_list_items),
            ListState::new().with_focus(true), // Start with focus on player inventory
            Position::new_f32(left_x + 1., 3.5, 0.),
            CleanupStateContainer,
            PlayerInventoryList,
        ))
        .id();

    // Right side - Container inventory
    let container_label = if let Ok(label) = q_labels.get(context.container_entity) {
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

    // Build container inventory list
    let container_list_items = build_container_list_items(
        container_inventory,
        &q_labels,
        &q_glyphs,
        &q_equipped,
        &q_stack_counts,
        &id_registry,
        &callbacks,
    );

    // Spawn container inventory list
    let start_y = 3.5;
    let _container_list_entity = cmds
        .spawn((
            List::new(container_list_items),
            ListState::new(),
            Position::new_f32(right_x, start_y, 0.),
            CleanupStateContainer,
            ContainerInventoryList,
        ))
        .id();

    // Set focus to player list initially
    list_focus.active_list = Some(player_list_entity);

    // Update focus states
    cmds.entity(player_list_entity)
        .insert(ListState::new().with_focus(true));

    // Help text
    let help_y = 17.0; // Fixed position near bottom
    cmds.spawn((
        Text::new("[{Y|I}] Back   [{Y|TAB}] Switch Side   [{Y|ENTER}] Transfer")
            .fg1(Palette::White)
            .layer(Layer::Ui),
        Position::new_f32(left_x, help_y, 0.),
        CleanupStateContainer,
    ));
}

fn refresh_container_display(
    q_inventory: Query<&Inventory>,
    q_labels: Query<&Label>,
    q_glyphs: Query<&Glyph>,
    q_equipped: Query<&Equipped>,
    q_stack_counts: Query<&StackCount>,
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
) {
    let Ok(player_inventory) = q_inventory.get(context.player_entity) else {
        return;
    };

    let Ok(container_inventory) = q_inventory.get(context.container_entity) else {
        return;
    };

    // Update player list
    if let Ok(mut player_list) = q_lists.p0().single_mut() {
        let player_list_items = build_player_list_items(
            player_inventory,
            &q_labels,
            &q_glyphs,
            &q_equipped,
            &q_stack_counts,
            &id_registry,
            &callbacks,
        );
        // Only update if items have changed to prevent flickering
        if player_list.items.len() != player_list_items.len()
            || player_list
                .items
                .iter()
                .zip(player_list_items.iter())
                .any(|(a, b)| a.label != b.label)
        {
            player_list.items = player_list_items;
        }
    }

    // Update container list
    if let Ok(mut container_list) = q_lists.p1().single_mut() {
        let container_list_items = build_container_list_items(
            container_inventory,
            &q_labels,
            &q_glyphs,
            &q_equipped,
            &q_stack_counts,
            &id_registry,
            &callbacks,
        );
        // Only update if items have changed to prevent flickering
        if container_list.items.len() != container_list_items.len()
            || container_list
                .items
                .iter()
                .zip(container_list_items.iter())
                .any(|(a, b)| a.label != b.label)
        {
            container_list.items = container_list_items;
        }
    }

    // Update weight display texts
    // Update player inventory weight text
    if let Ok(mut text) = q_weight_texts.p0().single_mut() {
        text.value = format!(
            "Weight: {:.1}/{:.1} kg",
            player_inventory.get_total_weight(),
            player_inventory.capacity
        );
    }

    // Update container inventory weight text
    if let Ok(mut text) = q_weight_texts.p1().single_mut() {
        text.value = format!(
            "Weight: {:.1}/{:.1} kg",
            container_inventory.get_total_weight(),
            container_inventory.capacity
        );
    }
}

fn handle_container_input(keys: Res<KeyInput>, mut game_state: ResMut<CurrentGameState>) {
    if keys.is_pressed(KeyCode::I) {
        game_state.next = GameState::Explore;
        return;
    }
    // Tab switching and item selection are now handled by List components
}
