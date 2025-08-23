use bevy_ecs::prelude::*;
use macroquad::input::KeyCode;
use serde::{Deserialize, Serialize};

use crate::{
    cfg::{MAP_SIZE, ZONE_SIZE},
    domain::{
        Collider, ConsumeEnergyEvent, EnergyActionType, GameSettings, Prefabs, StairDown, StairUp,
        TurnState, Zone,
    },
    engine::{InputRate, KeyInput, Mouse, SerializableComponent, Time},
    rendering::{Position, Text, zone_xyz},
    states::{CurrentGameState, GameState},
};

use super::{PrefabId, SpawnConfig};

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct Player;

#[derive(Resource, Default, Clone)]
pub struct PlayerPosition {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl PlayerPosition {
    pub fn from_position(pos: &Position) -> Self {
        Self {
            x: pos.x,
            y: pos.y,
            z: pos.z,
        }
    }
}

#[derive(Event)]
pub struct PlayerMovedEvent {
    pub x: usize,
    pub y: usize,
    pub z: usize,
}

pub fn player_input(
    mut cmds: Commands,
    mut q_player: Query<(Entity, &mut Position), With<Player>>,
    q_colliders: Query<&Position, (With<Collider>, Without<Player>)>,
    q_stairs_down: Query<&Position, (With<StairDown>, Without<Player>)>,
    q_stairs_up: Query<&Position, (With<StairUp>, Without<Player>)>,
    keys: Res<KeyInput>,
    time: Res<Time>,
    mut input_rate: Local<InputRate>,
    mut e_player_moved: EventWriter<PlayerMovedEvent>,
    mut e_consume_energy: EventWriter<ConsumeEnergyEvent>,
    mut game_state: ResMut<CurrentGameState>,
    turn_state: Res<TurnState>,
    settings: Res<GameSettings>,
    q_zone: Query<&Zone>,
    prefabs: Res<Prefabs>,
) {
    let now = time.elapsed;
    let rate = settings.input_rate;
    let delay = settings.input_initial_delay;
    let (player_entity, mut position) = q_player.single_mut().unwrap();
    let mut moved = false;
    let (x, y, z) = position.world();

    if keys.is_pressed(KeyCode::Escape) {
        game_state.next = GameState::Pause;
    }

    if keys.is_pressed(KeyCode::G) {
        prefabs.spawn(&mut cmds, SpawnConfig::new(PrefabId::Boulder, (x, y, z)));
    }

    if !turn_state.is_players_turn {
        return;
    }

    if keys.is_down(KeyCode::T) && input_rate.try_key(KeyCode::T, now, rate, delay) {
        e_consume_energy.write(ConsumeEnergyEvent::new(
            player_entity,
            EnergyActionType::Wait,
        ));
        return;
    }

    if x > 0
        && keys.is_down(KeyCode::A)
        && input_rate.try_key(KeyCode::A, now, rate, delay)
        && !has_collider_at((x - 1, y, z), &q_colliders, &q_zone)
    {
        position.x -= 1.;
        moved = true;
    }

    if x < (MAP_SIZE.0 * ZONE_SIZE.0) - 1
        && keys.is_down(KeyCode::D)
        && input_rate.try_key(KeyCode::D, now, rate, delay)
        && !has_collider_at((x + 1, y, z), &q_colliders, &q_zone)
    {
        position.x += 1.;
        moved = true;
    }

    if y > 0
        && keys.is_down(KeyCode::W)
        && input_rate.try_key(KeyCode::W, now, rate, delay)
        && !has_collider_at((x, y - 1, z), &q_colliders, &q_zone)
    {
        position.y -= 1.;
        moved = true;
    }

    if y < (MAP_SIZE.1 * ZONE_SIZE.1) - 1
        && keys.is_down(KeyCode::S)
        && input_rate.try_key(KeyCode::S, now, rate, delay)
        && !has_collider_at((x, y + 1, z), &q_colliders, &q_zone)
    {
        position.y += 1.;
        moved = true;
    }

    if z > 0
        && keys.is_down(KeyCode::E)
        && input_rate.try_key(KeyCode::E, now, rate, delay)
        && is_on_stair_up(x, y, z, &q_stairs_up)
    {
        position.z -= 1.;
        moved = true;
    }

    if z < MAP_SIZE.2 - 1
        && keys.is_down(KeyCode::Q)
        && input_rate.try_key(KeyCode::Q, now, rate, delay)
        && is_on_stair_down(x, y, z, &q_stairs_down)
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

        // Consume energy for movement
        e_consume_energy.write(ConsumeEnergyEvent::new(
            player_entity,
            EnergyActionType::Move,
        ));
    }
}

#[derive(Component)]
pub struct PlayerDebug;

pub fn update_player_position_resource(
    mut e_player_moved: EventReader<PlayerMovedEvent>,
    mut player_pos: ResMut<PlayerPosition>,
) {
    for e in e_player_moved.read() {
        player_pos.x = e.x as f32;
        player_pos.y = e.y as f32;
        player_pos.z = e.z as f32;
    }
}

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

fn has_collider_at(
    world_pos: (usize, usize, usize),
    colliders: &Query<&Position, (With<Collider>, Without<Player>)>,
    q_zones: &Query<&Zone>,
) -> bool {
    Zone::get_at(world_pos, q_zones)
        .iter()
        .any(|e| colliders.contains(*e))
}

fn is_on_stair_down(
    player_x: usize,
    player_y: usize,
    player_z: usize,
    stairs: &Query<&Position, (With<StairDown>, Without<Player>)>,
) -> bool {
    for stair_pos in stairs.iter() {
        if stair_pos.x.floor() as usize == player_x
            && stair_pos.y.floor() as usize == player_y
            && stair_pos.z.floor() as usize == player_z
        {
            return true;
        }
    }
    false
}

fn is_on_stair_up(
    player_x: usize,
    player_y: usize,
    player_z: usize,
    stairs: &Query<&Position, (With<StairUp>, Without<Player>)>,
) -> bool {
    for stair_pos in stairs.iter() {
        if stair_pos.x.floor() as usize == player_x
            && stair_pos.y.floor() as usize == player_y
            && stair_pos.z.floor() as usize == player_z
        {
            return true;
        }
    }
    false
}
