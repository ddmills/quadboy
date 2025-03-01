use bevy_ecs::prelude::*;
use ecs::{render_fps, update_time, Time};
use macroquad::prelude::*;
use common::Palette;
use rendering::GlyphMaterial;

mod ecs;
mod common;
mod rendering;

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
    let mut world = World::new();

    world.init_resource::<Time>();
    world.init_resource::<GlyphMaterial>();

    let mut schedule_pre_update = Schedule::default();
    let mut schedule_update = Schedule::default();

    schedule_pre_update.add_systems(update_time);
    schedule_update.add_systems(render_fps);

    loop {
        clear_background(Palette::Black.into());

        draw_line(40.0, 40.0, 100.0, 200.0, 15.0, Palette::Blue.into());
        draw_rectangle(screen_width() / 2.0 - 60.0, 100.0, 120.0, 60.0, Palette::Green.into());
        draw_circle(screen_width() - 30.0, screen_height() - 30.0, 15.0, Palette::Yellow.into());

        schedule_pre_update.run(&mut world);
        schedule_update.run(&mut world);

        next_frame().await;
    }
}
