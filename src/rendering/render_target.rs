use macroquad::prelude::*;

use crate::cfg::{TEXEL_SIZE_F32, TILE_SIZE, TILE_SIZE_F32};

pub fn get_screen_size_texels() -> UVec2 {
    uvec2(
        (screen_width() / TEXEL_SIZE_F32) as u32,
        (screen_height() / TEXEL_SIZE_F32) as u32,
    )
}

pub fn get_render_target_size() -> UVec2 {
    uvec2(
        ((screen_width() / TEXEL_SIZE_F32) / TILE_SIZE_F32.0) as u32 * TILE_SIZE.0 as u32,
        ((screen_height() / TEXEL_SIZE_F32) / TILE_SIZE_F32.1) as u32 * TILE_SIZE.1 as u32,
    )
}

pub fn create_render_target() -> RenderTarget {
    let size = get_render_target_size();
    let target = render_target(size.x, size.y);
    target.texture.set_filter(FilterMode::Nearest);

    target
}
