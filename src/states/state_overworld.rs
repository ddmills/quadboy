use bevy_ecs::prelude::*;
use macroquad::input::KeyCode;

use crate::{
    cfg::{MAP_SIZE, SURFACE_LEVEL_Z},
    common::Palette,
    domain::{Overworld, PlayerPosition, ZoneType},
    engine::{KeyInput, Plugin},
    rendering::{Glyph, Layer, Position, Text, world_to_zone_idx, zone_idx, zone_xyz},
    states::{CurrentGameState, GameStatePlugin, cleanup_system},
};

use super::GameState;

pub struct OverworldStatePlugin;

impl Plugin for OverworldStatePlugin {
    fn build(&self, app: &mut crate::engine::App) {
        GameStatePlugin::new(GameState::Overworld)
            .on_enter(app, (on_enter_overworld, render_overworld_map).chain())
            .on_update(app, listen_for_inputs)
            .on_leave(app, cleanup_system::<CleanupStateOverworld>);
    }
}

#[derive(Component)]
pub struct CleanupStateOverworld;

#[derive(Component)]
pub struct OverworldMapTile;

fn on_enter_overworld(mut cmds: Commands) {
    cmds.spawn((
        Text::new("OVERWORLD MAP").bg(Palette::Black),
        Position::new_f32(2., 1., 0.),
        CleanupStateOverworld,
    ));

    cmds.spawn((
        Text::new("({Y|ESC}) BACK TO EXPLORE").bg(Palette::Black),
        Position::new_f32(2., MAP_SIZE.1 as f32 + 4., 0.),
        CleanupStateOverworld,
    ));
}

fn render_overworld_map(
    mut cmds: Commands,
    mut overworld: ResMut<Overworld>,
    player_pos: Res<PlayerPosition>,
) {
    let map_start_x = 2.0;
    let map_start_y = 3.0;

    let player_world = player_pos.world();
    let player_zone_idx = world_to_zone_idx(player_world.0, player_world.1, player_world.2);
    let player_zone_pos = zone_xyz(player_zone_idx);

    for x in 0..MAP_SIZE.0 {
        for y in 0..MAP_SIZE.1 {
            let idx = zone_idx(x, y, SURFACE_LEVEL_Z);
            let ozone = overworld.get_overworld_zone(idx);

            let (glyph_char, fg1) = match ozone.zone_type {
                ZoneType::OpenAir => (16, Palette::Blue),
                ZoneType::Forest => (1, Palette::Green),
                ZoneType::Desert => (33, Palette::Yellow),
                ZoneType::Cavern => (129, Palette::Gray),
            };

            let is_player_zone = x == player_zone_pos.0
                && y == player_zone_pos.1
                && SURFACE_LEVEL_Z == player_zone_pos.2;
            let bg_color = if is_player_zone {
                Palette::Red
            } else {
                Palette::Black
            };

            cmds.spawn((
                Glyph::new(glyph_char, fg1, fg1)
                    .layer(Layer::Ui)
                    .bg(bg_color),
                Position::new_f32(map_start_x + x as f32, map_start_y + y as f32, 0.),
                OverworldMapTile,
                CleanupStateOverworld,
            ));
        }
    }
}

fn listen_for_inputs(keys: Res<KeyInput>, mut game_state: ResMut<CurrentGameState>) {
    if keys.is_pressed(KeyCode::Escape) {
        game_state.next = GameState::Explore;
    }
}
