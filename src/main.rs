use bevy_ecs::prelude::*;
use cfg::TEXEL_SIZE_F32;
use common::{render_shapes, MacroquadColorable, Palette, Rectangle};
use ecs::{Time, render_fps, update_time};
use macroquad::{miniquad::PassAction, prelude::*};
use rendering::{create_render_target, get_render_target_size, load_tilesets, render_all, render_glyphs, render_text, update_render_target, Glyph, GlyphBatch, GlyphMaterial, Layers, Position, RenderLayer, Text};

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

    let glyph_texture_id = tilesets.glyph_texture.raw_miniquad_id();
    let font_body_texture_id = tilesets.font_body_texture.raw_miniquad_id();

    let mut world = World::new();
    let mut schedule_pre_update = Schedule::default();
    let mut schedule_update = Schedule::default();
    let mut schedule_post_update = Schedule::default();

    world.init_resource::<Time>();
    world.init_resource::<GlyphMaterial>();
    world.insert_resource(tilesets);
    world.insert_resource(Layers {
        ground: GlyphBatch::new(glyph_texture_id, 10000),
        text: GlyphBatch::new(font_body_texture_id, 10000),
    });

    schedule_pre_update.add_systems(update_time);
    schedule_update.add_systems((render_fps, render_shapes, render_text, render_glyphs));
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
        Rectangle::new(16 * 5, 24, Palette::Green),
        Position::new(4, 8),
    ));

    world.spawn((
        Text::new("Hello strangers. 0123456789"),
        Position::screen(0., 0.),
    ));
    world.spawn((
        Text::new("Hello strangers. 0123456789").bg(Palette::Cyan),
        Position::screen(0., 0.5),
    ));

    let mut render_target = create_render_target();

    loop {
        clear_background(Palette::Black.to_macroquad_color());

        render_target = update_render_target(render_target);

        schedule_pre_update.run(&mut world);
        schedule_update.run(&mut world);

        let ctx = unsafe { get_internal_gl().quad_context };

        // clear render target
        ctx.begin_pass(Some(render_target.render_pass.raw_miniquad_id()), PassAction::clear_color(0.0, 0.0, 0.0, 0.0));
        ctx.end_render_pass();

        // render glyphs etc
        ctx.begin_pass(Some(render_target.render_pass.raw_miniquad_id()), PassAction::Nothing);
        schedule_post_update.run(&mut world);
        ctx.end_render_pass();

        set_default_camera();
        gl_use_default_material();

        // draw final texture as double size
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

        macroquad_profiler::profiler(Default::default());

        next_frame().await;
    }
}
