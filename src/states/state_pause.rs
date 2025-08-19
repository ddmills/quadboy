use bevy_ecs::{
    component::Component,
    system::{Commands, Res, ResMut},
};
use macroquad::input::KeyCode;

use crate::{
    engine::{KeyInput, Plugin},
    rendering::{Position, Text},
    states::{CurrentGameState, GameStatePlugin, cleanup_system},
};

use super::GameState;

pub struct PauseStatePlugin;

impl Plugin for PauseStatePlugin {
    fn build(&self, app: &mut crate::engine::App) {
        GameStatePlugin::new(GameState::Pause)
            .on_enter(app, on_enter_pause)
            .on_update(app, listen_for_inputs)
            .on_leave(app, cleanup_system::<CleanupStatePause>);
    }
}

#[derive(Component)]
pub struct CleanupStatePause;

fn on_enter_pause(mut cmds: Commands) {
    cmds.spawn((
        Text::new("PAUSED"),
        Position::new_f32(12., 1., 0.),
        CleanupStatePause,
    ));
}

fn listen_for_inputs(keys: Res<KeyInput>, mut game_state: ResMut<CurrentGameState>) {
    if keys.is_pressed(KeyCode::P) {
        game_state.next = GameState::Explore;
    }
}
