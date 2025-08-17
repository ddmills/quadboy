use web_sys;
use bevy_ecs::prelude::*;
use common::Palette;
use engine::{KeyInput, Time, render_fps, update_key_input, update_time};
use macroquad::{miniquad::conf::{Platform, WebGLVersion}, prelude::*};
use rendering::{
    GameCamera, Layers, Position, RenderLayer, RenderTargets, ScreenSize, Text, load_tilesets,
    render_all, render_glyphs, render_text, update_screen_size,
};
use ui::UiLayout;

use crate::{
    cfg::WINDOW_SIZE,
    domain::{
        LoadZoneEvent, PlayerMovedEvent, SetZoneStatusEvent, SpawnZoneEvent, UnloadZoneEvent, Zones,
    },
    engine::{App, ExitAppPlugin, FpsDisplay, ScheduleType, SerializableComponentRegistry},
    rendering::{CrtShader, Glyph, TrackZone, update_visibility},
    states::{
        CurrentAppState, CurrentGameState, ExploreStatePlugin, MainMenuStatePlugin,
        PauseStatePlugin, PlayStatePlugin, update_app_states, update_game_states,
    },
};

mod cfg;
mod common;
mod domain;
mod engine;
mod rendering;
mod states;
mod ui;

fn window_conf() -> Conf {
    Conf {
        window_title: "Quadboy".to_string(),
        window_width: WINDOW_SIZE.0 as i32,
        window_height: WINDOW_SIZE.1 as i32,
        fullscreen: false,
        window_resizable: true,
        platform: Platform {
            webgl_version: WebGLVersion::WebGL2,
            ..Default::default()
        },
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    trace!("HELLO WORLD");
    set_default_filter_mode(FilterMode::Nearest);

    let tilesets = load_tilesets().await;

    let mut app = App::new();

    let mut reg = SerializableComponentRegistry::new();
    reg.register::<Position>();
    reg.register::<TrackZone>();
    reg.register::<Glyph>();

    app.add_plugin(ExitAppPlugin)
        .add_plugin(MainMenuStatePlugin)
        .add_plugin(PlayStatePlugin)
        .add_plugin(ExploreStatePlugin)
        .add_plugin(PauseStatePlugin)
        .register_event::<LoadZoneEvent>()
        .register_event::<UnloadZoneEvent>()
        .register_event::<SpawnZoneEvent>()
        .register_event::<SetZoneStatusEvent>()
        .register_event::<PlayerMovedEvent>()
        .insert_resource(tilesets)
        .insert_resource(reg)
        .init_resource::<ScreenSize>()
        .init_resource::<Time>()
        .init_resource::<RenderTargets>()
        .init_resource::<Layers>()
        .init_resource::<KeyInput>()
        .init_resource::<CurrentAppState>()
        .init_resource::<CurrentGameState>()
        .init_resource::<GameCamera>()
        .init_resource::<UiLayout>()
        .init_resource::<CrtShader>()
        .init_resource::<Zones>()
        .add_systems(ScheduleType::PreUpdate, (update_time, update_key_input))
        .add_systems(
            ScheduleType::Update,
            (update_screen_size, render_fps, render_text, render_glyphs),
        )
        .add_systems(ScheduleType::PostUpdate, update_visibility)
        .add_systems(
            ScheduleType::FrameFinal,
            ((
                render_all,
                update_app_states,
                update_game_states,
                // render_profiler,
            )
                .chain(),),
        );

    let world = app.get_world_mut();

    world.spawn((
        Text::new("123")
            .fg1(Palette::White)
            .bg(Palette::Purple)
            .layer(RenderLayer::Ui),
        Position::new_f32(0., 0., 0.),
        FpsDisplay,
    ));

    while app.run() {
        next_frame().await;
    }
}
