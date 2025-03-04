use bevy_ecs::prelude::*;
use cfg::TILE_SIZE_F32;
use common::{render_shapes, MacroquadColorable, Palette, Rectangle};
use ecs::{Time, render_fps, update_time};
use macroquad::{miniquad::{PassAction, UniformsSource}, prelude::*};
use rendering::{create_render_camera, create_render_target, get_render_target_size, load_tilesets, render_all, render_glyphs, render_text, update_render_camera, update_render_target, BaseShaderUniforms, Glyph, GlyphMaterial, Position, Renderable, GlyphBatch, Text, TEXEL_SIZE, TEXEL_SIZE_F32};

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
    schedule_update.add_systems(render_fps);
    schedule_update.add_systems((render_shapes, render_glyphs, render_text).chain());
    schedule_post_update.add_systems(render_all);

    let mut idx = 0;
    for y in 0..160 {
        for x in 0..160 {
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

    let glyph_batch_id = world.spawn(GlyphBatch::new(id)).id();

    loop {

        render_target = update_render_target(render_target);
        update_render_camera(&mut render_camera, &render_target);

        clear_background(Palette::Black.to_macroquad_color());
        // set_camera(&render_camera);

        schedule_pre_update.run(&mut world);
        schedule_update.run(&mut world);
        schedule_post_update.run(&mut world);

        let mut q = world.query::<&mut GlyphBatch>();
        let mut glyph_batch = q.single_mut(&mut world);

        {
            let gl = unsafe { get_internal_gl() };
            gl.quad_context.begin_pass(Some(render_target.render_pass.raw_miniquad_id()), PassAction::Nothing);
        }

        glyph_batch.render();
        
        {
            let gl = unsafe { get_internal_gl() };
            gl.quad_context.end_render_pass();
        }

        set_default_camera();

        clear_background(Palette::Black.to_macroquad_color());
        let t = get_fps().to_string();
        draw_text(&t, 30.0, 200.0, 30.0, Palette::LightGreen.to_macroquad_color());

        let target_size = get_render_target_size();
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
