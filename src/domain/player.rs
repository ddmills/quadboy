use bevy_ecs::prelude::*;
use macroquad::input::KeyCode;
use serde::{Deserialize, Serialize};

use crate::{
    cfg::{MAP_SIZE, ZONE_SIZE},
    domain::{
        Collider, ConsumeEnergyEvent, Energy, EnergyActionType, GameSettings, IsExplored, Prefabs,
        StairDown, StairUp, TurnState, Zone,
    },
    engine::{InputRate, KeyInput, Mouse, SerializableComponent, Time},
    rendering::{Glyph, Position, Text, world_to_zone_idx},
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

    pub fn world(&self) -> (usize, usize, usize) {
        (
            self.x.floor() as usize,
            self.y.floor() as usize,
            self.z.floor() as usize,
        )
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
    mut movement_timer: Local<(f64, bool)>, // (last_move_time, past_initial_delay)
    mut e_player_moved: EventWriter<PlayerMovedEvent>,
    mut e_consume_energy: EventWriter<ConsumeEnergyEvent>,
    mut game_state: ResMut<CurrentGameState>,
    turn_state: Res<TurnState>,
    settings: Res<GameSettings>,
    q_zone: Query<&Zone>,
    q_unexplored: Query<Entity, Without<IsExplored>>,
) {
    let now = time.fixed_t;
    let rate = settings.input_rate;
    let delay = settings.input_initial_delay;
    let (player_entity, mut position) = q_player.single_mut().unwrap();
    let mut moved = false;
    let (x, y, z) = position.world();

    if keys.is_pressed(KeyCode::Escape) {
        game_state.next = GameState::Pause;
    }

    if keys.is_pressed(KeyCode::M) {
        game_state.next = GameState::Overworld;
    }

    if keys.is_pressed(KeyCode::G) {
        Prefabs::spawn(&mut cmds, SpawnConfig::new(PrefabId::Boulder, (x, y, z)));
    }

    if keys.is_pressed(KeyCode::V) {
        // Mark all entities in current zone as explored (doesn't consume energy)
        let zone_idx = world_to_zone_idx(x, y, z);
        for zone in q_zone.iter() {
            if zone.idx == zone_idx {
                // Iterate through all cells in the zone's entity grid
                for entity_vec in zone.entities.iter() {
                    // Each cell can have multiple entities
                    for &entity in entity_vec.iter() {
                        // Only add IsExplored if the entity doesn't already have it
                        if q_unexplored.contains(entity) {
                            cmds.entity(entity).insert(IsExplored);
                        }
                    }
                }
            }
        }
        // Don't return here - this action doesn't consume energy or end the turn
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

    // Check if any movement key is pressed or held
    let movement_keys_down = keys.is_down(KeyCode::A)
        || keys.is_down(KeyCode::D)
        || keys.is_down(KeyCode::W)
        || keys.is_down(KeyCode::S)
        || keys.is_down(KeyCode::Q)
        || keys.is_down(KeyCode::E);

    // Check if any movement key was just pressed (not held)
    let movement_keys_pressed = keys.is_pressed(KeyCode::A)
        || keys.is_pressed(KeyCode::D)
        || keys.is_pressed(KeyCode::W)
        || keys.is_pressed(KeyCode::S)
        || keys.is_pressed(KeyCode::Q)
        || keys.is_pressed(KeyCode::E);

    if !movement_keys_down {
        // Reset movement timer when no keys are held
        movement_timer.1 = false;
    }

    // Determine if we can move:
    // - Always move immediately on fresh key press
    // - Otherwise check timing based on whether we're in initial delay or repeat phase
    let can_move = if movement_keys_pressed {
        true // Immediate movement on key press
    } else if movement_keys_down {
        if movement_timer.1 {
            // Already past initial delay, use repeat rate
            now - movement_timer.0 >= rate
        } else {
            // Still in initial delay period
            if now - movement_timer.0 >= delay {
                // Initial delay has passed, switch to repeat rate
                movement_timer.1 = true;
                true
            } else {
                false
            }
        }
    } else {
        false
    };

    if movement_keys_down && can_move {
        // Collect intended movement deltas
        let mut dx: i32 = 0;
        let mut dy: i32 = 0;
        let mut dz: i32 = 0;

        if x > 0 && keys.is_down(KeyCode::A) {
            dx -= 1;
        }

        if x < (MAP_SIZE.0 * ZONE_SIZE.0) - 1 && keys.is_down(KeyCode::D) {
            dx += 1;
        }

        if y > 0 && keys.is_down(KeyCode::W) {
            dy -= 1;
        }

        if y < (MAP_SIZE.1 * ZONE_SIZE.1) - 1 && keys.is_down(KeyCode::S) {
            dy += 1;
        }

        if z > 0 && keys.is_down(KeyCode::E) && is_on_stair_up(x, y, z, &q_stairs_up) {
            dz -= 1;
        }

        if z < MAP_SIZE.2 - 1
            && keys.is_down(KeyCode::Q)
            && is_on_stair_down(x, y, z, &q_stairs_down)
        {
            dz += 1;
        }

        // Check final destination for collisions
        if dx != 0 || dy != 0 || dz != 0 {
            let new_x = (x as i32 + dx) as usize;
            let new_y = (y as i32 + dy) as usize;
            let new_z = (z as i32 + dz) as usize;

            // Check if crossing zone boundary for different rate
            let crosses_boundary = would_cross_zone_boundary((x, y, z), (new_x, new_y, new_z));
            // For zone boundary crossing, override the normal timing
            let zone_boundary_override =
                crosses_boundary && settings.zone_boundary_move_delay > 0.0;

            // Check timing: immediate on press, or respect delays for held keys
            let can_move_now = if movement_keys_pressed && !zone_boundary_override {
                true // Immediate on press (unless crossing zone boundary)
            } else if zone_boundary_override {
                now - movement_timer.0 >= settings.zone_boundary_move_delay
            } else {
                // Already checked timing in can_move above
                true
            };

            if can_move_now {
                // Check bounds
                if new_x < MAP_SIZE.0 * ZONE_SIZE.0
                    && new_y < MAP_SIZE.1 * ZONE_SIZE.1
                    && new_z < MAP_SIZE.2
                {
                    // For vertical movement, also check if we're still on the appropriate stair
                    let can_move_vertically = dz == 0
                        || (dz < 0 && is_on_stair_up(new_x, new_y, z, &q_stairs_up))
                        || (dz > 0 && is_on_stair_down(new_x, new_y, z, &q_stairs_down));

                    if can_move_vertically
                        && !has_collider_at((new_x, new_y, new_z), &q_colliders, &q_zone)
                    {
                        position.x = new_x as f32;
                        position.y = new_y as f32;
                        position.z = new_z as f32;
                        moved = true;

                        // Update movement timer
                        movement_timer.0 = now;
                        // Don't set past_initial_delay to true here - let the timing logic handle it
                    }
                }
            }
        }
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
    q_glyphs: Query<&Glyph>,
    q_energy: Query<&Energy>,
    cursor: Res<Mouse>,
) {
    let Ok(position) = q_player.single() else {
        return;
    };
    let Ok(mut debug) = q_debug.single_mut() else {
        return;
    };
    let zone_idx = position.zone_idx();

    debug.value = format!(
        "MOUSE={{C|{}}},{{C|{}}} ZONE_IDX={{C|{}}} GLYPHS={{C|{}}} ACTORS={{C|{}}}",
        cursor.world.0.floor(),
        cursor.world.1.floor(),
        zone_idx,
        q_glyphs.iter().len(),
        q_energy.iter().len(),
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

fn would_cross_zone_boundary(
    from_pos: (usize, usize, usize),
    to_pos: (usize, usize, usize),
) -> bool {
    let from_zone = world_to_zone_idx(from_pos.0, from_pos.1, from_pos.2);
    let to_zone = world_to_zone_idx(to_pos.0, to_pos.1, to_pos.2);
    from_zone != to_zone
}
