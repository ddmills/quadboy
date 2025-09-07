use bevy_ecs::{prelude::*, system::SystemId};
use macroquad::{input::KeyCode, prelude::trace};

use crate::{
    engine::{App, AudioKey, ExitAppEvent, Plugin},
    rendering::{Position, Text},
    states::{
        AppState, AppStatePlugin, CurrentAppState, CurrentGameState, GameState, cleanup_system,
    },
    ui::{List, ListItemData},
};

#[derive(Resource)]
struct MainMenuCallbacks {
    on_btn_new_game: SystemId,
    on_btn_load: SystemId,
    on_btn_settings: SystemId,
    on_btn_quit: SystemId,
}

pub struct MainMenuStatePlugin;

impl Plugin for MainMenuStatePlugin {
    fn build(&self, app: &mut App) {
        AppStatePlugin::new(AppState::MainMenu)
            .on_enter(app, (setup_callbacks, render_menu).chain())
            .on_leave(app, cleanup_system::<CleanupMainMenu>);
    }
}

fn setup_callbacks(world: &mut World) {
    let callbacks = MainMenuCallbacks {
        on_btn_new_game: world.register_system(on_btn_new_game),
        on_btn_load: world.register_system(on_btn_load),
        on_btn_quit: world.register_system(on_btn_quit),
        on_btn_settings: world.register_system(on_btn_settings),
    };

    world.insert_resource(callbacks);
}

fn on_btn_new_game(
    mut app_state: ResMut<CurrentAppState>,
    mut game_state: ResMut<CurrentGameState>,
) {
    app_state.next = AppState::Play;
    game_state.next = GameState::NewGame;
}

fn on_btn_settings(mut app_state: ResMut<CurrentAppState>) {
    app_state.next = AppState::Settings;
}

fn on_btn_quit(mut e_exit_app: EventWriter<ExitAppEvent>) {
    e_exit_app.write(ExitAppEvent);
}

fn on_btn_load(mut app_state: ResMut<CurrentAppState>, mut game_state: ResMut<CurrentGameState>) {
    app_state.next = AppState::Play;
    game_state.next = GameState::LoadGame;
}

#[derive(Component)]
struct CleanupMainMenu;

fn render_menu(mut cmds: Commands, callbacks: Res<MainMenuCallbacks>) {
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
        List::new(vec![
            ListItemData::new("({Y|N}) NEW GAME", callbacks.on_btn_new_game)
                .with_hotkey(KeyCode::N),
            ListItemData::new("({Y|L}) LOAD", callbacks.on_btn_load).with_hotkey(KeyCode::L),
            ListItemData::new("({Y|S}) SETTINGS", callbacks.on_btn_settings)
                .with_hotkey(KeyCode::S),
            ListItemData::new("({Y|ESC}) {R|QUIT}", callbacks.on_btn_quit)
                .with_hotkey(KeyCode::Escape)
                .with_audio(AudioKey::ButtonBack1),
        ])
        .with_focus_order(1000),
        Position::new_f32(4.0, 4.0, 0.),
        CleanupMainMenu,
    ));
}
