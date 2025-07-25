use bevy_ecs::prelude::*;
use macroquad::{miniquad::PassAction, prelude::*};

use crate::{
    common::{MacroquadColorable, Palette}, rendering::GlyphBatch,
};

use super::{create_render_target, Layers, ScreenSize};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RenderTargetType {
    World,
    Screen,
}

#[derive(Resource)]
pub struct RenderTargets {
    pub world: RenderTarget,
    pub screen: RenderTarget,
}

impl Default for RenderTargets {
    fn default() -> Self {
        RenderTargets {
            world: create_render_target(),
            screen: create_render_target(),
        }
    }
}

impl RenderTargets {
    pub fn get(&mut self, target_type: RenderTargetType) -> &mut RenderTarget {
        match target_type {
            RenderTargetType::World => &mut self.world,
            RenderTargetType::Screen => &mut self.screen,
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
        ren.screen = create_render_target();
    }

    clear_background(Palette::Black.to_macroquad_color());

    start_pass(&ren.world);
    layers.ground.render();
    end_pass();
    
    start_pass(&ren.screen);
    layers.ui.render();
    layers.text.render();
    end_pass();

    set_default_camera();
    gl_use_default_material();
}

fn start_pass(target: &RenderTarget)
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
}

fn end_pass()
{
    let ctx = unsafe { get_internal_gl().quad_context };
    
    ctx.end_render_pass();
}
