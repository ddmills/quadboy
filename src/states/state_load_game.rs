use bevy_ecs::{
    component::Component,
    event::EventReader,
    system::{Commands, Res, ResMut},
};

use crate::{
    domain::{GameSettings, LoadGameCommand, LoadGameResult},
    engine::{App, Plugin},
    rendering::{Position, Text},
    states::{CurrentGameState, GameState, GameStatePlugin, cleanup_system},
};

pub struct LoadGameStatePlugin;

impl Plugin for LoadGameStatePlugin {
    fn build(&self, app: &mut App) {
        GameStatePlugin::new(GameState::LoadGame)
            .on_enter(app, on_enter_load_game)
            .on_update(app, handle_load_game_result)
            .on_leave(app, cleanup_system::<CleanupStateLoadGame>);
    }
}

#[derive(Component)]
pub struct CleanupStateLoadGame;

fn on_enter_load_game(mut cmds: Commands, settings: Res<GameSettings>) {
    // Queue the LoadGameCommand
    cmds.queue(LoadGameCommand {
        save_name: settings.save_name.clone(),
    });

    // Show loading message
    cmds.spawn((
        Text::new("Loading game..."),
        Position::new_f32(12., 10., 0.),
        CleanupStateLoadGame,
    ));
}

fn handle_load_game_result(
    mut e_load_result: EventReader<LoadGameResult>,
    mut game_state: ResMut<CurrentGameState>,
) {
    for result in e_load_result.read() {
        if result.success {
            // Game loaded successfully, transition to Explore
            game_state.next = GameState::Explore;
        } else {
            // Load failed, show error and return to main menu
            // For now, just transition back to None (which should go to main menu)
            game_state.next = GameState::None;
        }
    }
}
