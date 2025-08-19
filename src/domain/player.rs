use bevy_ecs::prelude::*;
use macroquad::input::KeyCode;

use crate::{
    cfg::{INPUT_INITIAL_DELAY, INPUT_RATE, MAP_SIZE, ZONE_SIZE},
    common::Palette,
    engine::{InputRate, KeyInput, Mouse, Time},
    rendering::{Glyph, Position, RenderLayer, Text, TrackZone, zone_xyz},
    states::{AppState, CleanupStatePlay, CurrentAppState, CurrentGameState, GameState},
};

#[derive(Component)]
pub struct Player;

#[derive(Event)]
pub struct PlayerMovedEvent {
    pub x: usize,
    pub y: usize,
    pub z: usize,
}

pub fn player_input(
    mut cmds: Commands,
    mut q_player: Query<&mut Position, With<Player>>,
    keys: Res<KeyInput>,
    time: Res<Time>,
    mut input_rate: Local<InputRate>,
    mut e_player_moved: EventWriter<PlayerMovedEvent>,
    mut app_state: ResMut<CurrentAppState>,
    mut game_state: ResMut<CurrentGameState>,
) {
    let now = time.elapsed;
    let rate = INPUT_RATE;
    let delay = INPUT_INITIAL_DELAY;
    let mut position = q_player.single_mut().unwrap();
    let mut moved = false;
    let (x, y, z) = position.world();

    if keys.is_pressed(KeyCode::Escape) {
        app_state.next = AppState::MainMenu;
    } else if keys.is_pressed(KeyCode::P) {
        game_state.next = GameState::Pause;
    }

    if keys.is_pressed(KeyCode::G) {
        cmds.spawn((
            Position::new(x, y, z),
            Glyph::new(4, Palette::Orange, Palette::Green)
                .layer(RenderLayer::Actors)
                .bg(Palette::White)
                .outline(Palette::Red),
            TrackZone,
            CleanupStatePlay,
        ));
    }

    if x > 0 && keys.is_down(KeyCode::A) && input_rate.try_key(KeyCode::A, now, rate, delay) {
        position.x -= 1.;
        moved = true;
    }

    if x < (MAP_SIZE.0 * ZONE_SIZE.0) - 1
        && keys.is_down(KeyCode::D)
        && input_rate.try_key(KeyCode::D, now, rate, delay)
    {
        position.x += 1.;
        moved = true;
    }

    if y > 0 && keys.is_down(KeyCode::W) && input_rate.try_key(KeyCode::W, now, rate, delay) {
        position.y -= 1.;
        moved = true;
    }

    if y < (MAP_SIZE.1 * ZONE_SIZE.1) - 1
        && keys.is_down(KeyCode::S)
        && input_rate.try_key(KeyCode::S, now, rate, delay)
    {
        position.y += 1.;
        moved = true;
    }

    if z > 0 && keys.is_down(KeyCode::E) && input_rate.try_key(KeyCode::E, now, rate, delay) {
        position.z -= 1.;
        moved = true;
    }

    if z < MAP_SIZE.2 - 1
        && keys.is_down(KeyCode::Q)
        && input_rate.try_key(KeyCode::Q, now, rate, delay)
    {
        position.z += 1.;
        moved = true;
    }

    for key in keys.released.iter() {
        input_rate.keys.remove(key);
    }

    if moved {
        e_player_moved.write(PlayerMovedEvent {
            x: position.x as usize,
            y: position.y as usize,
            z: position.z as usize,
        });
    }
}

#[derive(Component)]
pub struct PlayerDebug;

pub fn render_player_debug(
    q_player: Query<&Position, With<Player>>,
    mut q_debug: Query<&mut Text, With<PlayerDebug>>,
    cursor: Res<Mouse>,
) {
    let position = q_player.single().unwrap();
    let mut debug = q_debug.single_mut().unwrap();
    let zone_idx = position.zone_idx();
    let zone_pos = zone_xyz(zone_idx);

    debug.value = format!(
        "{},{},{} ({},{},{} {{Y|{}}}) [{},{}]",
        position.x,
        position.y,
        position.z,
        zone_pos.0,
        zone_pos.1,
        zone_pos.2,
        zone_idx,
        cursor.world.0.floor(),
        cursor.world.1.floor()
    );
}
