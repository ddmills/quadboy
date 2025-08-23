use bevy_ecs::prelude::*;
use common::Palette;
use engine::{KeyInput, Time, render_fps, update_key_input, update_time};
use macroquad::{
    miniquad::conf::{Platform, WebGLVersion},
    prelude::*,
};
use rendering::{
    GameCamera, Layers, Position, RenderTargets, ScreenSize, Text, load_tilesets, render_all,
    render_glyphs, render_text, update_crt_uniforms, update_screen_size,
};
use ui::UiLayout;

use crate::{
    cfg::WINDOW_SIZE,
    common::Rand,
    domain::{
        ApplyVisibilityEffects, Bitmasker, Collider, ConsumeEnergyEvent, Energy, GameSettings,
        HideWhenNotVisible, Label, LoadGameResult, LoadZoneEvent, NewGameResult, Player,
        PlayerMovedEvent, Prefabs, RefreshBitmask, SaveGameResult, SetZoneStatusEvent, StairDown,
        StairUp, TurnState, UnloadZoneEvent, Vision, Zones, on_bitmask_spawn, on_refresh_bitmask,
    },
    engine::{
        App, Clock, ExitAppPlugin, FpsDisplay, Mouse, ScheduleType, SerializableComponentRegistry,
        update_mouse,
    },
    rendering::{CrtShader, Glyph, RecordZonePosition},
    states::{
        CleanupStateExplore, CleanupStatePlay, CurrentAppState, CurrentGameState,
        ExploreStatePlugin, LoadGameStatePlugin, MainMenuStatePlugin, NewGameStatePlugin,
        PauseStatePlugin, PlayStatePlugin, SettingsStatePlugin, update_app_states,
        update_game_states,
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
    set_default_filter_mode(FilterMode::Nearest);

    let tilesets = load_tilesets().await;

    let mut app = App::new();

    let mut reg = SerializableComponentRegistry::new();
    reg.register::<Position>();
    reg.register::<RecordZonePosition>();
    reg.register::<Glyph>();
    reg.register::<CleanupStatePlay>();
    reg.register::<CleanupStateExplore>();
    reg.register::<Label>();
    reg.register::<Collider>();
    reg.register::<Energy>();
    reg.register::<StairDown>();
    reg.register::<StairUp>();
    reg.register::<Player>();
    reg.register::<Vision>();
    reg.register::<ApplyVisibilityEffects>();
    reg.register::<HideWhenNotVisible>();

    app.add_plugin(ExitAppPlugin)
        .add_plugin(MainMenuStatePlugin)
        .add_plugin(SettingsStatePlugin)
        .add_plugin(PlayStatePlugin)
        .add_plugin(NewGameStatePlugin)
        .add_plugin(LoadGameStatePlugin)
        .add_plugin(ExploreStatePlugin)
        .add_plugin(PauseStatePlugin)
        .register_event::<LoadGameResult>()
        .register_event::<LoadZoneEvent>()
        .register_event::<NewGameResult>()
        .register_event::<UnloadZoneEvent>()
        .register_event::<SetZoneStatusEvent>()
        .register_event::<PlayerMovedEvent>()
        .register_event::<SaveGameResult>()
        .register_event::<ConsumeEnergyEvent>()
        .register_event::<RefreshBitmask>()
        .insert_resource(tilesets)
        .insert_resource(reg)
        .init_resource::<Mouse>()
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
        .init_resource::<GameSettings>()
        .init_resource::<TurnState>()
        .init_resource::<Clock>()
        .init_resource::<Bitmasker>()
        .init_resource::<Prefabs>()
        .init_resource::<Rand>()
        .add_systems(ScheduleType::PreUpdate, (update_time, update_key_input))
        .add_systems(
            ScheduleType::Update,
            (
                update_screen_size,
                update_mouse,
                update_crt_uniforms,
                render_fps,
                (
                    on_refresh_bitmask,
                    on_bitmask_spawn,
                    render_glyphs,
                    render_text,
                )
                    .chain(),
            ),
        )
        .add_systems(
            ScheduleType::FrameFinal,
            ((
                render_all,
                update_app_states,
                update_game_states,
                crate::engine::render_profiler,
            )
                .chain(),),
        );

    let world = app.get_world_mut();

    world.spawn((
        Text::new("123").bg(Palette::Black),
        Position::new_f32(0., 0., 0.),
        FpsDisplay,
    ));

    while app.run() {
        next_frame().await;
    }
}
