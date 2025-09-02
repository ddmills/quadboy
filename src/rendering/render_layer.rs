use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    cfg::{BODY_FONT_SIZE_F32, TILE_SIZE_F32},
    rendering::RenderTargetType,
};

use super::{GlyphBatch, TilesetRegistry};

#[derive(Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
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

#[derive(Default, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum Layer {
    Terrain,
    GroundOverlay,
    #[default]
    Objects,
    Actors,
    Overlay,
    UiPanels,
    Ui,
}

impl Layer {
    pub fn get_all() -> Vec<Layer> {
        vec![
            Self::Terrain,
            Self::GroundOverlay,
            Self::Objects,
            Self::Actors,
            Self::Overlay,
            Self::UiPanels,
            Self::Ui,
        ]
    }

    #[inline]
    pub fn as_index(&self) -> usize {
        match self {
            Self::Terrain => 0,
            Self::GroundOverlay => 1,
            Self::Objects => 2,
            Self::Actors => 3,
            Self::Overlay => 4,
            Self::UiPanels => 5,
            Self::Ui => 6,
        }
    }

    pub const COUNT: usize = 7;

    pub fn get_target_type(&self) -> RenderTargetType {
        match self {
            Self::Terrain => RenderTargetType::World,
            Self::GroundOverlay => RenderTargetType::World,
            Self::Objects => RenderTargetType::World,
            Self::Actors => RenderTargetType::World,
            Self::Overlay => RenderTargetType::World,
            Self::UiPanels => RenderTargetType::Screen,
            Self::Ui => RenderTargetType::Screen,
        }
    }
}

#[derive(Resource)]
pub struct Layers {
    pub all: [GlyphBatch; Layer::COUNT],
}

impl FromWorld for Layers {
    fn from_world(world: &mut World) -> Self {
        let textures = world.get_resource::<TilesetRegistry>().unwrap();
        let texture_glyph = textures.glyph_texture.raw_miniquad_id();
        let texture_body_text = textures.font_body_texture.raw_miniquad_id();

        let all = Layer::get_all()
            .into_iter()
            .map(|layer| {
                GlyphBatch::new(
                    texture_glyph,
                    texture_body_text,
                    layer.get_target_type(),
                    4000,
                )
            })
            .collect::<Vec<_>>()
            .try_into()
            .unwrap_or_else(|v: Vec<_>| {
                panic!("Expected {} layers, got {}", Layer::COUNT, v.len())
            });

        Self { all }
    }
}

impl Layers {
    #[inline]
    pub fn get_mut(&mut self, layer: Layer) -> &mut GlyphBatch {
        &mut self.all[layer.as_index()]
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut GlyphBatch> {
        self.all.iter_mut()
    }
}
