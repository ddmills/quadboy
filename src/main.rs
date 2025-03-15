use bevy_ecs::prelude::*;
use common::Palette;
use ecs::{Time, render_fps, update_time};
use engine::{update_key_input, KeyInput};
use macroquad::prelude::*;
use rendering::{
    load_tilesets, render_all, render_glyphs, render_text, update_camera, GameCamera, Glyph, Layers, Position, RenTarget, RenderLayer, Text
};

mod cfg;
mod common;
mod ecs;
mod rendering;
mod engine;

fn window_conf() -> Conf {
    Conf {
        window_title: "Quadboy".to_string(),
        window_width: 800,
        window_height: 600,
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
    world.init_resource::<Time>();
    world.init_resource::<RenTarget>();
    world.init_resource::<Layers>();
    world.init_resource::<KeyInput>();
    // world.init_resource::<GameCamera>();
    world.insert_resource(GameCamera {x: 1., y: 1. });

    schedule_pre_update.add_systems((update_time, update_key_input));
    schedule_update.add_systems((update_camera, render_fps, render_text, render_glyphs));
    schedule_post_update.add_systems(render_all);

    let mut idx = 0;

    for y in 0..128 {
        for x in 0..128 {
            world.spawn((
                Position::new(x, y),
                Glyph::new(idx % 256, Palette::Purple, Palette::Green).layer(RenderLayer::Ground),
            ));
            idx += 1;
        }
    }

    world.spawn((
        Text::new("Hello strangers. 0123456789"),
        Position::new_f32(0., 0.),
    ));
    world.spawn((
        Text::new("Hello strangers. 0123456789").bg(Palette::Cyan),
        Position::new_f32(0., 0.5),
    ));

    loop {
        schedule_pre_update.run(&mut world);
        schedule_update.run(&mut world);
        schedule_post_update.run(&mut world);

        let t = get_fps().to_string();
        draw_text(&t, 16.0, 32.0, 16.0, WHITE);

        macroquad_profiler::profiler(Default::default());

        next_frame().await;
    }
}
