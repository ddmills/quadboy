use macroquad::prelude::*;

pub const TEXEL_SIZE:u32 = 2;
pub const TEXEL_SIZE_F32:f32 = TEXEL_SIZE as f32;

pub fn get_render_target_size() -> IVec2 {
    ivec2(
        (screen_width() / TEXEL_SIZE_F32) as i32,
        (screen_height() / TEXEL_SIZE_F32) as i32,
    )
}

pub fn get_render_offset() -> Vec2 {
    let target_size = get_render_target_size().as_vec2();

    (target_size % 2.0) * 0.5
}

pub fn create_render_target() -> RenderTarget {
    let pref_size = get_render_target_size();
    let target = render_target(pref_size.x as u32, pref_size.y as u32);
    target.texture.set_filter(FilterMode::Nearest);

    target
}

pub fn update_render_target(target: RenderTarget) -> RenderTarget {
    let pref_size: IVec2 = get_render_target_size();

    if target.texture.size().as_ivec2() != pref_size {
        create_render_target()
    } else {
        target
    }
}

pub fn create_render_camera(target: &RenderTarget) -> Camera2D
{
    let pref_size = get_render_target_size().as_vec2();

    Camera2D {
        zoom: vec2(1. / pref_size.x * 2., 1. / pref_size.y * 2.),
        target: vec2(
            (pref_size.x * 0.5f32).floor(),
            (pref_size.y * 0.5f32).floor(),
        ),
        render_target: Some(target.clone()),
        ..Default::default()
    }
}

pub fn update_render_camera(camera: &mut Camera2D, target: &RenderTarget)
{
    let pref_size = get_render_target_size().as_vec2();

    camera.zoom.x = 1. / pref_size.x * 2.;
    camera.zoom.y = 1. / pref_size.y * 2.;

    camera.target.x = (pref_size.x * 0.5f32).floor();
    camera.target.y = (pref_size.y * 0.5f32).floor();

    camera.render_target = Some(target.clone());
}
