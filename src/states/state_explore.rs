use bevy_ecs::prelude::*;
use macroquad::prelude::trace;

use crate::{
    common::Palette,
    domain::{PlayerDebug, player_input, render_player_debug},
    engine::{App, Plugin},
    rendering::{Position, RenderLayer, Text},
    states::{GameStatePlugin, cleanup_system},
};

use super::GameState;

pub struct ExploreStatePlugin;

impl Plugin for ExploreStatePlugin {
    fn build(&self, app: &mut App) {
        GameStatePlugin::new(GameState::Explore)
            .on_enter(app, render_explore_hud)
            .on_update(app, (player_input, render_player_debug))
            .on_leave(
                app,
                (on_leave_explore, cleanup_system::<CleanupStateExplore>).chain(),
            );
    }
}

#[derive(Component)]
pub struct CleanupStateExplore;

fn render_explore_hud(mut cmds: Commands) {
    trace!("EnterGameState::<Explore>");

    cmds.spawn((
        Text::new("123").bg(Palette::Black),
        Position::new_f32(6., 0., 0.),
        PlayerDebug,
        CleanupStateExplore,
    ));

    cmds.spawn((
        Text::new("Under the {C-b border|vast, starry sky}, your {R|heart} aches").bg(Palette::Black),
        Position::new_f32(0., 13., 0.),
        CleanupStateExplore
    ));

    cmds.spawn((
        Text::new("for new {r-R-Y-Y-Y-Y-R-r stretch|horizons} and {G-g-o-G-g-o repeat|untamed trails}.").bg(Palette::Black),
        Position::new_f32(0., 13.5, 0.),
        CleanupStateExplore
    ));
    cmds.spawn((
        Text::new("With a steady hand, you grip the {C-c-w-W-Y-C-c-C-w repeat|chrome-plated pistol},").bg(Palette::Black),
        Position::new_f32(0., 14., 0.),
        CleanupStateExplore
    ));
    cmds.spawn((
        Text::new("Eyes Scanning The {b|Darkness}, ready to face the Unknown.").bg(Palette::Black),
        Position::new_f32(0., 14.5, 0.),
        CleanupStateExplore
    ));
}

fn on_leave_explore() {
    trace!("LeaveGameState::<Explore>");
}
