use bevy_ecs::prelude::*;
use macroquad::input::KeyCode;

use crate::{ecs::Time, engine::{InputRate, KeyInput}, rendering::Position};

#[derive(Component)]
pub struct Player;

pub fn player_input(
    mut q_player: Query<&mut Position, With<Player>>,
    keys: Res<KeyInput>,
    time: Res<Time>,
    mut input_rate: Local<InputRate>,
) {
    let now = time.elapsed;
    let rate = 0.025;
    let delay = 0.25;
    let mut position = q_player.single_mut().unwrap();

    if keys.is_down(KeyCode::A) && input_rate.try_key(KeyCode::A, now, rate, delay) {
        position.x -= 1.;
    }

    if keys.is_down(KeyCode::D) && input_rate.try_key(KeyCode::D, now, rate, delay) {
        position.x += 1.;
    }

    if keys.is_down(KeyCode::W) && input_rate.try_key(KeyCode::W, now, rate, delay) {
        position.y -= 1.;
    }

    if keys.is_down(KeyCode::S) && input_rate.try_key(KeyCode::S, now, rate, delay) {
        position.y += 1.;
    }

    for key in keys.released.iter() {
        input_rate.keys.remove(key);
    }
}
