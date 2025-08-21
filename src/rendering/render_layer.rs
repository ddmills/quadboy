use std::collections::{HashMap, hash_map::ValuesMut};

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    cfg::{BODY_FONT_SIZE_F32, TILE_SIZE_F32},
    rendering::RenderTargetType,
};

use super::{GlyphBatch, TilesetTextures};

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
    pub all: HashMap<Layer, GlyphBatch>,
}

impl FromWorld for Layers {
    fn from_world(world: &mut World) -> Self {
        let textures = world.get_resource::<TilesetTextures>().unwrap();
        let texture_glyph = textures.glyph_texture.raw_miniquad_id();
        let texture_body_text = textures.font_body_texture.raw_miniquad_id();

        let all = Layer::get_all()
            .iter()
            .map(|l| {
                (
                    *l,
                    GlyphBatch::new(texture_glyph, texture_body_text, l.get_target_type(), 4000),
                )
            })
            .collect::<HashMap<_, _>>();

        Self { all }
    }
}

impl Layers {
    #[inline]
    pub fn get_mut(&mut self, layer: Layer) -> &mut GlyphBatch {
        self.all
            .get_mut(&layer)
            .expect("Expected render layer to exist!")
    }

    pub fn iter_mut(&mut self) -> ValuesMut<'_, Layer, GlyphBatch> {
        self.all.values_mut()
    }
}
