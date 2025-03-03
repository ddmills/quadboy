use bevy_ecs::prelude::*;
use cfg::TILE_SIZE_F32;
use common::{render_shapes, MacroquadColorable, Palette, Rectangle};
use ecs::{Time, render_fps, update_time};
use macroquad::{miniquad::{PassAction, UniformsSource}, prelude::*};
use rendering::{create_render_camera, create_render_target, get_render_target_size, load_tilesets, render_all, render_glyphs, render_text, update_render_camera, update_render_target, BaseShaderUniforms, Glyph, GlyphMaterial, Position, Renderable, Renderer, Stage, Text, TEXEL_SIZE, TEXEL_SIZE_F32};

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

    let mut stage = {
        let InternalGlContext {
            quad_context: ctx, ..
        } = unsafe { get_internal_gl() };

        Stage::new(ctx, id)
    };

    let mut query = world.query::<(&Glyph, &Position)>();
    for (glyph, pos) in query.iter(&world) {
        let style = glyph.get_style();

        let x = pos.x * TILE_SIZE_F32.0;
        let y = pos.y * TILE_SIZE_F32.1;

        stage.add(Renderable {
            idx: glyph.idx,
            fg1: style.fg1,
            fg2: style.fg2,
            bg: style.bg,
            outline: style.outline,
            tileset_id: rendering::TilesetId::BodyFont,
            x,
            y,
            w: TILE_SIZE_F32.0,
            h: TILE_SIZE_F32.1,
        });
    }

    stage.add(Renderable {
        idx: 5,
        fg1: RED,
        fg2: BLUE,
        bg: WHITE,
        outline: GOLD,
        tileset_id: rendering::TilesetId::BodyFont,
        x: 0.,
        y: 0.,
        w: TILE_SIZE_F32.0,
        h: TILE_SIZE_F32.1,
    });

    let ss = get_render_target_size().as_vec2();
    let p = Mat4::orthographic_rh_gl(0., ss.x, ss.y, 0., 0., 1.);

    let out = p.mul_vec4(vec4(24., 24., 0., 1.));

    {
        let InternalGlContext {
            quad_context: ctx, ..
        } = unsafe { get_internal_gl() };

        stage.update_buffers(ctx);
    }

    loop {

        render_target = update_render_target(render_target);
        update_render_camera(&mut render_camera, &render_target);

        clear_background(Palette::Black.to_macroquad_color());
        // set_camera(&render_camera);

        draw_rectangle(-0.3, 0.3, 0.2, 0.2, GREEN);

        schedule_pre_update.run(&mut world);
        schedule_update.run(&mut world);

        {
            let mut gl = unsafe { get_internal_gl() };

            // make sure anything currently in the draw queue is rendered
            gl.flush();

            gl.quad_context.apply_pipeline(&stage.pipeline);
            gl.quad_context.apply_bindings(&stage.bindings);
            gl.quad_context.begin_default_pass(PassAction::Nothing);

            let screen_size = get_render_target_size().as_vec2();
            let projection = Mat4::orthographic_rh_gl(0., screen_size.x, screen_size.y, 0., 0., 1.);

            gl.quad_context.apply_uniforms(UniformsSource::table(
                &BaseShaderUniforms {
                    projection,
                },
            ));
            gl.quad_context.draw(0, stage.indices.len() as i32, 1);
            gl.quad_context.end_render_pass();
        }

        set_default_camera();
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
