use bevy_ecs::{
    change_detection::DetectChanges,
    component::Component,
    query::With,
    system::{Commands, Query, Res, ResMut},
};
use macroquad::input::KeyCode;

use crate::{
    domain::GameSettings,
    engine::{KeyInput, Plugin},
    rendering::{CrtCurvature, Position, Text},
    states::{CurrentGameState, GameStatePlugin, cleanup_system},
};

use super::GameState;

pub struct PauseStatePlugin;

impl Plugin for PauseStatePlugin {
    fn build(&self, app: &mut crate::engine::App) {
        GameStatePlugin::new(GameState::Pause)
            .on_enter(app, on_enter_pause)
            .on_update(app, (listen_for_inputs, update_curvature_display))
            .on_leave(app, cleanup_system::<CleanupStatePause>);
    }
}

#[derive(Component)]
pub struct CleanupStatePause;

#[derive(Component)]
pub struct CrtCurvatureDisplay;

fn on_enter_pause(mut cmds: Commands, settings: Res<GameSettings>) {
    cmds.spawn((
        Text::new("PAUSED"),
        Position::new_f32(12., 1., 0.),
        CleanupStatePause,
    ));

    cmds.spawn((
        Text::new("Use {G|UP}/{G|DOWN} arrows to adjust CRT curvature"),
        Position::new_f32(4., 3., 0.),
        CleanupStatePause,
    ));

    let curvature_text = match settings.crt_curvature {
        CrtCurvature::Off => "CRT Curvature: OFF".to_string(),
        CrtCurvature::Curve(x, y) => format!("CRT Curvature: {:.1}, {:.1}", x, y),
    };

    cmds.spawn((
        Text::new(&curvature_text),
        Position::new_f32(4., 4., 0.),
        CleanupStatePause,
        CrtCurvatureDisplay,
    ));
}

fn listen_for_inputs(
    keys: Res<KeyInput>,
    mut game_state: ResMut<CurrentGameState>,
    mut settings: ResMut<GameSettings>,
) {
    if keys.is_pressed(KeyCode::P) {
        game_state.next = GameState::Explore;
    }

    if keys.is_pressed(KeyCode::Left) {
        settings.crt_curvature = CrtCurvature::Off;
    }

    if keys.is_pressed(KeyCode::Right) {
        settings.crt_curvature = CrtCurvature::Curve(8., 8.);
    }

    if keys.is_pressed(KeyCode::Up) {
        adjust_crt_curvature(&mut settings, 0.25);
    }

    if keys.is_pressed(KeyCode::Down) {
        adjust_crt_curvature(&mut settings, -0.25);
    }
}

fn adjust_crt_curvature(settings: &mut GameSettings, delta: f32) {
    match settings.crt_curvature {
        CrtCurvature::Off => {
            if delta > 0.0 {
                settings.crt_curvature = CrtCurvature::Curve(1.0, 1.0);
            }
        }
        CrtCurvature::Curve(x, y) => {
            let new_x = (x + delta).clamp(2.0, 10.0);
            let new_y = (y + delta).clamp(2.0, 10.0);

            settings.crt_curvature = CrtCurvature::Curve(new_x, new_y);
        }
    }
}

fn update_curvature_display(
    mut q_display: Query<&mut Text, With<CrtCurvatureDisplay>>,
    settings: Res<GameSettings>,
) {
    if settings.is_changed()
        && let Ok(mut text) = q_display.single_mut()
    {
        text.value = match settings.crt_curvature {
            CrtCurvature::Off => "CRT Curvature: OFF".to_string(),
            CrtCurvature::Curve(x, y) => format!("CRT Curvature: {:.1}, {:.1}", x, y),
        };
    }
}
