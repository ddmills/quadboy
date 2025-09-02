use bevy_ecs::prelude::*;
use macroquad::input::KeyCode;
use serde::{Deserialize, Serialize};

use crate::{
    cfg::{MAP_SIZE, ZONE_SIZE},
    domain::{
        AttackAction, Collider, Energy, GameSettings, Inventory, InventoryAccessible, IsExplored,
        MoveAction, OpenContainerAction, Prefabs, StairDown, StairUp, TurnState, WaitAction, Zone,
    },
    engine::{InputRate, KeyInput, Mouse, SerializableComponent, Time},
    rendering::{Glyph, Position, Text, world_to_zone_idx, world_to_zone_local},
    states::{CurrentGameState, GameState},
};

use super::{Prefab, PrefabId};

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

    pub fn zone_idx(&self) -> usize {
        let (x, y, z) = self.world();
        world_to_zone_idx(x, y, z)
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
    q_player: Query<(Entity, &Position), With<Player>>,
    q_colliders: Query<&Position, (With<Collider>, Without<Player>)>,
    q_containers: Query<Entity, (With<Inventory>, With<InventoryAccessible>)>,
    q_stairs_down: Query<&Position, (With<StairDown>, Without<Player>)>,
    q_stairs_up: Query<&Position, (With<StairUp>, Without<Player>)>,
    keys: Res<KeyInput>,
    time: Res<Time>,
    mut input_rate: Local<InputRate>,
    mut movement_timer: Local<(f64, bool)>, // (last_move_time, past_initial_delay)
    mut game_state: ResMut<CurrentGameState>,
    turn_state: Res<TurnState>,
    settings: Res<GameSettings>,
    q_zone: Query<&Zone>,
    q_unexplored: Query<Entity, Without<IsExplored>>,
) {
    let now = time.fixed_t;
    let mut rate = settings.input_delay;
    let delay = settings.input_initial_delay;
    let (player_entity, position) = q_player.single().unwrap();
    let (x, y, z) = position.world();

    if keys.is_pressed(KeyCode::Escape) {
        game_state.next = GameState::Pause;
    }

    if keys.is_pressed(KeyCode::M) {
        game_state.next = GameState::Overworld;
    }

    if keys.is_pressed(KeyCode::I) {
        game_state.next = GameState::Inventory;
    }

    if keys.is_pressed(KeyCode::O) {
        // Check for adjacent containers
        let neighbors = Zone::get_neighbors((x, y, z), &q_zone);

        for entities in neighbors {
            for entity in entities {
                // Check if this entity is a container (has Inventory and InventoryAccessible)
                if q_containers.contains(entity) {
                    cmds.queue(OpenContainerAction {
                        player_entity,
                        container_entity: entity,
                    });
                    return;
                }
            }
        }
    }

    if keys.is_pressed(KeyCode::G) {
        Prefabs::spawn(&mut cmds, Prefab::new(PrefabId::Boulder, (x, y, z)));
    }

    if keys.is_pressed(KeyCode::L) {
        Prefabs::spawn(&mut cmds, Prefab::new(PrefabId::Lantern, (x, y, z)));
    }

    if keys.is_pressed(KeyCode::P) {
        Prefabs::spawn(&mut cmds, Prefab::new(PrefabId::Pickaxe, (x, y, z)));
    }

    if keys.is_pressed(KeyCode::H) {
        Prefabs::spawn(&mut cmds, Prefab::new(PrefabId::Hatchet, (x, y, z)));
    }

    if keys.is_pressed(KeyCode::C) {
        Prefabs::spawn(&mut cmds, Prefab::new(PrefabId::Chest, (x, y, z)));
    }

    if keys.is_pressed(KeyCode::Y) {
        Prefabs::spawn(&mut cmds, Prefab::new(PrefabId::CavalrySword, (x, y, z)));
    }

    if keys.is_down(KeyCode::LeftShift) {
        rate /= 2.0;
    }

    if keys.is_pressed(KeyCode::V) {
        let zone_idx = world_to_zone_idx(x, y, z);
        for zone in q_zone.iter() {
            if zone.idx == zone_idx {
                for entity_vec in zone.entities.iter() {
                    for &entity in entity_vec.iter() {
                        if q_unexplored.contains(entity) {
                            cmds.entity(entity).insert(IsExplored);
                        }
                    }
                }
            }
        }
    }

    if !turn_state.is_players_turn {
        return;
    }

    if keys.is_down(KeyCode::T) && input_rate.try_key(KeyCode::T, now, rate, delay) {
        cmds.queue(WaitAction {
            entity: player_entity,
        });
        return;
    }

    let movement_keys_down = keys.is_down(KeyCode::A)
        || keys.is_down(KeyCode::D)
        || keys.is_down(KeyCode::W)
        || keys.is_down(KeyCode::S)
        || keys.is_down(KeyCode::Q)
        || keys.is_down(KeyCode::E);

    let movement_keys_pressed = keys.is_pressed(KeyCode::A)
        || keys.is_pressed(KeyCode::D)
        || keys.is_pressed(KeyCode::W)
        || keys.is_pressed(KeyCode::S)
        || keys.is_pressed(KeyCode::Q)
        || keys.is_pressed(KeyCode::E);

    if !movement_keys_down {
        movement_timer.1 = false;
    }

    let can_move = if movement_keys_pressed {
        true
    } else if movement_keys_down {
        if movement_timer.1 {
            now - movement_timer.0 >= rate
        } else if now - movement_timer.0 >= delay {
            movement_timer.1 = true;
            true
        } else {
            false
        }
    } else {
        false
    };

    if movement_keys_down && can_move {
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

        if z > 0 && keys.is_down(KeyCode::Q) && is_on_stair_up(x, y, z, &q_stairs_up) {
            dz -= 1;
        }

        if z < MAP_SIZE.2 - 1
            && keys.is_down(KeyCode::Q)
            && is_on_stair_down(x, y, z, &q_stairs_down)
        {
            dz += 1;
        }

        if dx != 0 || dy != 0 || dz != 0 {
            let new_x = (x as i32 + dx) as usize;
            let new_y = (y as i32 + dy) as usize;
            let new_z = (z as i32 + dz) as usize;

            let crosses_boundary = would_cross_zone_boundary((x, y, z), (new_x, new_y, new_z));
            let zone_boundary_override =
                crosses_boundary && settings.zone_boundary_move_delay > 0.0;

            let can_move_now = if movement_keys_pressed && !zone_boundary_override {
                true
            } else if zone_boundary_override {
                now - movement_timer.0 >= settings.zone_boundary_move_delay
            } else {
                true
            };

            if can_move_now
                && new_x < MAP_SIZE.0 * ZONE_SIZE.0
                && new_y < MAP_SIZE.1 * ZONE_SIZE.1
                && new_z < MAP_SIZE.2
            {
                let can_move_vertically = dz == 0
                    || (dz < 0 && is_on_stair_up(new_x, new_y, z, &q_stairs_up))
                    || (dz > 0 && is_on_stair_down(new_x, new_y, z, &q_stairs_down));

                if can_move_vertically {
                    if has_collider_at((new_x, new_y, new_z), &q_colliders, &q_zone) {
                        // Bump attack - try to attack what we bumped into
                        cmds.queue(AttackAction {
                            attacker_entity: player_entity,
                            target_pos: (new_x, new_y, new_z),
                        });
                        movement_timer.0 = now;
                    } else {
                        // Normal movement
                        cmds.queue(MoveAction {
                            entity: player_entity,
                            new_position: (new_x, new_y, new_z),
                        });
                        movement_timer.0 = now;
                    }
                }
            }
        }
    }

    for key in keys.released.iter() {
        input_rate.keys.remove(key);
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
    q_zones: Query<&Zone>,
    cursor: Res<Mouse>,
) {
    let Ok(position) = q_player.single() else {
        return;
    };
    let Ok(mut debug) = q_debug.single_mut() else {
        return;
    };
    let zone_idx = position.zone_idx();

    let (cursor_x, cursor_y) = (
        cursor.world.0.floor() as usize,
        cursor.world.1.floor() as usize,
    );

    // Get terrain at player position
    let terrain_str = if let Some(zone) = q_zones.iter().find(|z| z.idx == zone_idx) {
        let (local_x, local_y) = world_to_zone_local(cursor_x, cursor_y);
        if let Some(terrain) = zone.terrain.get(local_x, local_y) {
            terrain.label_formatted()
        } else {
            "Unknown".to_string()
        }
    } else {
        "No Zone".to_string()
    };

    debug.value = format!(
        "MOUSE={{C|{}}},{{C|{}}} ZONE_IDX={{C|{}}} TERRAIN={} GLYPHS={{C|{}}} ACTORS={{C|{}}}",
        cursor_x,
        cursor_y,
        zone_idx,
        terrain_str,
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
