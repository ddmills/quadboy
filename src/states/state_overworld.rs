use bevy_ecs::{prelude::*, system::SystemId};
use macroquad::{input::KeyCode, prelude::trace};

use crate::{
    cfg::MAP_SIZE,
    common::Palette,
    domain::{BiomeType, Overworld, PlayerPosition},
    engine::{Mouse, Plugin},
    rendering::{Glyph, Layer, Position, Text, Visibility, world_to_zone_idx, zone_idx, zone_xyz},
    states::{CurrentGameState, GameStatePlugin, cleanup_system},
    ui::Button,
};

use super::GameState;

#[derive(Resource)]
struct OverworldCallbacks {
    back_to_explore: SystemId,
}

pub struct OverworldStatePlugin;

impl Plugin for OverworldStatePlugin {
    fn build(&self, app: &mut crate::engine::App) {
        GameStatePlugin::new(GameState::Overworld)
            .on_enter(
                app,
                (setup_callbacks, on_enter_overworld, render_overworld_map).chain(),
            )
            .on_update(app, display_overworld_debug_at_mouse)
            .on_leave(
                app,
                (
                    on_leave_overworld,
                    cleanup_system::<CleanupStateOverworld>,
                    remove_overworld_callbacks,
                )
                    .chain(),
            );
    }
}

fn on_leave_overworld() {
    trace!("LeaveGameState::<Overworld>");
}

#[derive(Component)]
pub struct CleanupStateOverworld;

#[derive(Component)]
pub struct OverworldMapTile;

#[derive(Component)]
pub struct OverworldDebugText;

fn setup_callbacks(world: &mut World) {
    let callbacks = OverworldCallbacks {
        back_to_explore: world.register_system(back_to_explore),
    };

    world.insert_resource(callbacks);
}

fn back_to_explore(mut game_state: ResMut<CurrentGameState>) {
    game_state.next = GameState::Explore;
}

fn remove_overworld_callbacks(mut cmds: Commands) {
    cmds.remove_resource::<OverworldCallbacks>();
}

fn on_enter_overworld(mut cmds: Commands, callbacks: Res<OverworldCallbacks>) {
    cmds.spawn((
        Text::new("{Y|OVERWORLD MAP}").bg(Palette::Black),
        Position::new_f32(2., 1., 0.),
        CleanupStateOverworld,
    ));

    cmds.spawn((
        Text::new("DEBUG").bg(Palette::Black),
        Position::new_f32(2., 1.5, 0.),
        OverworldDebugText,
        CleanupStateOverworld,
    ));

    cmds.spawn((
        Position::new_f32(2., MAP_SIZE.1 as f32 + 3., 0.),
        Button::new("({Y|M}) BACK TO EXPLORE", callbacks.back_to_explore).hotkey(KeyCode::M),
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
            let idx = zone_idx(x, y, player_world.2);
            let ozone = overworld.get_overworld_zone(idx);

            let (zone_glyph, zone_fg1) = match ozone.biome_type {
                BiomeType::OpenAir => (16, Palette::Blue),
                BiomeType::Forest => (1, Palette::Green),
                BiomeType::Desert => (33, Palette::Yellow),
                BiomeType::Cavern => (129, Palette::Gray),
                BiomeType::Mountain => (30, Palette::White),
            };

            let is_player_zone = x == player_zone_pos.0 && y == player_zone_pos.1;

            let has_town = ozone.town.is_some();
            let has_road = overworld.zone_has_road(idx);
            let has_river = overworld.zone_has_river(idx);

            let (glyph, fg1, fg2, bg) = if has_town {
                (
                    8,
                    Palette::Brown,
                    Palette::Yellow,
                    if is_player_zone {
                        Palette::Red
                    } else if has_river {
                        Palette::Blue
                    } else if has_road {
                        Palette::Brown
                    } else {
                        Palette::Black
                    },
                )
            } else if is_player_zone {
                (zone_glyph, zone_fg1, zone_fg1, Palette::Red)
            } else if has_river {
                // Rivers take precedence over roads
                (zone_glyph, zone_fg1, zone_fg1, Palette::Blue)
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

fn display_overworld_debug_at_mouse(
    mouse: Res<Mouse>,
    mut overworld: ResMut<Overworld>,
    mut q_debug_text: Query<(&mut Text, &mut Visibility), With<OverworldDebugText>>,
    player_pos: Res<PlayerPosition>,
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
    let zone_z = player_pos.z as usize;
    let zone_idx = zone_idx(zone_x, zone_y, zone_z);
    let ozone = overworld.get_overworld_zone(zone_idx);

    let has_road = overworld.zone_has_road(zone_idx);
    let has_river = overworld.zone_has_river(zone_idx);
    let road_connections = 0;

    let town_value = if let Some(town) = &ozone.town {
        town.name.clone()
    } else {
        "No town".to_string()
    };

    let debug_info = format!(
        "Zone {{C|({}, {})}}\nIndex: {{C|{}}}\nType: {{C|{}}}\nTown: {{C|{}}}\nRoad: {{C|{}}}\nRiver: {{C|{}}}\nConnections: {{C|{}}}",
        zone_x,
        zone_y,
        zone_idx,
        ozone.biome_type,
        town_value,
        if has_road { "Yes" } else { "No" },
        if has_river { "Yes" } else { "No" },
        road_connections
    );

    text.value = debug_info;
    *visibility = Visibility::Visible;
}
