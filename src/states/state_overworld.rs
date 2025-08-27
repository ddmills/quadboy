use bevy_ecs::prelude::*;
use macroquad::input::KeyCode;

use crate::{
    cfg::{MAP_SIZE, SURFACE_LEVEL_Z},
    common::Palette,
    domain::{BiomeType, Overworld, PlayerPosition},
    engine::{KeyInput, Mouse, Plugin},
    rendering::{Glyph, Layer, Position, Text, Visibility, world_to_zone_idx, zone_idx, zone_xyz},
    states::{CurrentGameState, GameStatePlugin, cleanup_system},
};

use super::GameState;

pub struct OverworldStatePlugin;

impl Plugin for OverworldStatePlugin {
    fn build(&self, app: &mut crate::engine::App) {
        GameStatePlugin::new(GameState::Overworld)
            .on_enter(app, (on_enter_overworld, render_overworld_map).chain())
            .on_update(app, (listen_for_inputs, display_overworld_debug_at_mouse))
            .on_leave(app, cleanup_system::<CleanupStateOverworld>);
    }
}

#[derive(Component)]
pub struct CleanupStateOverworld;

#[derive(Component)]
pub struct OverworldMapTile;

#[derive(Component)]
pub struct OverworldDebugText;

fn on_enter_overworld(mut cmds: Commands) {
    cmds.spawn((
        Text::new("{Y|OVERWORLD MAP}").bg(Palette::Black),
        Position::new_f32(2., 1., 0.),
        CleanupStateOverworld,
    ));

    cmds.spawn((
        Text::new("({Y|M}) BACK TO EXPLORE").bg(Palette::Black),
        Position::new_f32(2., MAP_SIZE.1 as f32 + 3., 0.),
        CleanupStateOverworld,
    ));

    cmds.spawn((
        Text::new("").bg(Palette::Black),
        Position::new_f32(2., 1.5, 0.),
        OverworldDebugText,
        CleanupStateOverworld,
    ));
}

fn render_overworld_map(
    mut cmds: Commands,
    mut overworld: ResMut<Overworld>,
    player_pos: Res<PlayerPosition>,
) {
    let map_start_x = 2.0;
    let map_start_y = 2.0;

    let player_world = player_pos.world();
    let player_zone_idx = world_to_zone_idx(player_world.0, player_world.1, player_world.2);
    let player_zone_pos = zone_xyz(player_zone_idx);

    for x in 0..MAP_SIZE.0 {
        for y in 0..MAP_SIZE.1 {
            let idx = zone_idx(x, y, SURFACE_LEVEL_Z);
            let ozone = overworld.get_overworld_zone(idx);

            let (zone_glyph, zone_fg1) = match ozone.biome_type {
                BiomeType::OpenAir => (16, Palette::Blue),
                BiomeType::Forest => (1, Palette::Green),
                BiomeType::Desert => (33, Palette::Yellow),
                BiomeType::Cavern => (129, Palette::Gray),
            };

            let is_player_zone = x == player_zone_pos.0
                && y == player_zone_pos.1
                && SURFACE_LEVEL_Z == player_zone_pos.2;

            let has_town = ozone.town.is_some();
            let has_road = overworld.road_network.has_road(idx);

            let (glyph, fg1, fg2, bg) = if has_town {
                (
                    8,
                    Palette::Brown,
                    Palette::Yellow,
                    if is_player_zone {
                        Palette::Red
                    } else if has_road {
                        Palette::Brown
                    } else {
                        Palette::Black
                    },
                )
            } else if is_player_zone {
                (zone_glyph, zone_fg1, zone_fg1, Palette::Red)
            } else if has_road {
                (zone_glyph, zone_fg1, zone_fg1, Palette::Brown)
            } else {
                (zone_glyph, zone_fg1, zone_fg1, Palette::Black)
            };

            cmds.spawn((
                Glyph::new(glyph, fg1, fg2).layer(Layer::Ui).bg(bg),
                Position::new_f32(map_start_x + x as f32, map_start_y + y as f32, 0.),
                OverworldMapTile,
                CleanupStateOverworld,
            ));
        }
    }
}

fn listen_for_inputs(keys: Res<KeyInput>, mut game_state: ResMut<CurrentGameState>) {
    if keys.is_pressed(KeyCode::M) {
        game_state.next = GameState::Explore;
    }
}

fn display_overworld_debug_at_mouse(
    mouse: Res<Mouse>,
    mut overworld: ResMut<Overworld>,
    mut q_debug_text: Query<(&mut Text, &mut Visibility), With<OverworldDebugText>>,
) {
    let Ok((mut text, mut visibility)) = q_debug_text.single_mut() else {
        return;
    };

    let map_start_x = 2.0;
    let map_start_y = 2.0;

    // Convert mouse local coordinates to map coordinates
    let mouse_map_x = mouse.ui.0 - map_start_x;
    let mouse_map_y = mouse.ui.1 - map_start_y;

    // Check if mouse is within the map bounds
    if mouse_map_x < 0.0
        || mouse_map_y < 0.0
        || mouse_map_x >= MAP_SIZE.0 as f32
        || mouse_map_y >= MAP_SIZE.1 as f32
    {
        *visibility = Visibility::Hidden;
        return;
    }

    let zone_x = mouse_map_x.floor() as usize;
    let zone_y = mouse_map_y.floor() as usize;
    let zone_idx = zone_idx(zone_x, zone_y, SURFACE_LEVEL_Z);
    let ozone = overworld.get_overworld_zone(zone_idx);

    let has_road = overworld.road_network.has_road(zone_idx);
    let road_connections = if has_road {
        overworld
            .road_network
            .edges
            .iter()
            .filter(|((from, _), _)| *from == zone_idx)
            .count()
    } else {
        0
    };

    let town_value = if let Some(town) = &ozone.town {
        town.name.clone()
    } else {
        "No town".to_string()
    };

    let debug_info = format!(
        "Zone {{C|({}, {})}}\nIndex: {{C|{}}}\nType: {{C|{}}}\nTown: {{C|{}}}\nRoad: {{C|{}}}\nConnections: {{C|{}}}",
        zone_x,
        zone_y,
        zone_idx,
        ozone.biome_type,
        town_value,
        if has_road { "Yes" } else { "No" },
        road_connections
    );

    text.value = debug_info;
    *visibility = Visibility::Visible;
}
