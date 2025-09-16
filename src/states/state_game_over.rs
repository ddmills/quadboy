use bevy_ecs::{
    component::Component,
    prelude::*,
    schedule::IntoScheduleConfigs,
    system::{Commands, Res, ResMut, SystemId},
};
use macroquad::{input::KeyCode, prelude::trace};

use crate::{
    common::Palette,
    domain::GameSettings,
    engine::Plugin,
    rendering::{Position, Text},
    states::{AppState, CurrentAppState, CurrentGameState, GameStatePlugin, cleanup_system},
    ui::{List, ListItemData},
};

use super::GameState;

#[derive(Resource)]
struct GameOverCallbacks {
    load_last_save: SystemId,
    quit_to_menu: SystemId,
}

pub struct GameOverStatePlugin;

impl Plugin for GameOverStatePlugin {
    fn build(&self, app: &mut crate::engine::App) {
        GameStatePlugin::new(GameState::GameOver)
            .on_enter(app, (setup_callbacks, on_enter_game_over).chain())
            .on_leave(
                app,
                (
                    on_leave_game_over,
                    cleanup_system::<CleanupStateGameOver>,
                    remove_game_over_callbacks,
                )
                    .chain(),
            );
    }
}

fn on_leave_game_over() {
    trace!("LeaveGameState::<GameOver>");
}

#[derive(Component)]
pub struct CleanupStateGameOver;

fn setup_callbacks(world: &mut World) {
    let callbacks = GameOverCallbacks {
        load_last_save: world.register_system(load_last_save),
        quit_to_menu: world.register_system(quit_to_menu),
    };

    world.insert_resource(callbacks);
}

fn load_last_save(mut game_state: ResMut<CurrentGameState>, settings: Res<GameSettings>) {
    if settings.enable_saves {
        game_state.next = GameState::LoadGame;
    }
}

fn quit_to_menu(mut app_state: ResMut<CurrentAppState>, mut game_state: ResMut<CurrentGameState>) {
    game_state.next = GameState::None;
    app_state.next = AppState::MainMenu;
}

fn remove_game_over_callbacks(mut cmds: Commands) {
    cmds.remove_resource::<GameOverCallbacks>();
}

fn on_enter_game_over(
    mut cmds: Commands,
    callbacks: Res<GameOverCallbacks>,
    settings: Res<GameSettings>,
) {
    cmds.spawn((
        Text::new("{R|GAME OVER}").bg(Palette::Black),
        Position::new_f32(4., 2., 0.),
        CleanupStateGameOver,
    ));

    let mut options = vec![];

    if settings.enable_saves {
        options.push(
            ListItemData::new("({Y|L}) LOAD LAST SAVE", callbacks.load_last_save)
                .with_hotkey(KeyCode::L),
        );
    }

    options.push(
        ListItemData::new("({R|Q}) QUIT TO MAIN MENU", callbacks.quit_to_menu)
            .with_hotkey(KeyCode::Q),
    );

    cmds.spawn((
        List::new(options).with_focus_order(1000),
        Position::new_f32(4., 4., 0.),
        CleanupStateGameOver,
    ));
}
