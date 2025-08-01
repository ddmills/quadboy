use bevy_ecs::prelude::*;
use macroquad::{miniquad::PassAction, prelude::*, telemetry};

use crate::{
    cfg::{TEXEL_SIZE_F32, TILE_SIZE, TILE_SIZE_F32}, common::{MacroquadColorable, Palette}, rendering::{CrtShader, GameCamera}, ui::UiLayout
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
}

pub fn render_all(
    mut layers: ResMut<Layers>,
    mut ren: ResMut<RenderTargets>,
    camera: Res<GameCamera>,
    screen: Res<ScreenSize>,
    crt: Res<CrtShader>,
    ui: Res<UiLayout>
) {
    telemetry::begin_zone("render_all");
    let target_size = uvec2(
        (camera.get_width_world().floor() * TILE_SIZE_F32.0) as u32,
        (camera.get_height_world().floor() * TILE_SIZE_F32.1) as u32
    );

    if ren.world.texture.size().as_uvec2() != target_size {
        ren.world = create_render_target();
        ren.screen = create_render_target();
    }

    clear_background(Color::from_hex(0x141414));

    start_pass(&ren.world);
    layers.ground.render();
    layers.actors.render();
    end_pass();
    
    start_pass(&ren.screen);
    layers.ui.render();
    end_pass();

    // draw final texture as double size
    let dest_size: macroquad::prelude::Vec2 = target_size.as_vec2() * TEXEL_SIZE_F32;
    crt.mat.set_uniform("iTime", get_time() as f32);
    crt.mat.set_uniform("iResolution", (dest_size.x, dest_size.y));
    gl_use_material(&crt.mat);

    let x = (screen.width - target_size.x) as f32;
    let y = (screen.height - target_size.y) as f32;
    draw_rectangle(x, y, target_size.x as f32 * TEXEL_SIZE_F32, target_size.y as f32 * TEXEL_SIZE_F32, Palette::Clear.to_macroquad_color());

    draw_texture_ex(
        &ren.world.texture,
        x + (ui.game_panel.x * TILE_SIZE.0) as f32 * TEXEL_SIZE_F32,
        y + (ui.game_panel.y * TILE_SIZE.1) as f32 * TEXEL_SIZE_F32,
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
    telemetry::end_zone();
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
