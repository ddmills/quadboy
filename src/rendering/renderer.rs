use bevy_ecs::prelude::*;
use macroquad::{miniquad::PassAction, prelude::*};

use crate::{
    common::{MacroquadColorable, Palette}, rendering::GlyphBatch,
};

use super::{create_render_target, Layers, ScreenSize};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RenderTargetType {
    World,
    Ui,
}

#[derive(Resource)]
pub struct RenderTargets {
    pub world: RenderTarget,
    pub ui: RenderTarget,
}

impl Default for RenderTargets {
    fn default() -> Self {
        RenderTargets {
            world: create_render_target(),
            ui: create_render_target(),
        }
    }
}

impl RenderTargets {
    pub fn get(&mut self, target_type: RenderTargetType) -> &mut RenderTarget {
        match target_type {
            RenderTargetType::World => &mut self.world,
            RenderTargetType::Ui => &mut self.ui,
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

pub fn render_all(mut layers: ResMut<Layers>, mut ren: ResMut<RenderTargets>, screen: Res<ScreenSize>) {
    let target_size = uvec2(screen.width, screen.height);

    if ren.world.texture.size().as_uvec2() != target_size {
        ren.world = create_render_target();
        ren.ui = create_render_target();
    }

    clear_background(Palette::Black.to_macroquad_color());

    let target = ren.get(layers.ground.target_type);

    ctx_render_layer(target, &mut layers.ground);
    ctx_render_layer(&ren.ui, &mut layers.text);

    set_default_camera();
    gl_use_default_material();
}

fn ctx_render_layer(target: &RenderTarget, glyphs: &mut GlyphBatch)
{
    let ctx = unsafe { get_internal_gl().quad_context };

    // clear render target
    ctx.begin_pass(
        Some(target.render_pass.raw_miniquad_id()),
        PassAction::clear_color(0.0, 0.0, 0.0, 0.0),
    );
    ctx.end_render_pass();

    // render glyphs etc
    ctx.begin_pass(
        Some(target.render_pass.raw_miniquad_id()),
        PassAction::Nothing,
    );

    glyphs.render();

    ctx.end_render_pass();
}