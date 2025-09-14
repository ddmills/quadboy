use bevy_ecs::{prelude::*, system::SystemId};
use macroquad::input::KeyCode;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::{
    common::Palette,
    domain::{PlayerPosition, Prefab, PrefabId, Prefabs, game_loop},
    engine::{AudioKey, Clock, KeyInput, Mouse, Plugin, SerializableComponent},
    rendering::{Layer, Position, Text},
    states::{CurrentGameState, GameState, GameStatePlugin, cleanup_system},
    ui::{
        ActivatableBuilder, List, ListContext, ListItemData, SelectableList, SelectableListState,
        SelectionMode,
    },
};

const SPAWNABLE_PREFABS: &[PrefabId] = &[
    PrefabId::PineTree,
    PrefabId::Boulder,
    PrefabId::Campfire,
    PrefabId::GoldNugget,
    PrefabId::Cactus,
    PrefabId::CavalrySword,
    PrefabId::Chest,
    PrefabId::GiantMushroom,
    PrefabId::Bandit,
    PrefabId::Hatchet,
    PrefabId::Lantern,
    PrefabId::Pickaxe,
    PrefabId::StairDown,
    PrefabId::StairUp,
    PrefabId::Dynamite,
    PrefabId::Apple,
    PrefabId::Bedroll,
    PrefabId::LongJohns,
    PrefabId::Duster,
    PrefabId::Poncho,
    PrefabId::Overcoat,
    PrefabId::WoolShirt,
    PrefabId::SteelToeBoots,
    PrefabId::LeverActionRifle,
    PrefabId::DoubleBarrelShotgun,
    PrefabId::NavyRevolver,
    PrefabId::Amulet,
    PrefabId::Ring,
];

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct CleanupStateDebugSpawn;

#[derive(Component)]
pub struct SelectedPrefabDisplay;

#[derive(Resource)]
struct DebugSpawnCallbacks {
    exit_debug_spawn: SystemId,
}

pub struct DebugSpawnStatePlugin;

impl Plugin for DebugSpawnStatePlugin {
    fn build(&self, app: &mut crate::engine::App) {
        GameStatePlugin::new(GameState::DebugSpawn)
            .on_enter(app, (setup_callbacks, on_enter_debug_spawn).chain())
            .on_update(
                app,
                (
                    handle_input,
                    handle_spawn_click,
                    update_selected_prefab_display,
                ),
            )
            .on_leave(
                app,
                (
                    cleanup_system::<CleanupStateDebugSpawn>,
                    remove_debug_spawn_callbacks,
                )
                    .chain(),
            );
    }
}

fn setup_callbacks(world: &mut World) {
    let callbacks = DebugSpawnCallbacks {
        exit_debug_spawn: world.register_system(exit_debug_spawn),
    };

    world.insert_resource(callbacks);
}

fn exit_debug_spawn(mut game_state: ResMut<CurrentGameState>) {
    game_state.next = GameState::Explore;
}

fn remove_debug_spawn_callbacks(mut cmds: Commands) {
    cmds.remove_resource::<DebugSpawnCallbacks>();
}

fn on_enter_debug_spawn(mut cmds: Commands, callbacks: Res<DebugSpawnCallbacks>) {
    cmds.spawn((
        Text::new("SPAWN PREFAB MODE (DEBUG)")
            .fg1(Palette::Yellow)
            .bg(Palette::Black),
        Position::new_f32(1.0, 1.0, 0.0),
        CleanupStateDebugSpawn,
    ));

    cmds.spawn((
        Text::new("Select a prefab, click to spawn")
            .fg1(Palette::White)
            .bg(Palette::Black),
        Position::new_f32(14.0, 1.0, 0.0),
        CleanupStateDebugSpawn,
    ));

    cmds.spawn((
        ActivatableBuilder::new("({Y|ESC}) Back", callbacks.exit_debug_spawn)
            .with_hotkey(KeyCode::Escape)
            .with_audio(AudioKey::ButtonBack1)
            .as_button(Layer::Ui),
        Position::new_f32(1.0, 2.0, 0.0),
        CleanupStateDebugSpawn,
    ));

    cmds.spawn((
        Text::new("Selected: None")
            .fg1(Palette::Green)
            .bg(Palette::Black),
        Position::new_f32(6.5, 2.0, 0.0),
        SelectedPrefabDisplay,
        CleanupStateDebugSpawn,
    ));

    let list_items = SPAWNABLE_PREFABS
        .iter()
        .enumerate()
        .map(|(index, prefab_id)| {
            ListItemData::new(&format!("{}", prefab_id), callbacks.exit_debug_spawn)
                .with_context(index as u64)
        })
        .collect();

    let list_entity = cmds
        .spawn((
            List {
                items: list_items,
                width: 12.0,
                focus_order: Some(1),
                selected_index: 0,
                height: Some(16),
                scroll_offset: 0,
            },
            SelectableList {
                selection_mode: SelectionMode::Single,
                on_selection_change: None,
            },
            SelectableListState {
                selected_indices: {
                    let mut set = HashSet::new();
                    set.insert(0); // Start with first item selected
                    set
                },
            },
            Position::new_f32(1.0, 3.0, 0.0),
            CleanupStateDebugSpawn,
        ))
        .id();

    cmds.insert_resource(ListContext {
        activated_item_index: 0,
        activated_list: list_entity,
        context_data: Some(0),
    });
}

fn handle_input(keys: Res<KeyInput>, mut game_state: ResMut<CurrentGameState>) {
    if keys.is_pressed(KeyCode::Escape) {
        game_state.next = GameState::Explore;
    }
}

fn handle_spawn_click(world: &mut World) {
    // First collect all the data we need without holding borrows
    let (_mouse_pressed, world_pos, list_entity, player_z) = {
        let Some(mouse) = world.get_resource::<Mouse>() else {
            return;
        };
        let Some(list_context) = world.get_resource::<ListContext>() else {
            return;
        };
        let Some(player_pos) = world.get_resource::<PlayerPosition>() else {
            return;
        };

        if !mouse.left_just_pressed {
            return;
        }

        if mouse.is_captured {
            return;
        }

        (
            mouse.left_just_pressed,
            mouse.world,
            list_context.activated_list,
            player_pos.z,
        )
    };

    let selected_index = {
        let mut query = world.query::<&SelectableListState>();
        let Ok(state) = query.get(world, list_entity) else {
            return;
        };

        let Some(&selected_index) = state.selected_indices.iter().next() else {
            return;
        };
        selected_index
    };

    if selected_index >= SPAWNABLE_PREFABS.len() {
        return;
    }

    let selected_prefab = SPAWNABLE_PREFABS[selected_index].clone();
    let spawn_pos = (
        world_pos.0.floor() as usize,
        world_pos.1.floor() as usize,
        player_z.floor() as usize,
    );

    Prefabs::spawn_world(world, Prefab::new(selected_prefab, spawn_pos));

    let Some(mut clock) = world.get_resource_mut::<Clock>() else {
        return;
    };
    clock.force_update();

    game_loop(world);
}

fn update_selected_prefab_display(
    list_context: Res<ListContext>,
    q_lists: Query<&SelectableListState>,
    mut q_selected_display: Query<&mut Text, With<SelectedPrefabDisplay>>,
) {
    let Ok(mut text) = q_selected_display.single_mut() else {
        return;
    };

    // Get the SelectableListState from the list entity
    let Ok(state) = q_lists.get(list_context.activated_list) else {
        text.value = "Selected: None".to_string();
        return;
    };

    // Get the selected index
    let Some(&selected_index) = state.selected_indices.iter().next() else {
        text.value = "Selected: None".to_string();
        return;
    };

    if selected_index >= SPAWNABLE_PREFABS.len() {
        text.value = "Selected: None".to_string();
        return;
    }

    let selected_prefab = &SPAWNABLE_PREFABS[selected_index];
    text.value = format!("Selected: {}", selected_prefab);
}
