use bevy_ecs::prelude::*;
use macroquad::prelude::trace;

use crate::{common::Palette, domain::{player_input, render_player_debug, PlayerDebug}, engine::{App, Plugin, ScheduleType}, rendering::{Position, RenderLayer, Text}, states::{cleanup_system, enter_game_state, in_game_state, leave_game_state}};

use super::GameState;

pub struct ExploreStatePlugin;

impl Plugin for ExploreStatePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(ScheduleType::PreUpdate, (
                render_explore_hud,
            ).chain().run_if(enter_game_state(GameState::Explore)))
            .add_systems(ScheduleType::Update, (
                player_input,
                render_player_debug,
            ).run_if(in_game_state(GameState::Explore)))
            .add_systems(ScheduleType::PostUpdate, 
                (
                    on_leave_explore,
                    cleanup_system::<CleanupStateExplore>,
                ).chain().run_if(leave_game_state(GameState::Explore))
            );
    }
}

#[derive(Component)]
pub struct CleanupStateExplore;

fn render_explore_hud(mut cmds: Commands)
{
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
