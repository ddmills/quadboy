use bevy_ecs::{prelude::*, system::SystemId};
use macroquad::prelude::trace;
use serde::{Deserialize, Serialize};

use crate::{
    common::Palette,
    domain::{
        Health, Level, Player, PlayerDebug, PlayerMovedEvent, StatType, Stats,
        collect_valid_targets, game_loop, handle_item_pickup, init_targeting_resource,
        player_input, render_player_debug, render_target_crosshair, render_target_info,
        spawn_targeting_ui, update_mouse_targeting, update_target_cycling,
    },
    engine::{App, Plugin, SerializableComponent},
    rendering::{Layer, Position, Text},
    states::{CurrentGameState, GameStatePlugin, cleanup_system},
    ui::{
        Button, XPProgressBar, display_entity_names_at_mouse, render_cursor, render_lighting_debug,
        render_tick_display, spawn_debug_ui_entities, update_xp_progress_bars,
    },
};

use super::GameState;

#[derive(Resource)]
struct ExploreCallbacks {
    open_map: SystemId,
    open_inventory: SystemId,
    open_debug_spawn: SystemId,
    open_attributes: SystemId,
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
                    update_xp_progress_bars,
                    update_player_hp_bar,
                    update_player_armor_bar,
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

#[derive(Component)]
pub struct PlayerHPBar;

#[derive(Component)]
pub struct PlayerArmorBar;

fn setup_callbacks(world: &mut World) {
    let callbacks = ExploreCallbacks {
        open_map: world.register_system(open_map),
        open_inventory: world.register_system(open_inventory),
        open_debug_spawn: world.register_system(open_debug_spawn),
        open_attributes: world.register_system(open_attributes),
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

fn open_attributes(mut game_state: ResMut<CurrentGameState>) {
    game_state.next = GameState::Attributes;
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
        Text::new("123").bg(Palette::Black),
        Position::new_f32(6., 0., 0.),
        PlayerDebug,
        CleanupStateExplore,
    ));

    // Spawn UI buttons
    spawn_ui_buttons(&mut cmds, &callbacks);

    // Spawn XP progress bar
    cmds.spawn((
        Text::new("").layer(Layer::Ui),
        Position::new_f32(30., 1.5, 0.),
        XPProgressBar::new(30),
        CleanupStateExplore,
    ));

    // Spawn player HP display
    cmds.spawn((
        Text::new("HP").fg1(Palette::White).layer(Layer::Ui),
        Position::new_f32(1., 3., 0.),
        PlayerHPBar,
        CleanupStateExplore,
    ));

    // Spawn player armor display
    cmds.spawn((
        Text::new("").fg1(Palette::White).layer(Layer::Ui),
        Position::new_f32(1., 3.5, 0.),
        PlayerArmorBar,
        CleanupStateExplore,
    ));
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

fn update_player_hp_bar(
    q_player: Query<
        (&Health, &Level, &Stats),
        (With<Player>, Or<(Changed<Health>, Changed<Stats>)>),
    >,
    mut q_hp_display: Query<&mut Text, With<PlayerHPBar>>,
) {
    let Ok((health, level, stats)) = q_player.single() else {
        return;
    };
    let Ok(mut hp_text) = q_hp_display.single_mut() else {
        return;
    };

    let max_hp = Health::get_max_hp(level, stats);
    hp_text.value = format!("HP: {}/{}", health.current, max_hp);
}

fn update_player_armor_bar(
    q_player: Query<(&Health, &Stats), (With<Player>, Or<(Changed<Health>, Changed<Stats>)>)>,
    mut q_armor_display: Query<&mut Text, With<PlayerArmorBar>>,
) {
    let Ok((health, stats)) = q_player.single() else {
        return;
    };
    let Ok(mut armor_text) = q_armor_display.single_mut() else {
        return;
    };

    let (current_armor, max_armor) = health.get_current_max_armor(stats);
    armor_text.value = format!("Armor: {}/{}", current_armor, max_armor);
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

    cmds.spawn((
        Position::new_f32(22., 1.5, 0.),
        Button::new("({Y|Y}) ATTRIBUTES", callbacks.open_attributes)
            .hotkey(macroquad::input::KeyCode::Y),
        CleanupStateExplore,
    ));
}
