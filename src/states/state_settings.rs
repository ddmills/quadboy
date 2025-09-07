use bevy_ecs::{prelude::*, system::SystemId};
use macroquad::{input::KeyCode, prelude::trace};

use crate::{
    domain::GameSettings,
    engine::{App, AudioKey, Plugin},
    rendering::{CameraMode, CrtCurvature, Layer, Position, Text},
    states::{AppState, AppStatePlugin, CurrentAppState, cleanup_system},
    ui::{ActivatableBuilder, Button},
};

#[derive(Resource)]
struct SettingsCallbacks {
    toggle_curvature: SystemId,
    toggle_scanlines: SystemId,
    toggle_film_grain: SystemId,
    toggle_flicker: SystemId,
    toggle_vignette: SystemId,
    toggle_chromatic_ab: SystemId,
    toggle_camera_mode: SystemId,
    toggle_saves: SystemId,
    back_to_menu: SystemId,
}

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
            .on_enter(app, (setup_callbacks, render_settings_menu).chain())
            .on_update(app, update_settings_display)
            .on_leave(
                app,
                (
                    cleanup_system::<CleanupSettings>,
                    remove_settings_ui_resource,
                    remove_settings_callbacks,
                ),
            );
    }
}

#[derive(Component)]
struct CleanupSettings;

fn setup_callbacks(world: &mut World) {
    let callbacks = SettingsCallbacks {
        toggle_curvature: world.register_system(toggle_curvature),
        toggle_scanlines: world.register_system(toggle_scanlines),
        toggle_film_grain: world.register_system(toggle_film_grain),
        toggle_flicker: world.register_system(toggle_flicker),
        toggle_vignette: world.register_system(toggle_vignette),
        toggle_chromatic_ab: world.register_system(toggle_chromatic_ab),
        toggle_camera_mode: world.register_system(toggle_camera_mode),
        toggle_saves: world.register_system(toggle_saves),
        back_to_menu: world.register_system(back_to_menu),
    };

    world.insert_resource(callbacks);
}

fn toggle_curvature(mut settings: ResMut<GameSettings>) {
    settings.crt_curvature = match settings.crt_curvature {
        CrtCurvature::Off => CrtCurvature::Curve(9.0, 7.0),
        CrtCurvature::Curve(_, _) => CrtCurvature::Off,
    };
}

fn toggle_scanlines(mut settings: ResMut<GameSettings>) {
    settings.crt_scanline = !settings.crt_scanline;
}

fn toggle_film_grain(mut settings: ResMut<GameSettings>) {
    settings.crt_film_grain = !settings.crt_film_grain;
}

fn toggle_flicker(mut settings: ResMut<GameSettings>) {
    settings.crt_flicker = !settings.crt_flicker;
}

fn toggle_vignette(mut settings: ResMut<GameSettings>) {
    settings.crt_vignette = !settings.crt_vignette;
}

fn toggle_chromatic_ab(mut settings: ResMut<GameSettings>) {
    settings.crt_chromatic_ab = !settings.crt_chromatic_ab;
}

fn toggle_camera_mode(mut settings: ResMut<GameSettings>) {
    settings.camera_mode = match settings.camera_mode {
        CameraMode::Snap => CameraMode::Smooth(0.02),
        CameraMode::Smooth(_) => CameraMode::Snap,
    };
}

fn toggle_saves(mut settings: ResMut<GameSettings>) {
    settings.enable_saves = !settings.enable_saves;
}

fn back_to_menu(mut app_state: ResMut<CurrentAppState>) {
    app_state.next = AppState::MainMenu;
}

fn render_settings_menu(mut cmds: Commands, callbacks: Res<SettingsCallbacks>) {
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
            Position::new_f32(6., 4.5, 0.),
            ActivatableBuilder::new("", callbacks.toggle_curvature)
                .with_hotkey(KeyCode::Key1)
                .with_focus_order(1000)
                .as_button(Layer::Ui),
            CleanupSettings,
        ))
        .id();

    let scanlines = cmds
        .spawn((
            Position::new_f32(6., 5., 0.),
            ActivatableBuilder::new("", callbacks.toggle_scanlines)
                .with_hotkey(KeyCode::Key2)
                .with_focus_order(1100)
                .as_button(Layer::Ui),
            CleanupSettings,
        ))
        .id();

    let film_grain = cmds
        .spawn((
            Position::new_f32(6., 5.5, 0.),
            ActivatableBuilder::new("", callbacks.toggle_film_grain)
                .with_hotkey(KeyCode::Key3)
                .with_focus_order(1200)
                .as_button(Layer::Ui),
            CleanupSettings,
        ))
        .id();

    let flicker = cmds
        .spawn((
            Position::new_f32(6., 6., 0.),
            ActivatableBuilder::new("", callbacks.toggle_flicker)
                .with_hotkey(KeyCode::Key4)
                .with_focus_order(1300)
                .as_button(Layer::Ui),
            CleanupSettings,
        ))
        .id();

    let vignette = cmds
        .spawn((
            Position::new_f32(6., 6.5, 0.),
            ActivatableBuilder::new("", callbacks.toggle_vignette)
                .with_hotkey(KeyCode::Key5)
                .with_focus_order(1400)
                .as_button(Layer::Ui),
            CleanupSettings,
        ))
        .id();

    let chromatic_ab = cmds
        .spawn((
            Position::new_f32(6., 7., 0.),
            ActivatableBuilder::new("", callbacks.toggle_chromatic_ab)
                .with_hotkey(KeyCode::Key6)
                .with_focus_order(1500)
                .as_button(Layer::Ui),
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
            Position::new_f32(6., 9., 0.),
            ActivatableBuilder::new("", callbacks.toggle_camera_mode)
                .with_hotkey(KeyCode::Key7)
                .with_focus_order(2000)
                .as_button(Layer::Ui),
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
            Position::new_f32(6., 11., 0.),
            ActivatableBuilder::new("", callbacks.toggle_saves)
                .with_hotkey(KeyCode::Key8)
                .with_focus_order(3000)
                .as_button(Layer::Ui),
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
        Position::new_f32(4., 15., 0.),
        ActivatableBuilder::new("({R|ESC}) BACK TO MAIN MENU", callbacks.back_to_menu)
            .with_hotkey(KeyCode::Escape)
            .with_audio(AudioKey::ButtonBack1)
            .with_focus_order(9000)
            .as_button(Layer::Ui),
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

fn update_settings_display(
    mut q_button: Query<&mut Button>,
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
    if let Ok(mut button) = q_button.get_mut(ui_entities.curvature) {
        button.set_label(match settings.crt_curvature {
            CrtCurvature::Off => "({Y|1}) Curvature: {R|OFF}".to_string(),
            CrtCurvature::Curve(x, y) => format!("({{Y|1}}) Curvature: {{G|{:.1}, {:.1}}}", x, y),
        });
    }

    // Update scanlines
    if let Ok(mut button) = q_button.get_mut(ui_entities.scanlines) {
        button.set_label(format!(
            "({{Y|2}}) Scanlines: {}",
            if settings.crt_scanline {
                "{G|ON}"
            } else {
                "{R|OFF}"
            }
        ));
    }

    // Update film grain
    if let Ok(mut button) = q_button.get_mut(ui_entities.film_grain) {
        button.set_label(format!(
            "({{Y|3}}) Film Grain: {}",
            if settings.crt_film_grain {
                "{G|ON}"
            } else {
                "{R|OFF}"
            }
        ));
    }

    // Update flicker
    if let Ok(mut button) = q_button.get_mut(ui_entities.flicker) {
        button.set_label(format!(
            "({{Y|4}}) Flicker: {}",
            if settings.crt_flicker {
                "{G|ON}"
            } else {
                "{R|OFF}"
            }
        ));
    }

    // Update vignette
    if let Ok(mut button) = q_button.get_mut(ui_entities.vignette) {
        button.set_label(format!(
            "({{Y|5}}) Vignette: {}",
            if settings.crt_vignette {
                "{G|ON}"
            } else {
                "{R|OFF}"
            }
        ));
    }

    // Update chromatic aberration
    if let Ok(mut button) = q_button.get_mut(ui_entities.chromatic_ab) {
        button.set_label(format!(
            "({{Y|6}}) Chromatic Aberration: {}",
            if settings.crt_chromatic_ab {
                "{G|ON}"
            } else {
                "{R|OFF}"
            }
        ));
    }

    // Update camera mode
    if let Ok(mut button) = q_button.get_mut(ui_entities.camera_mode) {
        button.set_label(match settings.camera_mode {
            CameraMode::Snap => "({Y|7}) Camera Mode: {G|SNAP}".to_string(),
            CameraMode::Smooth(speed) => {
                format!("({{Y|7}}) Camera Mode: {{G|SMOOTH ({:.3})}}", speed)
            }
        });
    }

    // Update saves enabled
    if let Ok(mut button) = q_button.get_mut(ui_entities.saves_enabled) {
        button.set_label(format!(
            "({{Y|8}}) Saves Enabled: {}",
            if settings.enable_saves {
                "{G|ON}"
            } else {
                "{R|OFF}"
            }
        ));
    }

    if let Ok(mut text) = q_text.get_mut(ui_entities.save_name) {
        text.value = format!("({{Y|9}}) Save Name: {{G|{}}}", settings.save_name);
    }

    if let Ok(mut text) = q_text.get_mut(ui_entities.input_rate) {
        text.value = format!(
            "({{Y|↑}}/{{Y|↓}}) Input Rate: {{G|{:.3}}}",
            settings.input_delay
        );
    }

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

fn remove_settings_callbacks(mut cmds: Commands) {
    cmds.remove_resource::<SettingsCallbacks>();
}
