use bevy_ecs::{
    component::Component,
    event::EventReader,
    system::{Commands, Res, ResMut},
};

use crate::{
    domain::{GameSettings, NewGameCommand, NewGameResult},
    engine::{App, Plugin},
    rendering::{Position, Text},
    states::{CurrentGameState, GameState, GameStatePlugin, cleanup_system},
};

pub struct NewGameStatePlugin;

impl Plugin for NewGameStatePlugin {
    fn build(&self, app: &mut App) {
        GameStatePlugin::new(GameState::NewGame)
            .on_enter(app, on_enter_new_game)
            .on_update(app, handle_new_game_result)
            .on_leave(app, cleanup_system::<CleanupStateNewGame>);
    }
}

#[derive(Component)]
pub struct CleanupStateNewGame;

fn on_enter_new_game(mut cmds: Commands, settings: Res<GameSettings>) {
    cmds.queue(NewGameCommand {
        save_name: settings.save_name.clone(),
    });

    cmds.spawn((
        Text::new("Starting new game..."),
        Position::new_f32(12., 10., 0.),
        CleanupStateNewGame,
    ));
}

fn handle_new_game_result(
    mut e_new_game_result: EventReader<NewGameResult>,
    mut game_state: ResMut<CurrentGameState>,
) {
    for result in e_new_game_result.read() {
        if result.success {
            game_state.next = GameState::Explore;
        } else {
            game_state.next = GameState::None;
        }
    }
}
