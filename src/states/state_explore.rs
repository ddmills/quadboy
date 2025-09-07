use bevy_ecs::{prelude::*, system::SystemId};
use macroquad::{input::KeyCode, prelude::trace};
use serde::{Deserialize, Serialize};

use crate::{
    common::Palette,
    domain::{
        IgnoreLighting, IsExplored, Label, Player, PlayerDebug, PlayerMovedEvent, PlayerPosition,
        StackCount, Zone, game_loop, handle_item_pickup, player_input, render_player_debug,
    },
    engine::{App, Clock, Mouse, Plugin, SerializableComponent},
    rendering::{
        Glyph, Layer, LightingData, Position, Text, Visibility, world_to_zone_idx,
        world_to_zone_local,
    },
    states::{CurrentGameState, GameStatePlugin, cleanup_system},
    ui::Button,
};

use super::GameState;

#[derive(Resource)]
struct ExploreCallbacks {
    open_map: SystemId,
    open_inventory: SystemId,
}

pub struct ExploreStatePlugin;

impl Plugin for ExploreStatePlugin {
    fn build(&self, app: &mut App) {
        GameStatePlugin::new(GameState::Explore)
            .on_enter(
                app,
                (setup_callbacks, on_enter_explore, center_camera_on_player).chain(),
            )
            .on_update(
                app,
                (
                    render_player_debug,
                    render_tick_display,
                    render_lighting_debug,
                    render_lighting_ambient_debug,
                    render_cursor,
                    display_entity_names_at_mouse,
                    player_input,
                    handle_item_pickup,
                    game_loop,
                ),
            )
            .on_leave(
                app,
                (
                    on_leave_explore,
                    cleanup_system::<CleanupStateExplore>,
                    remove_explore_callbacks,
                )
                    .chain(),
            );
    }
}

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct CleanupStateExplore;

fn setup_callbacks(world: &mut World) {
    let callbacks = ExploreCallbacks {
        open_map: world.register_system(open_map),
        open_inventory: world.register_system(open_inventory),
    };

    world.insert_resource(callbacks);
}

fn open_map(mut game_state: ResMut<CurrentGameState>) {
    game_state.next = GameState::Overworld;
}

fn open_inventory(mut game_state: ResMut<CurrentGameState>) {
    game_state.next = GameState::Inventory;
}

fn remove_explore_callbacks(mut cmds: Commands) {
    cmds.remove_resource::<ExploreCallbacks>();
}

fn on_enter_explore(mut cmds: Commands, callbacks: Res<ExploreCallbacks>) {
    trace!("EnterGameState::<Explore>");

    cmds.spawn((
        Text::new("123").bg(Palette::Black),
        Position::new_f32(6., 0., 0.),
        PlayerDebug,
        CleanupStateExplore,
    ));

    cmds.spawn((
        Text::new("Turn: 0.000").bg(Palette::Black),
        Position::new_f32(0., 0.5, 0.),
        TickDisplay,
        CleanupStateExplore,
    ));

    cmds.spawn((
        Text::new("Light: R:0.0 G:0.0 B:0.0 I:0.0").bg(Palette::Black),
        Position::new_f32(0., 1.0, 0.),
        LightingDebugText,
        CleanupStateExplore,
    ));
    cmds.spawn((
        Text::new("#ff00ff").fg1(Palette::White).bg(0xff00ff_u32),
        Position::new_f32(0., 2.5, 0.),
        LightingDebugAmbient,
        CleanupStateExplore,
    ));

    cmds.spawn((
        Glyph::new(0, Palette::Orange, Palette::Orange)
            .bg(Palette::Orange)
            .layer(Layer::GroundOverlay),
        Position::new_f32(0., 0., 0.),
        CursorGlyph,
        CleanupStateExplore,
    ));

    cmds.spawn((
        Text::new("")
            .fg1(Palette::White)
            .bg(Palette::Black)
            .layer(Layer::Overlay),
        Position::new_f32(0., 0., 0.),
        Visibility::Hidden,
        MouseHoverText,
        IgnoreLighting,
        CleanupStateExplore,
    ));

    cmds.spawn((
        Position::new_f32(3., 1.5, 0.),
        Button::new("({Y|M}) MAP", callbacks.open_map).hotkey(KeyCode::M),
        CleanupStateExplore,
    ));

    cmds.spawn((
        Position::new_f32(7., 1.5, 0.),
        Button::new("({Y|I}) INVENTORY", callbacks.open_inventory).hotkey(KeyCode::I),
        CleanupStateExplore,
    ));
}

fn on_leave_explore() {
    trace!("LeaveGameState::<Explore>");
}

#[derive(Component)]
struct CursorGlyph;

#[derive(Component)]
struct MouseHoverText;

#[derive(Component)]
struct TickDisplay;

#[derive(Component)]
struct LightingDebugText;

#[derive(Component)]
struct LightingDebugAmbient;

fn render_cursor(
    mouse: Res<Mouse>,
    mut q_cursor: Query<&mut Position, With<CursorGlyph>>,
    player_pos: Res<PlayerPosition>,
) {
    let Ok(mut cursor) = q_cursor.single_mut() else {
        return;
    };

    cursor.x = mouse.world.0.floor();
    cursor.y = mouse.world.1.floor();
    cursor.z = player_pos.z.floor();
}

fn display_entity_names_at_mouse(
    mouse: Res<Mouse>,
    player_pos: Res<PlayerPosition>,
    q_zones: Query<&Zone>,
    q_names: Query<(&Label, Option<&StackCount>), With<IsExplored>>,
    mut q_hover_text: Query<(&mut Text, &mut Position, &mut Visibility), With<MouseHoverText>>,
) {
    let mouse_x = mouse.world.0.floor() as usize;
    let mouse_y = mouse.world.1.floor() as usize;
    let mouse_z = player_pos.z as usize;
    let mut names: Vec<String> = Vec::new();

    let zone_idx = world_to_zone_idx(mouse_x, mouse_y, mouse_z);
    let (local_x, local_y) = world_to_zone_local(mouse_x, mouse_y);

    let Some(zone) = q_zones.iter().find(|z| z.idx == zone_idx) else {
        return;
    };

    let Some(entities) = zone.entities.get(local_x, local_y) else {
        return;
    };

    for entity in entities {
        if let Ok((name, stack_count)) = q_names.get(*entity) {
            let mut name = name.get().to_string();

            if let Some(stack) = stack_count
                && stack.count > 1
            {
                name = format!("{} x{}", name, stack.count)
            }

            names.push(name);
        }
    }

    let Ok((mut text, mut text_pos, mut visibility)) = q_hover_text.single_mut() else {
        return;
    };

    if names.is_empty() {
        *visibility = Visibility::Hidden;
        text.value = String::new();
    } else {
        *visibility = Visibility::Visible;
        text.value = names.join(", ");
        text_pos.x = mouse_x as f32 + 1.0;
        text_pos.y = mouse_y as f32;
        text_pos.z = mouse_z as f32;
    }
}

fn render_tick_display(clock: Res<Clock>, mut q_tick_display: Query<&mut Text, With<TickDisplay>>) {
    let Ok(mut text) = q_tick_display.single_mut() else {
        return;
    };

    text.value = format!(
        "{{G|{}}}.{{g|{:03}}} {{G|Day {}}} {{Y|{:02}}}:{{g|{:02}}}",
        clock.current_turn(),
        clock.sub_turn(),
        clock.get_day() + 1,
        clock.get_hour(),
        clock.get_minute() % 60,
    );
}

fn center_camera_on_player(
    mut e_player_moved: EventWriter<PlayerMovedEvent>,
    q_player: Query<&Position, With<Player>>,
) {
    let p = q_player.single().expect("Expect Player").world();
    e_player_moved.write(PlayerMovedEvent {
        x: p.0,
        y: p.1,
        z: p.2,
    });
}

fn render_lighting_ambient_debug(
    lighting_data: Res<LightingData>,
    mut q_debug_ambient_text: Query<&mut Text, With<LightingDebugAmbient>>,
) {
    let Ok(mut text) = q_debug_ambient_text.single_mut() else {
        return;
    };

    // Get lighting at cursor position
    let light_value = lighting_data.get_ambient_vec4();

    let r = light_value.x;
    let g = light_value.y;
    let b = light_value.z;
    let intensity = light_value.w;

    // Convert to hex for comparison
    let hex_r = (r * 255.0) as u32;
    let hex_g = (g * 255.0) as u32;
    let hex_b = (b * 255.0) as u32;
    let hex_color = (hex_r << 16) | (hex_g << 8) | hex_b;

    text.bg = Some(lighting_data.get_ambient_color());

    text.value = format!("#{:06X} ({:.2})", hex_color, intensity);
}

fn render_lighting_debug(
    mouse: Res<Mouse>,
    lighting_data: Res<LightingData>,
    mut q_debug_text: Query<&mut Text, With<LightingDebugText>>,
) {
    let Ok(mut text) = q_debug_text.single_mut() else {
        return;
    };

    let mouse_x = mouse.world.0.floor() as usize;
    let mouse_y = mouse.world.1.floor() as usize;
    let (local_x, local_y) = world_to_zone_local(mouse_x, mouse_y);

    // Get lighting at cursor position
    let light_info = if let Some(light_value) = lighting_data.get_light(local_x, local_y) {
        let r = light_value.rgb.x;
        let g = light_value.rgb.y;
        let b = light_value.rgb.z;
        let intensity = light_value.intensity;
        let flicker = light_value.flicker;

        // Convert to hex for comparison
        let hex_r = (r * 255.0) as u32;
        let hex_g = (g * 255.0) as u32;
        let hex_b = (b * 255.0) as u32;
        let hex_color = (hex_r << 16) | (hex_g << 8) | hex_b;

        format!(
            "Light: R:{:.2} G:{:.2} B:{:.2} I:{:.2} F:{:.2} (#{:06X})",
            r, g, b, intensity, flicker, hex_color
        )
    } else {
        "Light: No data".to_string()
    };

    // Get ambient info for comparison
    let ambient_color = lighting_data.get_ambient_color();
    let ambient_intensity = lighting_data.get_ambient_intensity();
    let ambient_r = ((ambient_color >> 16) & 0xFF) as f32 / 255.0;
    let ambient_g = ((ambient_color >> 8) & 0xFF) as f32 / 255.0;
    let ambient_b = (ambient_color & 0xFF) as f32 / 255.0;

    text.value = format!(
        "{}\nAmbient: R:{:.2} G:{:.2} B:{:.2} I:{:.2} (#{:06X})",
        light_info, ambient_r, ambient_g, ambient_b, ambient_intensity, ambient_color
    );
}
