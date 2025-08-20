use bevy_ecs::prelude::*;
use macroquad::{input::KeyCode, prelude::trace};

use crate::{
    domain::GameSettings,
    engine::{App, KeyInput, Plugin},
    rendering::{CameraMode, CrtCurvature, Position, Text},
    states::{AppState, AppStatePlugin, CurrentAppState, cleanup_system},
};

#[derive(Resource)]
struct SettingsUIEntities {
    curvature: Entity,
    scanlines: Entity,
    film_grain: Entity,
    flicker: Entity,
    vignette: Entity,
    chromatic_ab: Entity,
    camera_mode: Entity,
    saves_enabled: Entity,
    save_name: Entity,
    input_rate: Entity,
    input_delay: Entity,
}

pub struct SettingsStatePlugin;

impl Plugin for SettingsStatePlugin {
    fn build(&self, app: &mut App) {
        AppStatePlugin::new(AppState::Settings)
            .on_enter(app, render_settings_menu)
            .on_update(app, (settings_input, update_settings_display))
            .on_leave(
                app,
                (
                    cleanup_system::<CleanupSettings>,
                    remove_settings_ui_resource,
                ),
            );
    }
}

#[derive(Component)]
struct CleanupSettings;

fn render_settings_menu(mut cmds: Commands) {
    trace!("EnterAppState::<Settings>");

    cmds.spawn((
        Text::new("SETTINGS"),
        Position::new_f32(4., 2., 0.),
        CleanupSettings,
    ));

    // CRT Settings Section
    cmds.spawn((
        Text::new("{Y|CRT EFFECTS}"),
        Position::new_f32(4., 4., 0.),
        CleanupSettings,
    ));

    let curvature = cmds
        .spawn((
            Text::new(""),
            Position::new_f32(6., 4.5, 0.),
            CleanupSettings,
        ))
        .id();

    let scanlines = cmds
        .spawn((
            Text::new(""),
            Position::new_f32(6., 5., 0.),
            CleanupSettings,
        ))
        .id();

    let film_grain = cmds
        .spawn((
            Text::new(""),
            Position::new_f32(6., 5.5, 0.),
            CleanupSettings,
        ))
        .id();

    let flicker = cmds
        .spawn((
            Text::new(""),
            Position::new_f32(6., 6., 0.),
            CleanupSettings,
        ))
        .id();

    let vignette = cmds
        .spawn((
            Text::new(""),
            Position::new_f32(6., 6.5, 0.),
            CleanupSettings,
        ))
        .id();

    let chromatic_ab = cmds
        .spawn((
            Text::new(""),
            Position::new_f32(6., 7., 0.),
            CleanupSettings,
        ))
        .id();

    // Camera Settings Section
    cmds.spawn((
        Text::new("{Y|CAMERA}"),
        Position::new_f32(4., 8.5, 0.),
        CleanupSettings,
    ));

    let camera_mode = cmds
        .spawn((
            Text::new(""),
            Position::new_f32(6., 9., 0.),
            CleanupSettings,
        ))
        .id();

    // Save Settings Section
    cmds.spawn((
        Text::new("{Y|SAVE SETTINGS}"),
        Position::new_f32(4., 10.5, 0.),
        CleanupSettings,
    ));

    let saves_enabled = cmds
        .spawn((
            Text::new(""),
            Position::new_f32(6., 11., 0.),
            CleanupSettings,
        ))
        .id();

    let save_name = cmds
        .spawn((
            Text::new(""),
            Position::new_f32(6., 11.5, 0.),
            CleanupSettings,
        ))
        .id();

    // Input Settings Section
    cmds.spawn((
        Text::new("{Y|INPUT}"),
        Position::new_f32(4., 13., 0.),
        CleanupSettings,
    ));

    let input_rate = cmds
        .spawn((
            Text::new(""),
            Position::new_f32(6., 13.5, 0.),
            CleanupSettings,
        ))
        .id();

    let input_delay = cmds
        .spawn((
            Text::new(""),
            Position::new_f32(6., 14., 0.),
            CleanupSettings,
        ))
        .id();

    // Controls
    cmds.spawn((
        Text::new("({R|ESC}) BACK TO MAIN MENU"),
        Position::new_f32(4., 15., 0.),
        CleanupSettings,
    ));

    // Insert the resource with all entity IDs
    cmds.insert_resource(SettingsUIEntities {
        curvature,
        scanlines,
        film_grain,
        flicker,
        vignette,
        chromatic_ab,
        camera_mode,
        saves_enabled,
        save_name,
        input_rate,
        input_delay,
    });
}

fn settings_input(
    keys: Res<KeyInput>,
    mut app_state: ResMut<CurrentAppState>,
    mut settings: ResMut<GameSettings>,
) {
    // ESC to go back
    if keys.is_pressed(KeyCode::Escape) {
        app_state.next = AppState::MainMenu;
        return;
    }

    // CRT Settings
    if keys.is_pressed(KeyCode::Key1) {
        settings.crt_curvature = match settings.crt_curvature {
            CrtCurvature::Off => CrtCurvature::Curve(9.0, 7.0),
            CrtCurvature::Curve(_, _) => CrtCurvature::Off,
        };
    }

    if keys.is_pressed(KeyCode::Key2) {
        settings.crt_scanline = !settings.crt_scanline;
    }

    if keys.is_pressed(KeyCode::Key3) {
        settings.crt_film_grain = !settings.crt_film_grain;
    }

    if keys.is_pressed(KeyCode::Key4) {
        settings.crt_flicker = !settings.crt_flicker;
    }

    if keys.is_pressed(KeyCode::Key5) {
        settings.crt_vignette = !settings.crt_vignette;
    }

    if keys.is_pressed(KeyCode::Key6) {
        settings.crt_chromatic_ab = !settings.crt_chromatic_ab;
    }

    // Camera Mode
    if keys.is_pressed(KeyCode::Key7) {
        settings.camera_mode = match settings.camera_mode {
            CameraMode::Snap => CameraMode::Smooth(0.04),
            CameraMode::Smooth(_) => CameraMode::Snap,
        };
    }

    // Save Settings
    if keys.is_pressed(KeyCode::Key8) {
        settings.enable_saves = !settings.enable_saves;
    }

    // Input Rate controls (Up/Down arrows)
    if keys.is_pressed(KeyCode::Up) {
        settings.input_rate = (settings.input_rate + 0.005).min(0.2);
    }

    if keys.is_pressed(KeyCode::Down) {
        settings.input_rate = (settings.input_rate - 0.005).max(0.01);
    }

    // Input Delay controls (Left/Right arrows)
    if keys.is_pressed(KeyCode::Left) {
        settings.input_initial_delay = (settings.input_initial_delay - 0.01).max(0.05);
    }

    if keys.is_pressed(KeyCode::Right) {
        settings.input_initial_delay = (settings.input_initial_delay + 0.01).min(0.5);
    }
}

fn update_settings_display(
    mut q_text: Query<&mut Text>,
    settings: Res<GameSettings>,
    ui_entities: Option<Res<SettingsUIEntities>>,
) {
    let Some(ui_entities) = ui_entities else {
        return;
    };

    if !settings.is_changed() && !ui_entities.is_changed() {
        return;
    }

    // Update curvature
    if let Ok(mut text) = q_text.get_mut(ui_entities.curvature) {
        text.value = match settings.crt_curvature {
            CrtCurvature::Off => "({Y|1}) Curvature: {R|OFF}".to_string(),
            CrtCurvature::Curve(x, y) => format!("({{Y|1}}) Curvature: {{G|{:.1}, {:.1}}}", x, y),
        };
    }

    // Update scanlines
    if let Ok(mut text) = q_text.get_mut(ui_entities.scanlines) {
        text.value = format!(
            "({{Y|2}}) Scanlines: {}",
            if settings.crt_scanline {
                "{G|ON}"
            } else {
                "{R|OFF}"
            }
        );
    }

    // Update film grain
    if let Ok(mut text) = q_text.get_mut(ui_entities.film_grain) {
        text.value = format!(
            "({{Y|3}}) Film Grain: {}",
            if settings.crt_film_grain {
                "{G|ON}"
            } else {
                "{R|OFF}"
            }
        );
    }

    // Update flicker
    if let Ok(mut text) = q_text.get_mut(ui_entities.flicker) {
        text.value = format!(
            "({{Y|4}}) Flicker: {}",
            if settings.crt_flicker {
                "{G|ON}"
            } else {
                "{R|OFF}"
            }
        );
    }

    // Update vignette
    if let Ok(mut text) = q_text.get_mut(ui_entities.vignette) {
        text.value = format!(
            "({{Y|5}}) Vignette: {}",
            if settings.crt_vignette {
                "{G|ON}"
            } else {
                "{R|OFF}"
            }
        );
    }

    // Update chromatic aberration
    if let Ok(mut text) = q_text.get_mut(ui_entities.chromatic_ab) {
        text.value = format!(
            "({{Y|6}}) Chromatic Aberration: {}",
            if settings.crt_chromatic_ab {
                "{G|ON}"
            } else {
                "{R|OFF}"
            }
        );
    }

    // Update camera mode
    if let Ok(mut text) = q_text.get_mut(ui_entities.camera_mode) {
        text.value = match settings.camera_mode {
            CameraMode::Snap => "({Y|7}) Camera Mode: {G|SNAP}".to_string(),
            CameraMode::Smooth(speed) => {
                format!("({{Y|7}}) Camera Mode: {{G|SMOOTH ({:.3})}}", speed)
            }
        };
    }

    // Update saves enabled
    if let Ok(mut text) = q_text.get_mut(ui_entities.saves_enabled) {
        text.value = format!(
            "({{Y|8}}) Saves Enabled: {}",
            if settings.enable_saves {
                "{G|ON}"
            } else {
                "{R|OFF}"
            }
        );
    }

    // Update save name
    if let Ok(mut text) = q_text.get_mut(ui_entities.save_name) {
        text.value = format!("({{Y|9}}) Save Name: {{G|{}}}", settings.save_name);
    }

    // Update input rate
    if let Ok(mut text) = q_text.get_mut(ui_entities.input_rate) {
        text.value = format!(
            "({{Y|↑}}/{{Y|↓}}) Input Rate: {{G|{:.3}}}",
            settings.input_rate
        );
    }

    // Update input delay
    if let Ok(mut text) = q_text.get_mut(ui_entities.input_delay) {
        text.value = format!(
            "({{Y|←}}/{{Y|→}}) Input Delay: {{G|{:.3}}}",
            settings.input_initial_delay
        );
    }
}

fn remove_settings_ui_resource(mut cmds: Commands) {
    cmds.remove_resource::<SettingsUIEntities>();
}
