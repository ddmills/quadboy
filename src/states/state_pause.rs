use bevy_ecs::{schedule::IntoScheduleConfigs, system::{Res, ResMut}};
use macroquad::{input::KeyCode, prelude::trace};

use crate::{engine::{KeyInput, Plugin, ScheduleType}, states::{in_game_state, CurrentGameState}};

use super::GameState;

pub struct PauseStatePlugin;

impl Plugin for PauseStatePlugin {
    fn build(&self, app: &mut crate::engine::App) {
        app.add_systems(ScheduleType::Update, listen_for_inputs.run_if(in_game_state(GameState::Pause)));
    }
}

fn listen_for_inputs(keys: Res<KeyInput>, mut game_state: ResMut<CurrentGameState>)
{
    if keys.is_pressed(KeyCode::P) {
        trace!("leave pause");
        game_state.next = GameState::Explore;
    }
}
