use bevy_ecs::system::{Res, ResMut};
use macroquad::{input::KeyCode, prelude::trace};

use crate::{
    engine::{KeyInput, Plugin},
    states::{CurrentGameState, GameStatePlugin},
};

use super::GameState;

pub struct PauseStatePlugin;

impl Plugin for PauseStatePlugin {
    fn build(&self, app: &mut crate::engine::App) {
        GameStatePlugin::new(GameState::Pause).on_update(app, listen_for_inputs);
    }
}

fn listen_for_inputs(keys: Res<KeyInput>, mut game_state: ResMut<CurrentGameState>) {
    if keys.is_pressed(KeyCode::P) {
        trace!("leave pause");
        game_state.next = GameState::Explore;
    }
}
