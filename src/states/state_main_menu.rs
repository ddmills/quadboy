use bevy_ecs::prelude::*;
use macroquad::{input::KeyCode, prelude::trace};

use crate::{
    common::Palette,
    engine::{App, ExitAppEvent, KeyInput, Plugin},
    rendering::{Position, RenderLayer, Text},
    states::{AppState, AppStatePlugin, CurrentAppState, cleanup_system},
};

pub struct MainMenuStatePlugin;

impl Plugin for MainMenuStatePlugin {
    fn build(&self, app: &mut App) {
        AppStatePlugin::new(AppState::MainMenu)
            .on_enter(app, render_menu)
            .on_update(app, main_menu_input)
            .on_leave(app, cleanup_system::<CleanupMainMenu>);
    }
}

#[derive(Component)]
struct CleanupMainMenu;

fn render_menu(mut cmds: Commands) {
    trace!("EnterAppState::<MainMenu>");

    cmds.spawn((
        Text::new("Welcome to...   QUADBOY"),
        Position::new_f32(4., 2., 0.),
        CleanupMainMenu,
    ));

    cmds.spawn((
        Text::new("A cowboy adventure."),
        Position::new_f32(4., 2.5, 0.),
        CleanupMainMenu,
    ));

    cmds.spawn((
        Text::new("({G|N}) NEW GAME"),
        Position::new_f32(4., 4., 0.),
        CleanupMainMenu,
    ));

    cmds.spawn((
        Text::new("(L) LOAD"),
        Position::new_f32(4., 4.5, 0.),
        CleanupMainMenu,
    ));

    cmds.spawn((
        Text::new("({R|Q}) QUIT"),
        Position::new_f32(4., 5., 0.),
        CleanupMainMenu,
    ));
}

fn main_menu_input(
    keys: Res<KeyInput>,
    mut state: ResMut<CurrentAppState>,
    mut e_exit_app: EventWriter<ExitAppEvent>,
) {
    if keys.is_pressed(KeyCode::N) {
        state.next = AppState::Play;
    }

    if keys.is_pressed(KeyCode::Q) {
        e_exit_app.write(ExitAppEvent);
    }
}
