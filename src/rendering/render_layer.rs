use bevy_ecs::prelude::*;

use crate::cfg::{BODY_FONT_SIZE_F32, TILE_SIZE_F32};

use super::GlyphBatch;

#[derive(Default, Clone, Copy)]
pub enum RenderLayer {
    #[default]
    Ground,
    Text,
}

#[derive(Resource)]
pub struct Layers {
    pub ground: GlyphBatch,
    pub text: GlyphBatch,
}

impl Layers {
    #[inline]
    pub fn get_layer(&mut self, layer: RenderLayer) -> &mut GlyphBatch
    {
        match layer {
            RenderLayer::Ground => &mut self.ground,
            RenderLayer::Text => &mut self.text,
        }
    }

    #[inline]
    pub fn get_glyph_width(&self, layer: RenderLayer) -> f32
    {
        match layer {
            RenderLayer::Ground => TILE_SIZE_F32.0,
            RenderLayer::Text => BODY_FONT_SIZE_F32.0,
        }
    }

    #[inline]
    pub fn get_glyph_height(&self, layer: RenderLayer) -> f32
    {
        match layer {
            RenderLayer::Ground => TILE_SIZE_F32.1,
            RenderLayer::Text => BODY_FONT_SIZE_F32.1,
        }
    }
}
