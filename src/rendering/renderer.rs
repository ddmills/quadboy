use bevy_ecs::prelude::*;
use macroquad::{miniquad::PassAction, prelude::*};

use crate::{
    cfg::{TEXEL_SIZE_F32, TILE_SIZE},
    engine::Time,
    rendering::{AmbientTransition, CrtShader},
    tracy_span,
};

use super::{Layers, ScreenSize, create_render_target};

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

pub struct Renderable {
    pub idx: usize,
    pub tex_idx: usize,
    pub fg1: Vec4,
    pub fg2: Vec4,
    pub bg: Vec4,
    pub outline: Vec4,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub is_shrouded: u32,
    pub light_rgba: Vec4,
    pub light_flicker: f32,
    pub ignore_lighting: f32,
}

pub fn render_all(
    mut layers: ResMut<Layers>,
    mut ren: ResMut<RenderTargets>,
    screen: Res<ScreenSize>,
    crt: Res<CrtShader>,
    time: Res<Time>,
    mut ambient_transition: ResMut<AmbientTransition>,
    lighting_data: Res<super::LightingData>,
) {
    tracy_span!("render_all");
    let target_size = uvec2(
        (screen.tile_w * TILE_SIZE.0) as u32,
        (screen.tile_h * TILE_SIZE.1) as u32,
    );

    if ren.world.texture.size().as_uvec2() != target_size {
        ren.world = create_render_target();
        ren.screen = create_render_target();
    }

    clear_background(Color::from_hex(0x0E0505));

    // Update ambient transition
    ambient_transition.update(time.dt);

    // Check if we need to start a new transition
    let current_ambient = lighting_data.get_ambient_vec4();
    ambient_transition.start_transition(current_ambient);

    // Use interpolated ambient instead of direct value
    let shader_time = get_time() as f32;
    let ambient = ambient_transition.get_interpolated_ambient();

    start_pass(&ren.world, ambient);
    layers.iter_mut().for_each(|l| {
        if l.target_type == RenderTargetType::World {
            l.render(shader_time, ambient);
        }
    });
    end_pass();

    start_pass(&ren.screen, macroquad::prelude::Vec4::splat(0.0)); // Screen layers use transparent
    layers.iter_mut().for_each(|l| {
        if l.target_type == RenderTargetType::Screen {
            l.render(shader_time, ambient);
        }
    });
    end_pass();

    // draw final texture as double size
    let dest_size: macroquad::prelude::Vec2 = target_size.as_vec2() * TEXEL_SIZE_F32;

    crt.mat.set_uniform("u_time", get_time() as f32);
    crt.mat
        .set_uniform("u_resolution", vec2(dest_size.x, dest_size.y));

    gl_use_material(&crt.mat);

    let x = (screen.width - target_size.x) as f32;
    let y = (screen.height - target_size.y) as f32;
    // Background rectangle no longer needed - render target now clears to ambient color

    draw_texture_ex(
        &ren.world.texture,
        x,
        y,
        WHITE,
        DrawTextureParams {
            dest_size: Some(dest_size),
            flip_y: true,
            ..Default::default()
        },
    );

    draw_texture_ex(
        &ren.screen.texture,
        x,
        y,
        WHITE,
        DrawTextureParams {
            dest_size: Some(dest_size),
            flip_y: true,
            ..Default::default()
        },
    );

    set_default_camera();
    gl_use_default_material();
}

fn start_pass(target: &RenderTarget, clear_color: macroquad::prelude::Vec4) {
    let ctx = unsafe { get_internal_gl().quad_context };

    // clear render target to ambient color
    ctx.begin_pass(
        Some(target.render_pass.raw_miniquad_id()),
        PassAction::clear_color(clear_color.x, clear_color.y, clear_color.z, clear_color.w),
    );
    ctx.end_render_pass();

    // render glyphs etc
    ctx.begin_pass(
        Some(target.render_pass.raw_miniquad_id()),
        PassAction::Nothing,
    );
}

fn end_pass() {
    let ctx = unsafe { get_internal_gl().quad_context };

    ctx.end_render_pass();
}
