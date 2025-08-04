use bevy_ecs::prelude::*;
use common::Palette;
use engine::{KeyInput, update_key_input, Time, render_fps, update_time};
use macroquad::prelude::*;
use rendering::{
    load_tilesets, render_all, render_glyphs, render_text, update_camera, update_screen_size, GameCamera, Glyph, Layers, Position, RenderTargets, RenderLayer, ScreenSize, Text
};
use ui::{update_ui_layout, UiLayout};

use crate::{cfg::WINDOW_SIZE, domain::{LoadZoneEvent, PlayerMovedEvent, SetZoneStatusEvent, SpawnZoneEvent, UnloadZoneEvent, Zones}, engine::{App, FpsDisplay, ScheduleType}, rendering::{update_visibility, CrtShader}, states::{update_states, CurrentState, MainMenuPlugin, PlayingStatePlugin}};

mod cfg;
mod common;
mod engine;
mod rendering;
mod domain;
mod ui;
mod states;

fn window_conf() -> Conf {
    Conf {
        window_title: "Quadboy".to_string(),
        window_width: WINDOW_SIZE.0 as i32,
        window_height: WINDOW_SIZE.1 as i32,
        fullscreen: false,
        window_resizable: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    set_default_filter_mode(FilterMode::Nearest);

    let tilesets = load_tilesets().await;

    let mut app = App::new();

    app
        .add_plugin(MainMenuPlugin)
        .add_plugin(PlayingStatePlugin)
        .register_event::<LoadZoneEvent>()
        .register_event::<UnloadZoneEvent>()
        .register_event::<SpawnZoneEvent>()
        .register_event::<SetZoneStatusEvent>()
        .register_event::<PlayerMovedEvent>()
        .insert_resource(tilesets)
        .init_resource::<ScreenSize>()
        .init_resource::<Time>()
        .init_resource::<RenderTargets>()
        .init_resource::<Layers>()
        .init_resource::<KeyInput>()
        .init_resource::<CurrentState>()
        .init_resource::<GameCamera>()
        .init_resource::<UiLayout>()
        .init_resource::<CrtShader>()
        .init_resource::<Zones>()
        .add_systems(ScheduleType::PreUpdate, (
            update_time,
            update_key_input
        ))
        .add_systems(ScheduleType::Update, (
            update_screen_size,
            render_fps,
            render_text,
            render_glyphs,
        ))
        .add_systems(ScheduleType::PostUpdate, update_visibility)
        .add_systems(ScheduleType::FrameFinal, (
            (
                render_all,
                update_states,
                // render_profiler,
            ).chain(),
        ));

    let world = app.get_world_mut();

    world.spawn((
        Text::new("123")
            .fg1(Palette::White)
            .bg(Palette::Purple)
            .layer(RenderLayer::Ui),
        Position::new_f32(0., 0., 0.),
        FpsDisplay,
    ));

    loop {
        app.run();
        next_frame().await;
    }
}
