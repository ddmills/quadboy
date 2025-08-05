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
        Text::new("123")
            .fg1(Palette::White)
            .bg(Palette::Purple)
            .layer(RenderLayer::Ui),
        Position::new_f32(0., 0.5, 0.),
        PlayerDebug,
        CleanupStateExplore,
    ));

    let hp = (9.5, 0.5);

    cmds.spawn((
        Text::new("HP             ")
            .fg1(Palette::White)
            .bg(Palette::Red)
            .layer(RenderLayer::Ui),
        Position::new_f32(hp.0, hp.1, 0.),
        CleanupStateExplore,
    ));

    cmds.spawn((
        Text::new("            ")
            .fg1(Palette::Black)
            .bg(Palette::Gray)
            .layer(RenderLayer::Ui),
        Position::new_f32(hp.0 + 7.5, hp.1, 0.),
        CleanupStateExplore,
    ));
}

fn on_leave_explore() {
    trace!("LeaveGameState::<Explore>");
}
