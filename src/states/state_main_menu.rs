use bevy_ecs::prelude::*;
use macroquad::{input::KeyCode, prelude::trace};

use crate::{
    engine::{App, ExitAppEvent, KeyInput, Plugin},
    rendering::{Position, Text},
    states::{
        AppState, AppStatePlugin, CurrentAppState, CurrentGameState, GameState, cleanup_system,
    },
};

pub struct MainMenuStatePlugin;

impl Plugin for MainMenuStatePlugin {
    fn build(&self, app: &mut App) {
        AppStatePlugin::new(AppState::MainMenu)
            .on_enter(app, render_menu)
            .on_update(app, main_menu_input)
            .on_leave(app, cleanup_system::<CleanupMainMenu>);
    }
}

#[derive(Component)]
struct CleanupMainMenu;

fn render_menu(mut cmds: Commands) {
    trace!("EnterAppState::<MainMenu>");

    cmds.spawn((
        Text::new("Welcome to..."),
        Position::new_f32(4., 2., 0.),
        CleanupMainMenu,
    ));

    cmds.spawn((
        Text::new("a cowboy adventure. With a {W-Y-W-C-C-C-C-C-C-C-C-C-C-C-C-C-C-C-C-C-C-C-C-C-C-C-C-C-C-C-C-C scrollf|shiny revolver}"),
        Position::new_f32(4., 2.5, 0.),
        CleanupMainMenu,
    ));

    cmds.spawn((
        Text::new("({Y|N}) NEW GAME"),
        Position::new_f32(4., 4., 0.),
        CleanupMainMenu,
    ));

    cmds.spawn((
        Text::new("({Y|L}) LOAD"),
        Position::new_f32(4., 4.5, 0.),
        CleanupMainMenu,
    ));

    cmds.spawn((
        Text::new("({Y|S}) SETTINGS"),
        Position::new_f32(4., 5., 0.),
        CleanupMainMenu,
    ));

    cmds.spawn((
        Text::new("({R|Q}) QUIT"),
        Position::new_f32(4., 6., 0.),
        CleanupMainMenu,
    ));
}

fn main_menu_input(
    keys: Res<KeyInput>,
    mut app_state: ResMut<CurrentAppState>,
    mut game_state: ResMut<CurrentGameState>,
    mut e_exit_app: EventWriter<ExitAppEvent>,
) {
    if keys.is_pressed(KeyCode::N) {
        app_state.next = AppState::Play;
        game_state.next = GameState::NewGame;
    }

    if keys.is_pressed(KeyCode::L) {
        app_state.next = AppState::Play;
        game_state.next = GameState::LoadGame;
    }

    if keys.is_pressed(KeyCode::S) {
        app_state.next = AppState::Settings;
    }

    if keys.is_pressed(KeyCode::Q) {
        e_exit_app.write(ExitAppEvent);
    }
}
