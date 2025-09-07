use bevy_ecs::{prelude::*, system::SystemId};
use macroquad::{input::KeyCode, prelude::trace};
use serde::{Deserialize, Serialize};

use crate::{
    common::Palette,
    domain::{game_loop, PlayerPosition, Prefab, PrefabId, Prefabs, Terrain},
    engine::{Clock, KeyInput, Mouse, Plugin, SerializableComponent},
    rendering::{Position, Text},
    states::{cleanup_system, CurrentGameState, GameState, GameStatePlugin},
    ui::{List, ListContext, ListItemData, ListState},
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
    PrefabId::TerrainTile(Terrain::Grass),
    PrefabId::TerrainTile(Terrain::Dirt),
    PrefabId::TerrainTile(Terrain::River),
    PrefabId::TerrainTile(Terrain::Sand),
    PrefabId::TerrainTile(Terrain::Shallows),
    PrefabId::TerrainTile(Terrain::OpenAir),
];

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct CleanupStateDebugSpawn;

#[derive(Component)]
pub struct MousePositionDisplay;

#[derive(Component)]
pub struct SelectedPrefabDisplay;

#[derive(Resource)]
struct DebugSpawnCallbacks {
    exit_debug_spawn: SystemId,
    spawn_prefab: SystemId,
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
                    update_mouse_position_display,
                    update_selected_prefab_display,
                ),
            )
            .on_leave(
                app,
                (
                    on_leave_debug_spawn,
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
        spawn_prefab: world.register_system(spawn_prefab),
    };

    world.insert_resource(callbacks);
}

fn exit_debug_spawn(mut game_state: ResMut<CurrentGameState>) {
    game_state.next = GameState::Explore;
}

fn spawn_prefab(
    mut cmds: Commands,
    mouse: Res<Mouse>,
    list_context: Res<ListContext>,
) {
    let selected_index = list_context.activated_item_index;
    if selected_index >= SPAWNABLE_PREFABS.len() {
        return;
    }

    let selected_prefab = SPAWNABLE_PREFABS[selected_index].clone();
    let world_pos = mouse.world;
    let spawn_pos = (
        world_pos.0.floor() as usize,
        world_pos.1.floor() as usize,
        0, // Default z-level
    );

    Prefabs::spawn(&mut cmds, Prefab::new(selected_prefab, spawn_pos));
}

fn remove_debug_spawn_callbacks(mut cmds: Commands) {
    cmds.remove_resource::<DebugSpawnCallbacks>();
}

fn on_enter_debug_spawn(mut cmds: Commands, callbacks: Res<DebugSpawnCallbacks>) {
    cmds.spawn((
        Text::new("Debug Spawn Mode").fg1(Palette::Yellow).bg(Palette::Black),
        Position::new_f32(1.0, 1.0, 0.0),
        CleanupStateDebugSpawn,
    ));

    cmds.spawn((
        Text::new("ESC: Exit | Click: Spawn | Arrow Keys: Navigate").fg1(Palette::White).bg(Palette::Black),
        Position::new_f32(1.0, 2.0, 0.0),
        CleanupStateDebugSpawn,
    ));

    cmds.spawn((
        Text::new("Select Prefab:").fg1(Palette::White).bg(Palette::Black),
        Position::new_f32(1.0, 4.0, 0.0),
        CleanupStateDebugSpawn,
    ));

    cmds.spawn((
        Text::new("Mouse: (0, 0)").fg1(Palette::Gray).bg(Palette::Black),
        Position::new_f32(1.0, 30.0, 0.0),
        MousePositionDisplay,
        CleanupStateDebugSpawn,
    ));

    cmds.spawn((
        Text::new("Selected: None").fg1(Palette::Green).bg(Palette::Black),
        Position::new_f32(1.0, 31.0, 0.0),
        SelectedPrefabDisplay,
        CleanupStateDebugSpawn,
    ));

    let mut list_items = Vec::new();
    for (index, prefab_id) in SPAWNABLE_PREFABS.iter().enumerate() {
        let item = ListItemData::new(
            &format!("{}", prefab_id),
            callbacks.spawn_prefab,
        )
        .with_context(index as u64);
        list_items.push(item);
    }

    let list_entity = cmds.spawn((
        List {
            items: list_items,
            width: 10.0,
            focus_order: Some(1),
        },
        ListState::new(),
        Position::new_f32(1.0, 5.0, 0.0),
        CleanupStateDebugSpawn,
    )).id();

    cmds.insert_resource(ListContext {
        activated_item_index: 0,
        activated_list: list_entity,
        context_data: Some(0),
    });
}

fn on_leave_debug_spawn() {
    // ListContext is a global resource, don't remove it
}

fn handle_input(
    keys: Res<KeyInput>,
    mut game_state: ResMut<CurrentGameState>,
) {
    if keys.is_pressed(KeyCode::Escape) {
        game_state.next = GameState::Explore;
    }
}

fn handle_spawn_click(
    world: &mut World,
) {
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

    let selected_index = list_context.activated_item_index;

    trace!("selected_index, {}", selected_index);
    if selected_index >= SPAWNABLE_PREFABS.len() {
        return;
    }

    let selected_prefab = SPAWNABLE_PREFABS[selected_index].clone();
    let world_pos = mouse.world;
    let spawn_pos = (
        world_pos.0.floor() as usize,
        world_pos.1.floor() as usize,
        player_pos.z.floor() as usize,
    );

    Prefabs::spawn_world(world, Prefab::new(selected_prefab, spawn_pos));

    let Some(mut clock) = world.get_resource_mut::<Clock>() else {
        return;
    };
    clock.force_update();

    game_loop(world);
}

fn update_mouse_position_display(
    mouse: Res<Mouse>,
    mut q_mouse_display: Query<&mut Text, With<MousePositionDisplay>>,
) {
    let Ok(mut text) = q_mouse_display.single_mut() else {
        return;
    };

    text.value = format!("Mouse: ({:.1}, {:.1})", mouse.world.0, mouse.world.1);
}

fn update_selected_prefab_display(
    list_context: Res<ListContext>,
    mut q_selected_display: Query<&mut Text, With<SelectedPrefabDisplay>>,
) {
    let Ok(mut text) = q_selected_display.single_mut() else {
        return;
    };

    let selected_index = list_context.activated_item_index;
    if selected_index >= SPAWNABLE_PREFABS.len() {
        text.value = "Selected: None".to_string();
        return;
    }

    let selected_prefab = &SPAWNABLE_PREFABS[selected_index];
    text.value = format!("Selected: {}", selected_prefab);
}