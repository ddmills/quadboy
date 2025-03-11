use bevy_ecs::prelude::*;
use common::{render_shapes, MacroquadColorable, Palette, Rectangle};
use ecs::{Time, render_fps, update_time};
use macroquad::{miniquad::PassAction, prelude::*, telemetry::{self, ZoneGuard}};
use rendering::{create_render_camera, create_render_target, get_render_target_size, load_tilesets, render_all, render_glyphs, render_text, update_render_camera, update_render_target, BaseShaderUniforms, Glyph, GlyphBatch, GlyphMaterial, Position, Renderable, Text, TEXEL_SIZE, TEXEL_SIZE_F32};

mod common;
mod ecs;
mod rendering;
mod cfg;

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

    let id = tilesets.glyph_texture.raw_miniquad_id();

    let mut world = World::new();

    world.init_resource::<Time>();
    world.init_resource::<GlyphMaterial>();
    world.insert_resource(tilesets);

    let mut schedule_pre_update = Schedule::default();
    let mut schedule_update = Schedule::default();
    let mut schedule_post_update = Schedule::default();

    schedule_pre_update.add_systems(update_time);
    schedule_update.add_systems((render_fps, render_shapes));
    schedule_update.add_systems((render_glyphs));
    schedule_post_update.add_systems(render_all);

    let mut idx = 0;
    for y in 0..128 {
        for x in 0..128 {
            world.spawn((
                Position::new(x, y),
                Glyph::new(idx % 256, Palette::Yellow, Palette::Red),
            ));
            idx += 1;
        }
    }

    world.spawn((
        Rectangle::new(16 * 5, 24, Palette::Green),
        Position::new(4, 8),
    ));

    world.spawn((
        Text::new("Hello strangers. 0123456789"),
        Position::screen(0., 1.),
    ));

    let mut render_target = create_render_target();
    let mut render_camera = create_render_camera(&render_target);

    world.spawn((GlyphBatch::new(id)));

    loop {
        clear_background(Palette::Purple.to_macroquad_color());

        // resize the render target to half the current screen size
        render_target = update_render_target(render_target);

        // update_render_camera(&mut render_camera, &render_target);
        // set_camera(&render_camera);

        schedule_pre_update.run(&mut world);
        schedule_update.run(&mut world);

        {
            let _z = ZoneGuard::new("glyph-render-pass");

            {
                let gl = unsafe { get_internal_gl() };
                gl.quad_context.begin_pass(Some(render_target.render_pass.raw_miniquad_id()), PassAction::Nothing);
            }

            // will call "render" for the glyph batch
            schedule_post_update.run(&mut world);

            {
                let gl = unsafe { get_internal_gl() };
                gl.quad_context.end_render_pass();
            }
        }

        set_default_camera();
        // update_render_camera(&mut render_camera, &render_target);
        // set_camera(&render_camera);

        clear_background(Palette::Black.to_macroquad_color());

        let target_size = get_render_target_size();
        let dest_size = target_size.as_vec2() * TEXEL_SIZE_F32;

        draw_texture_ex(
            &render_target.texture,
            0.,
            0.,
            WHITE,
            DrawTextureParams {
                dest_size: Some(dest_size),
                flip_y: true,
                ..Default::default()
            },
        );

        let t = get_fps().to_string();
        draw_text(&t, 16.0, 32.0, 16.0, WHITE);

        gl_use_default_material();

        macroquad_profiler::profiler(Default::default());

        next_frame().await;
    }
}
