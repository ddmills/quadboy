use bevy_ecs::prelude::*;
use macroquad::prelude::*;

use super::GlyphBatch;

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

pub fn render_all(
    mut q_glyph_batches: Query<&mut GlyphBatch>,
) {
    for mut batch in q_glyph_batches.iter_mut() {
        batch.render();
    }
}
