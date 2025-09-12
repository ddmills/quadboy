use bevy_ecs::{prelude::*, system::SystemId};
use macroquad::prelude::trace;
use serde::{Deserialize, Serialize};

use crate::{
    common::Palette,
    domain::{
        Player, PlayerDebug, PlayerMovedEvent, collect_valid_targets, game_loop,
        handle_item_pickup, init_targeting_resource, player_input, render_player_debug,
        render_target_crosshair, render_target_info, spawn_targeting_ui, update_mouse_targeting,
        update_target_cycling,
    },
    engine::{App, Plugin, SerializableComponent},
    rendering::Position,
    states::{CurrentGameState, GameStatePlugin, cleanup_system},
    ui::{
        Button, display_entity_names_at_mouse, render_cursor, render_lighting_debug,
        render_tick_display, spawn_debug_ui_entities,
    },
};

use super::GameState;

#[derive(Resource)]
struct ExploreCallbacks {
    open_map: SystemId,
    open_inventory: SystemId,
    open_debug_spawn: SystemId,
}

pub struct ExploreStatePlugin;

impl Plugin for ExploreStatePlugin {
    fn build(&self, app: &mut App) {
        GameStatePlugin::new(GameState::Explore)
            .on_enter(
                app,
                (setup_callbacks, on_enter_explore, center_camera_on_player).chain(),
            )
            .on_update(
                app,
                (
                    collect_valid_targets,
                    update_target_cycling,
                    update_mouse_targeting,
                    render_target_crosshair,
                    render_target_info,
                    render_player_debug,
                    render_tick_display,
                    render_lighting_debug,
                    render_cursor,
                    display_entity_names_at_mouse,
                ),
            )
            .on_update(app, player_input)
            .on_update(app, (handle_item_pickup, game_loop))
            .on_leave(
                app,
                (
                    on_leave_explore,
                    cleanup_system::<CleanupStateExplore>,
                    remove_explore_callbacks,
                )
                    .chain(),
            );
    }
}

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct CleanupStateExplore;

fn setup_callbacks(world: &mut World) {
    let callbacks = ExploreCallbacks {
        open_map: world.register_system(open_map),
        open_inventory: world.register_system(open_inventory),
        open_debug_spawn: world.register_system(open_debug_spawn),
    };

    world.insert_resource(callbacks);
}

fn open_map(mut game_state: ResMut<CurrentGameState>) {
    game_state.next = GameState::Overworld;
}

fn open_inventory(mut game_state: ResMut<CurrentGameState>) {
    game_state.next = GameState::Inventory;
}

fn open_debug_spawn(mut game_state: ResMut<CurrentGameState>) {
    game_state.next = GameState::DebugSpawn;
}

fn remove_explore_callbacks(mut cmds: Commands) {
    cmds.remove_resource::<ExploreCallbacks>();
}

fn on_enter_explore(mut cmds: Commands, callbacks: Res<ExploreCallbacks>) {
    trace!("EnterGameState::<Explore>");

    // Initialize targeting system
    init_targeting_resource(&mut cmds);
    spawn_targeting_ui(&mut cmds, CleanupStateExplore);

    // Spawn debug UI elements
    spawn_debug_ui_entities(&mut cmds, CleanupStateExplore);

    // Spawn player debug info
    cmds.spawn((
        crate::rendering::Text::new("123").bg(Palette::Black),
        Position::new_f32(6., 0., 0.),
        PlayerDebug,
        CleanupStateExplore,
    ));

    // Spawn UI buttons
    spawn_ui_buttons(&mut cmds, &callbacks);
}

fn on_leave_explore() {
    trace!("LeaveGameState::<Explore>");
}

fn center_camera_on_player(
    mut e_player_moved: EventWriter<PlayerMovedEvent>,
    q_player: Query<&Position, With<Player>>,
) {
    let p = q_player.single().expect("Expect Player").world();
    e_player_moved.write(PlayerMovedEvent {
        x: p.0,
        y: p.1,
        z: p.2,
    });
}

fn spawn_ui_buttons(cmds: &mut Commands, callbacks: &ExploreCallbacks) {
    cmds.spawn((
        Position::new_f32(3., 1.5, 0.),
        Button::new("({Y|M}) MAP", callbacks.open_map).hotkey(macroquad::input::KeyCode::M),
        CleanupStateExplore,
    ));

    cmds.spawn((
        Position::new_f32(7., 1.5, 0.),
        Button::new("({Y|I}) INVENTORY", callbacks.open_inventory)
            .hotkey(macroquad::input::KeyCode::I),
        CleanupStateExplore,
    ));

    cmds.spawn((
        Position::new_f32(16., 1.5, 0.),
        Button::new("({Y|B}) DEBUG", callbacks.open_debug_spawn)
            .hotkey(macroquad::input::KeyCode::B),
        CleanupStateExplore,
    ));
}
