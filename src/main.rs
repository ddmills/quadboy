use bevy_ecs::{event::EventRegistry, prelude::*};
use common::Palette;
use ecs::{Time, render_fps, update_time};
use engine::{CurrentState, KeyInput, update_key_input, update_states};
use macroquad::{prelude::*, telemetry};
use macroquad_profiler::ProfilerParams;
use rendering::{
    load_tilesets, render_all, render_glyphs, render_text, update_camera, update_screen_size, GameCamera, Glyph, Layers, Position, RenderTargets, RenderLayer, ScreenSize, Text
};
use ui::{update_ui_layout, UiLayout};

use crate::{cfg::WINDOW_SIZE, domain::{activate_zones_by_player, load_nearby_zones, on_load_zone, on_set_zone_status, on_unload_zone, player_input, render_player_debug, LoadZoneEvent, Player, PlayerDebug, PlayerMovedEvent, SetZoneStatusEvent, UnloadZoneEvent, Zones}, ecs::FpsDisplay, rendering::{on_zone_status_change, update_visibility, CrtShader}};

mod cfg;
mod common;
mod ecs;
mod engine;
mod rendering;
mod domain;
mod ui;

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

    let mut world = World::new();
    let mut schedule_pre_update = Schedule::default();
    let mut schedule_update = Schedule::default();
    let mut schedule_post_update = Schedule::default();

    world.insert_resource(tilesets);
    world.init_resource::<ScreenSize>();
    world.init_resource::<Time>();
    world.init_resource::<RenderTargets>();
    world.init_resource::<Layers>();
    world.init_resource::<KeyInput>();
    world.init_resource::<CurrentState>();
    world.init_resource::<GameCamera>();
    world.init_resource::<UiLayout>();
    world.init_resource::<CrtShader>();
    world.init_resource::<Zones>();

    EventRegistry::register_event::<LoadZoneEvent>(&mut world);
    EventRegistry::register_event::<UnloadZoneEvent>(&mut world);
    EventRegistry::register_event::<SetZoneStatusEvent>(&mut world);
    EventRegistry::register_event::<PlayerMovedEvent>(&mut world);

    schedule_pre_update.add_systems((
        update_time,
        update_key_input
    ));
    schedule_update.add_systems((
        (
            activate_zones_by_player,
            load_nearby_zones,
            on_load_zone,
            on_unload_zone,
            on_set_zone_status,
            on_zone_status_change,
        ).chain(),
        update_screen_size,
        update_ui_layout.run_if(resource_changed::<ScreenSize>),
        player_input,
        update_camera,
        render_fps,
        render_text,
        render_glyphs,
        render_player_debug,
    ));
    schedule_post_update.add_systems((
        (update_visibility, render_all, update_states).chain(),
    ));

    world.spawn((
        Position::new(15, 12),
        Glyph::new(4, Palette::Orange, Palette::Green)
            .layer(RenderLayer::Actors)
            .bg(Palette::White)
            .outline(Palette::Red)
    ));

    world.spawn((
        Position::new(10, 12),
        Glyph::new(147, Palette::Yellow, Palette::LightBlue)
            .layer(RenderLayer::Actors),
        Player,
    ));

    world.spawn((
        Text::new("123")
            .fg1(Palette::White)
            .bg(Palette::Purple)
            .layer(RenderLayer::Ui),
        Position::new_f32(0., 0.),
        FpsDisplay,
    ));

    world.spawn((
        Text::new("123")
            .fg1(Palette::White)
            .bg(Palette::Purple)
            .layer(RenderLayer::Ui),
        Position::new_f32(0., 0.5),
        PlayerDebug,
    ));

    loop {
        telemetry::begin_zone("schedule_pre_update");
        schedule_pre_update.run(&mut world);
        telemetry::end_zone();

        telemetry::begin_zone("schedule_update");
        schedule_update.run(&mut world);
        telemetry::end_zone();

        telemetry::begin_zone("schedule_post_update");
        schedule_post_update.run(&mut world);
        telemetry::end_zone();

        // macroquad_profiler::profiler(ProfilerParams {
        //     fps_counter_pos: vec2(0., 0.),
        // });

        next_frame().await;
    }
}
