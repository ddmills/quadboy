use bevy_ecs::prelude::*;
use common::Palette;
use ecs::{Time, render_fps, update_time};
use macroquad::prelude::*;
use rendering::{create_render_camera, create_render_target, get_render_target_size, load_tilesets, render_glyphs, update_render_camera, update_render_target, Glyph, GlyphMaterial, Position, TEXEL_SIZE, TEXEL_SIZE_F32};

mod common;
mod ecs;
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
    set_default_filter_mode(FilterMode::Nearest);

    let tilesets = load_tilesets().await;

    let mut world = World::new();

    world.init_resource::<Time>();
    world.init_resource::<GlyphMaterial>();
    world.insert_resource(tilesets);

    let mut schedule_pre_update = Schedule::default();
    let mut schedule_update = Schedule::default();

    schedule_pre_update.add_systems(update_time);

    schedule_update.add_systems(render_fps);
    schedule_update.add_systems(render_glyphs);

    world.spawn((
        Position::new(100., 100.),
        Glyph::new(10, Palette::Red, Palette::Yellow),
    ));

    let mut render_target = create_render_target();
    let mut render_camera = create_render_camera(&render_target);

    loop {
        let target_size = get_render_target_size();
        let target_size_f32 = target_size.as_vec2();

        render_target = update_render_target(render_target);
        update_render_camera(&mut render_camera, &render_target);

        set_camera(&render_camera);

        clear_background(Palette::Black.into());

        draw_line(40.0, 40.0, 100.0, 200.0, 15.0, Palette::Blue.into());
        draw_rectangle(
            screen_width() / 2.0 - 60.0,
            100.0,
            120.0,
            60.0,
            Palette::Green.into(),
        );
        draw_circle(
            screen_width() - 30.0,
            screen_height() - 30.0,
            15.0,
            Palette::Yellow.into(),
        );

        schedule_pre_update.run(&mut world);
        schedule_update.run(&mut world);

        set_default_camera();
        clear_background(ORANGE);

        let dest_size = target_size.as_vec2() * vec2(TEXEL_SIZE_F32, TEXEL_SIZE_F32);

        draw_texture_ex(
            &render_target.texture,
            0.,
            0.,
            WHITE,
            DrawTextureParams {
                dest_size: Some(dest_size),
                ..Default::default()
            },
        );

        gl_use_default_material();

        next_frame().await;
    }
}
