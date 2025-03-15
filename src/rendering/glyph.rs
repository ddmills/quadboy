use bevy_ecs::prelude::*;
use macroquad::{
    miniquad::gl, prelude::*, telemetry::{self}
};

use crate::{
    cfg::TILE_SIZE_F32,
    common::{MacroquadColorable, Palette},
};

use super::{get_render_target_size, GameCamera, Layers, Position, RenderLayer, Renderable};

#[derive(Component, Default)]
pub struct Glyph {
    pub idx: usize,
    pub fg1: Option<u32>,
    pub fg2: Option<u32>,
    pub bg: Option<u32>,
    pub outline: Option<u32>,
    pub layer_id: RenderLayer,
}

#[derive(Resource)]
pub struct TilesetTextures {
    pub glyph_texture: Texture2D,
    pub font_body_texture: Texture2D,
}

pub struct GlyphStyle {
    pub fg1: Vec4,
    pub fg2: Vec4,
    pub bg: Vec4,
    pub outline: Vec4,
}

pub const TRANSPARENT: Vec4 = Vec4::splat(0.);

impl Glyph {
    pub fn new<T: Into<u32>>(idx: usize, fg1: T, fg2: T) -> Self {
        Self {
            idx,
            fg1: Some(fg1.into()),
            fg2: Some(fg2.into()),
            bg: None,
            outline: Some(Palette::Black.into()),
            layer_id: RenderLayer::default(),
        }
    }

    pub fn layer(mut self, layer_id: RenderLayer) -> Self {
        self.layer_id = layer_id;
        self
    }

    pub fn get_style(&self) -> GlyphStyle {
        GlyphStyle {
            bg: self.bg.map(|x| x.to_vec4_a(1.)).unwrap_or(TRANSPARENT),
            fg1: self.fg1.map(|x| x.to_vec4_a(1.)).unwrap_or(TRANSPARENT),
            fg2: self.fg2.map(|x| x.to_vec4_a(1.)).unwrap_or(TRANSPARENT),
            outline: self.outline.map(|x| x.to_vec4_a(1.)).unwrap_or(TRANSPARENT),
        }
    }
}

pub fn render_glyphs(q_glyphs: Query<(&Glyph, &Position)>, mut layers: ResMut<Layers>, camera: Res<GameCamera>) {
    let screen = get_render_target_size().as_vec2();

    layers.ground.clear();
    layers.text.clear();

    telemetry::begin_zone("set-glyphs");

    q_glyphs.iter().for_each(|(glyph, pos)| {
        let mut x = pos.x * TILE_SIZE_F32.0;
        let mut y = pos.y * TILE_SIZE_F32.1;
        let w = layers.get_glyph_width(glyph.layer_id);
        let h = layers.get_glyph_height(glyph.layer_id);

        if glyph.layer_id == RenderLayer::Ground {
            x -= camera.x * TILE_SIZE_F32.0;
            y -= camera.y * TILE_SIZE_F32.1;
        }

        if x + w < 0. || x > screen.x || y + h < 0. || y > screen.y {
            return;
        }

        let style = glyph.get_style();

        layers.get_layer(glyph.layer_id).add(Renderable {
            idx: glyph.idx,
            fg1: style.fg1,
            fg2: style.fg2,
            bg: style.bg,
            outline: style.outline,
            x,
            y,
            w,
            h,
        });
    });

    telemetry::end_zone();
}

pub async fn load_tilesets() -> TilesetTextures {
    let glyph_texture_fut = load_texture("./src/assets/textures/cowboy.png");
    let font_body_texture_fut = load_texture("./src/assets/textures/tocky_8x12.png");

    let glyph_texture = glyph_texture_fut.await.unwrap();
    let font_body_texture = font_body_texture_fut.await.unwrap();

    TilesetTextures {
        glyph_texture,
        font_body_texture,
    }
}
