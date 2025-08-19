use bevy_ecs::{
    component::Component,
    event::EventReader,
    query::With,
    system::{Commands, Local, Query, Res, ResMut},
};
use macroquad::input::KeyCode;

use crate::{
    common::Palette,
    domain::{GameSettings, SaveGameCommand, SaveGameResult},
    engine::{KeyInput, Plugin, Time},
    rendering::{Position, Text},
    states::{AppState, CurrentAppState, CurrentGameState, GameStatePlugin, cleanup_system},
};

use super::GameState;

pub struct PauseStatePlugin;

impl Plugin for PauseStatePlugin {
    fn build(&self, app: &mut crate::engine::App) {
        GameStatePlugin::new(GameState::Pause)
            .on_enter(app, on_enter_pause)
            .on_update(
                app,
                (
                    listen_for_inputs,
                    update_save_status_display,
                    handle_save_result,
                ),
            )
            .on_leave(app, cleanup_system::<CleanupStatePause>);
    }
}

#[derive(Component)]
pub struct CleanupStatePause;

#[derive(Component)]
pub struct SaveStatusDisplay;

fn on_enter_pause(mut cmds: Commands) {
    cmds.spawn((
        Text::new("PAUSED").bg(Palette::Black),
        Position::new_f32(4., 2., 0.),
        CleanupStatePause,
    ));

    cmds.spawn((
        Text::new("({Y|ESC}) CONTINUE").bg(Palette::Black),
        Position::new_f32(4., 3.5, 0.),
        CleanupStatePause,
    ));

    cmds.spawn((
        Text::new("({Y|S}) SAVE GAME").bg(Palette::Black),
        Position::new_f32(4., 4., 0.),
        CleanupStatePause,
    ));

    cmds.spawn((
        Text::new("({R|Q}) QUIT TO MAIN MENU").bg(Palette::Black),
        Position::new_f32(4., 5., 0.),
        CleanupStatePause,
    ));

    cmds.spawn((
        Text::new("").bg(Palette::Black),
        Position::new_f32(4., 6., 0.),
        CleanupStatePause,
        SaveStatusDisplay,
    ));
}

fn listen_for_inputs(
    mut cmds: Commands,
    keys: Res<KeyInput>,
    mut app_state: ResMut<CurrentAppState>,
    mut game_state: ResMut<CurrentGameState>,
    settings: Res<GameSettings>,
    mut q_save_status: Query<&mut Text, With<SaveStatusDisplay>>,
) {
    if keys.is_pressed(KeyCode::Escape) {
        game_state.next = GameState::Explore;
    }

    if keys.is_pressed(KeyCode::S) {
        save_game(&mut cmds, &settings, &mut q_save_status);
    }

    if keys.is_pressed(KeyCode::Q) {
        app_state.next = AppState::MainMenu;
    }
}

fn save_game(
    cmds: &mut Commands,
    settings: &GameSettings,
    q_save_status: &mut Query<&mut Text, With<SaveStatusDisplay>>,
) {
    if !settings.enable_saves {
        if let Ok(mut text) = q_save_status.single_mut() {
            text.value = "Save disabled in settings".to_string();
        }
        return;
    }

    cmds.queue(SaveGameCommand);

    if let Ok(mut text) = q_save_status.single_mut() {
        text.value = format!("Saving game to '{}'...", settings.save_name);
    }
}

fn update_save_status_display(
    mut q_save_status: Query<&mut Text, With<SaveStatusDisplay>>,
    mut save_message_timer: Local<Option<f64>>,
    time: Res<Time>,
) {
    if let Ok(text) = q_save_status.single()
        && !text.value.is_empty()
        && save_message_timer.is_none()
    {
        *save_message_timer = Some(time.elapsed + 3.0); // Clear after 3 seconds
    }

    if let Some(clear_time) = *save_message_timer
        && time.elapsed >= clear_time
    {
        if let Ok(mut text) = q_save_status.single_mut() {
            text.value.clear();
        }
        *save_message_timer = None;
    }
}

fn handle_save_result(
    mut e_save_result: EventReader<SaveGameResult>,
    mut q_save_status: Query<&mut Text, With<SaveStatusDisplay>>,
) {
    for result in e_save_result.read() {
        if let Ok(mut text) = q_save_status.single_mut() {
            text.value = result.message.clone();
        }
    }
}
