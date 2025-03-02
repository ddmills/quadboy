use bevy_ecs::prelude::*;
use cfg::{TILE_SIZE, TILE_SIZE_F32};
use common::{render_shapes, MacroquadColorable, Palette, Rectangle};
use ecs::{Time, render_fps, update_time};
use macroquad::prelude::*;
use rendering::{create_render_camera, create_render_target, get_render_target_size, load_tilesets, render_all, render_glyphs, render_text, update_render_camera, update_render_target, Glyph, GlyphMaterial, Position, Renderer, Text, TEXEL_SIZE, TEXEL_SIZE_F32};

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
    for y in 0..32 {
        for x in 0..32 {
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

    loop {
        render_target = update_render_target(render_target);
        update_render_camera(&mut render_camera, &render_target);

        set_camera(&render_camera);

        clear_background(Palette::Black.to_macroquad_color());

        schedule_pre_update.run(&mut world);
        schedule_update.run(&mut world);

        set_default_camera();

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
