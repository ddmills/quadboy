use bevy_ecs::prelude::*;
use macroquad::{miniquad::PassAction, prelude::*};

use crate::{
    cfg::TEXEL_SIZE_F32,
    common::{MacroquadColorable, Palette},
};

use super::{create_render_target, Layers, ScreenSize};

#[derive(Resource)]
pub struct MainRenderTarget {
    pub target: RenderTarget,
}

impl Default for MainRenderTarget {
    fn default() -> Self {
        MainRenderTarget {
            target: create_render_target(),
        }
    }
}

pub struct Renderable {
    pub idx: usize,
    pub fg1: Vec4,
    pub fg2: Vec4,
    pub bg: Vec4,
    pub outline: Vec4,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

pub fn render_all(mut layers: ResMut<Layers>, mut ren: ResMut<MainRenderTarget>, screen: Res<ScreenSize>) {
    let target_size = uvec2(screen.width, screen.height);

    if ren.target.texture.size().as_uvec2() != target_size {
        ren.target = create_render_target();
    }

    clear_background(Palette::Black.to_macroquad_color());

    let ctx = unsafe { get_internal_gl().quad_context };

    // clear render target
    ctx.begin_pass(
        Some(ren.target.render_pass.raw_miniquad_id()),
        PassAction::clear_color(0.0, 0.0, 0.0, 0.0),
    );
    ctx.end_render_pass();

    // render glyphs etc
    ctx.begin_pass(
        Some(ren.target.render_pass.raw_miniquad_id()),
        PassAction::Nothing,
    );

    layers.ground.render();
    layers.text.render();

    ctx.end_render_pass();

    set_default_camera();
    gl_use_default_material();

    // draw final texture as double size
    let dest_size = target_size.as_vec2() * TEXEL_SIZE_F32;

    draw_texture_ex(
        &ren.target.texture,
        0.,
        0.,
        WHITE,
        DrawTextureParams {
            dest_size: Some(dest_size),
            flip_y: true,
            ..Default::default()
        },
    );
}
