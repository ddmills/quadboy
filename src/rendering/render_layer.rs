use bevy_ecs::prelude::*;

use crate::{cfg::{BODY_FONT_SIZE_F32, TILE_SIZE_F32}, rendering::RenderTargetType};

use super::{GlyphBatch, TilesetTextures};

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum RenderLayer {
    #[default]
    Ground,
    Ui,
    Text,
}

#[derive(Resource)]
pub struct Layers {
    pub ground: GlyphBatch,
    pub ui: GlyphBatch,
    pub text: GlyphBatch,
}

impl FromWorld for Layers {
    fn from_world(world: &mut World) -> Self {
        let textures = world.get_resource::<TilesetTextures>().unwrap();

        Self {
            ui: GlyphBatch::new(textures.glyph_texture.raw_miniquad_id(), RenderTargetType::Screen, 8000),
            ground: GlyphBatch::new(textures.glyph_texture.raw_miniquad_id(), RenderTargetType::World, 8000),
            text: GlyphBatch::new(textures.font_body_texture.raw_miniquad_id(), RenderTargetType::Screen, 8000),
        }
    }
}

impl Layers {
    #[inline]
    pub fn get_layer(&mut self, layer: RenderLayer) -> &mut GlyphBatch {
        match layer {
            RenderLayer::Ground => &mut self.ground,
            RenderLayer::Ui => &mut self.ui,
            RenderLayer::Text => &mut self.text,
        }
    }

    pub fn get_all(&mut self) -> Vec<&mut GlyphBatch> {
        vec![
            &mut self.ui,
            &mut self.ground,
            &mut self.text,
        ]
    }

    #[inline]
    pub fn get_glyph_width(&self, layer: RenderLayer) -> f32 {
        match layer {
            RenderLayer::Ground => TILE_SIZE_F32.0,
            RenderLayer::Ui => TILE_SIZE_F32.0,
            RenderLayer::Text => BODY_FONT_SIZE_F32.0,
        }
    }

    #[inline]
    pub fn get_glyph_height(&self, layer: RenderLayer) -> f32 {
        match layer {
            RenderLayer::Ground => TILE_SIZE_F32.1,
            RenderLayer::Ui => TILE_SIZE_F32.1,
            RenderLayer::Text => BODY_FONT_SIZE_F32.1,
        }
    }
}
