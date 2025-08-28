use bevy_ecs::prelude::*;
use macroquad::prelude::trace;
use serde::{Deserialize, Serialize};

use crate::{
    common::Palette,
    domain::{
        game_loop, player_input, render_player_debug, IsExplored, Label, Player, PlayerDebug, PlayerMovedEvent, PlayerPosition, Zone
    },
    engine::{App, Clock, Mouse, Plugin, SerializableComponent},
    rendering::{world_to_zone_idx, world_to_zone_local, Glyph, Layer, Position, Text, Visibility},
    states::{cleanup_system, GameStatePlugin},
};

use super::GameState;

pub struct ExploreStatePlugin;

impl Plugin for ExploreStatePlugin {
    fn build(&self, app: &mut App) {
        GameStatePlugin::new(GameState::Explore)
            .on_enter(app, (on_enter_explore, center_camera_on_player).chain())
            .on_update(
                app,
                (
                    render_player_debug,
                    render_tick_display,
                    render_cursor,
                    display_entity_names_at_mouse,
                    player_input,
                    game_loop,
                ),
            )
            .on_leave(
                app,
                (on_leave_explore, cleanup_system::<CleanupStateExplore>).chain(),
            );
    }
}

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct CleanupStateExplore;

fn on_enter_explore(mut cmds: Commands) {
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
    q_names: Query<&Label, With<IsExplored>>,
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
        if let Ok(name) = q_names.get(*entity) {
            names.push(name.get().to_string());
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
        "{{G|{}}}.{{g|{:03}}}",
        clock.current_turn(),
        clock.sub_turn(),
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
