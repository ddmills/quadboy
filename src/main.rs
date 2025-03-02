use bevy_ecs::prelude::*;
use common::{render_shapes, MacroquadColorable, Palette, Rectangle};
use ecs::{Time, render_fps, update_time};
use macroquad::prelude::*;
use rendering::{create_render_camera, create_render_target, get_render_target_size, load_tilesets, render_all, render_glyphs, render_text, update_render_camera, update_render_target, BaseShaderUniforms, Glyph, GlyphMaterial, Position, Renderer, Stage, Text, TEXEL_SIZE, TEXEL_SIZE_F32};

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

    let id = tilesets.font_body_texture.raw_miniquad_id();

    let mut world = World::new();

    world.init_resource::<Time>();
    world.init_resource::<GlyphMaterial>();
    world.init_resource::<Renderer>();
    world.insert_resource(tilesets);

    let mut schedule_pre_update = Schedule::default();
    let mut schedule_update = Schedule::default();

    schedule_pre_update.add_systems(update_time);

    schedule_update.add_systems(render_fps);
    schedule_update.add_systems((render_shapes, render_glyphs, render_text, render_all).chain());

    let mut idx = 0;
    for y in 0..16 {
        for x in 0..16 {
            world.spawn((
                Position::new(x, y),
                Glyph::new(idx % 255, Palette::Red, Palette::Yellow),
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

    let stage = {
        let InternalGlContext {
            quad_context: ctx, ..
        } = unsafe { get_internal_gl() };
        
        
        Stage::new(ctx, id)
    };

    loop {

        render_target = update_render_target(render_target);
        update_render_camera(&mut render_camera, &render_target);

        set_camera(&render_camera);
        // set_camera(&Camera2D {
        //     zoom: vec2(1., screen_width() / screen_height()),
        //     ..Default::default()
        // });
        // clear_background(Palette::Black.to_macroquad_color());

        draw_line(-0.4, 0.4, -0.8, 0.9, 0.05, BLUE);
        draw_rectangle(-0.3, 0.3, 0.2, 0.2, GREEN);
        draw_circle(0., 0., 40.1, YELLOW);

        // schedule_pre_update.run(&mut world);
        // schedule_update.run(&mut world);

        {
            let mut gl = unsafe { get_internal_gl() };

            // make sure anything currently in the draw queue is rendered
            gl.flush();


            let t = get_time();

            gl.quad_context.apply_pipeline(&stage.pipeline);
            gl.quad_context.begin_default_pass(miniquad::PassAction::Nothing);
            gl.quad_context.apply_bindings(&stage.bindings);

            for i in 0..10 {
                let t = t + i as f64 * 0.3;

                gl.quad_context.apply_uniforms(miniquad::UniformsSource::table(
                    &BaseShaderUniforms {
                        offset: (t.sin() as f32 * 0.5, (t * 3.).cos() as f32 * 0.5),
                    },
                ));
                gl.quad_context.draw(0, 6, 1);
            }

            gl.quad_context.end_render_pass();
        }

        set_default_camera();
        draw_text("HELLO", 30.0, 200.0, 30.0, RED);

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
