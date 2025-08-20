use bevy_ecs::prelude::*;
use macroquad::prelude::trace;
use serde::{Deserialize, Serialize};

use crate::{
    common::Palette,
    domain::{Player, PlayerDebug, PlayerMovedEvent, player_input, render_player_debug},
    engine::{App, Mouse, Plugin, SerializableComponent},
    rendering::{Glyph, Position, RenderLayer, Text},
    states::{GameStatePlugin, cleanup_system},
};

use super::GameState;

pub struct ExploreStatePlugin;

impl Plugin for ExploreStatePlugin {
    fn build(&self, app: &mut App) {
        GameStatePlugin::new(GameState::Explore)
            .on_enter(app, (on_enter_explore, center_camera_on_player))
            .on_update(app, (player_input, render_player_debug, render_cursor))
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
        Glyph::new(0, Palette::Orange, Palette::Orange)
            .bg(Palette::Orange)
            .layer(RenderLayer::Actors),
        Position::new_f32(0., 0., 0.),
        CursorGlyph,
        CleanupStateExplore,
    ));
}

fn on_leave_explore() {
    trace!("LeaveGameState::<Explore>");
}

#[derive(Component)]
struct CursorGlyph;

fn render_cursor(mouse: Res<Mouse>, mut q_cursor: Query<&mut Position, With<CursorGlyph>>) {
    let Ok(mut cursor) = q_cursor.single_mut() else {
        return;
    };

    cursor.x = mouse.world.0.floor();
    cursor.y = mouse.world.1.floor();
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
