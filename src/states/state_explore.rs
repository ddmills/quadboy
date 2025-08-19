use bevy_ecs::prelude::*;
use macroquad::prelude::trace;
use serde::{Deserialize, Serialize};

use crate::{
    common::Palette,
    domain::{PlayerDebug, player_input, render_player_debug},
    engine::{App, Mouse, Plugin, SerializableComponent},
    rendering::{Glyph, Position, RenderLayer, Text},
    states::{GameStatePlugin, cleanup_system},
};

use super::GameState;

pub struct ExploreStatePlugin;

impl Plugin for ExploreStatePlugin {
    fn build(&self, app: &mut App) {
        GameStatePlugin::new(GameState::Explore)
            .on_enter(app, on_enter_explore)
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

    cmds.spawn((
        Text::new("Under the {C-b border|vast, starry sky}, your {R|heart} aches")
            .bg(Palette::Black),
        Position::new_f32(0., 13., 0.),
        CleanupStateExplore,
    ));

    cmds.spawn((
        Text::new(
            "for new {r-R-Y-Y-Y-Y-R-r stretch|horizons} and {G-g-o-G-g-o repeat|untamed trails}.",
        )
        .bg(Palette::Black),
        Position::new_f32(0., 13.5, 0.),
        CleanupStateExplore,
    ));
    cmds.spawn((
        Text::new(
            "With a steady hand, you grip the {C-c-w-W-Y-C-c-C-w repeat|chrome-plated pistol},",
        )
        .bg(Palette::Black),
        Position::new_f32(0., 14., 0.),
        CleanupStateExplore,
    ));
    cmds.spawn((
        Text::new("Eyes Scanning The {b|Darkness}, ready to face the Unknown.").bg(Palette::Black),
        Position::new_f32(0., 14.5, 0.),
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
