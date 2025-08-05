use bevy_ecs::prelude::*;

use crate::{
    cfg::{BODY_FONT_SIZE_F32, TILE_SIZE_F32},
    rendering::RenderTargetType,
};

use super::{GlyphBatch, TilesetTextures};

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum RenderLayer {
    #[default]
    Ground,
    Actors,
    UiPanels,
    Ui,
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum GlyphTextureId {
    #[default]
    Cowboy,
    BodyFont,
}

impl GlyphTextureId {
    #[inline]
    pub fn get_texture_idx(&self) -> usize {
        match self {
            Self::Cowboy => 0,
            Self::BodyFont => 1,
        }
    }

    #[inline]
    pub fn get_glyph_width(&self) -> f32 {
        match self {
            Self::Cowboy => TILE_SIZE_F32.0,
            Self::BodyFont => BODY_FONT_SIZE_F32.0,
        }
    }

    #[inline]
    pub fn get_glyph_height(&self) -> f32 {
        match self {
            Self::Cowboy => TILE_SIZE_F32.1,
            Self::BodyFont => BODY_FONT_SIZE_F32.1,
        }
    }
}

#[derive(Resource)]
pub struct Layers {
    pub ground: GlyphBatch,
    pub actors: GlyphBatch,
    pub panels: GlyphBatch,
    pub ui: GlyphBatch,
}

impl FromWorld for Layers {
    fn from_world(world: &mut World) -> Self {
        let textures = world.get_resource::<TilesetTextures>().unwrap();
        let texture_glyph = textures.glyph_texture.raw_miniquad_id();
        let texture_body_text = textures.font_body_texture.raw_miniquad_id();

        Self {
            ground: GlyphBatch::new(
                texture_glyph,
                texture_body_text,
                RenderTargetType::World,
                8000,
            ),
            actors: GlyphBatch::new(
                texture_glyph,
                texture_body_text,
                RenderTargetType::World,
                8000,
            ),
            panels: GlyphBatch::new(
                texture_glyph,
                texture_body_text,
                RenderTargetType::Screen,
                8000,
            ),
            ui: GlyphBatch::new(
                texture_glyph,
                texture_body_text,
                RenderTargetType::Screen,
                8000,
            ),
        }
    }
}

impl Layers {
    #[inline]
    pub fn get_layer(&mut self, layer: RenderLayer) -> &mut GlyphBatch {
        match layer {
            RenderLayer::Ground => &mut self.ground,
            RenderLayer::Actors => &mut self.actors,
            RenderLayer::Ui => &mut self.ui,
            RenderLayer::UiPanels => &mut self.panels,
        }
    }

    pub fn get_all(&mut self) -> Vec<&mut GlyphBatch> {
        vec![
            &mut self.ui,
            &mut self.panels,
            &mut self.ground,
            &mut self.actors,
        ]
    }
}
