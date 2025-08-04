use bevy_ecs::prelude::*;
use macroquad::{input::KeyCode, prelude::trace};

use crate::{common::Palette, engine::{App, KeyInput, Plugin, ScheduleType}, rendering::{Position, RenderLayer, Text}, states::{cleanup_system, enter_state, in_state, leave_state, CurrentState, GameState}};

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(ScheduleType::PreUpdate, (render_menu).run_if(enter_state(GameState::MainMenu)))
            .add_systems(ScheduleType::Update,
                main_menu_input.run_if(in_state(GameState::MainMenu)),
            )
            .add_systems(ScheduleType::PostUpdate, cleanup_system::<CleanupMainMenu>.run_if(leave_state(GameState::MainMenu)));
    }
}

#[derive(Component)]
struct CleanupMainMenu;

fn render_menu(mut cmds: Commands)
{
    trace!("enter main menu!");

    cmds.spawn((
        Text::new("(N) NEW GAME")
            .fg1(Palette::White)
            .layer(RenderLayer::Ui),
        Position::new_f32(4., 4., 0.),
        CleanupMainMenu,
    ));

    cmds.spawn((
        Text::new("(L) LOAD")
            .fg1(Palette::White)
            .layer(RenderLayer::Ui),
        Position::new_f32(4., 4.5, 0.),
        CleanupMainMenu,
    ));

    cmds.spawn((
        Text::new("(Q) QUIT")
            .fg1(Palette::White)
            .layer(RenderLayer::Ui),
        Position::new_f32(4., 5., 0.),
        CleanupMainMenu,
    ));
}

fn main_menu_input(
    keys: Res<KeyInput>,
    mut state: ResMut<CurrentState>,
) {
    if keys.is_pressed(KeyCode::N)
    {
        state.next = GameState::Playing;
    }
}
