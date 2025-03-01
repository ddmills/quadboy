use bevy_ecs::prelude::*;
use macroquad::prelude::*;

use crate::{cfg::{BODY_FONT_SIZE_F32, TILE_SIZE_F32, TITLE_FONT_SIZE_F32}, common::{MacroquadColorable, Palette}};

use super::{Position, Renderable, Renderer};

pub const TRANSPARENT: Color = Color::new(0., 0., 0., 0.);

#[derive(Component, Default)]
pub struct Glyph {
    idx: usize,
    fg1: Option<u32>,
    fg2: Option<u32>,
    bg: Option<u32>,
    outline: Option<u32>,
}

#[derive(Resource)]
pub struct TilesetTextures {
    pub glyph_texture: Texture2D,
    pub font_body_texture: Texture2D,
    pub font_title_texture: Texture2D,
}

impl TilesetTextures {
    pub fn get_by_id(&self, id: &TilesetId) -> &Texture2D {
        match id {
            TilesetId::Glyph => &self.glyph_texture,
            TilesetId::BodyFont => &self.font_body_texture,
            TilesetId::TitleFont => &self.font_title_texture,
        }
    }

    pub fn get_size(&self, id: &TilesetId) -> Vec2 {
        match id {
            TilesetId::Glyph => vec2(TILE_SIZE_F32.0, TILE_SIZE_F32.1),
            TilesetId::BodyFont => vec2(BODY_FONT_SIZE_F32.0, BODY_FONT_SIZE_F32.1),
            TilesetId::TitleFont => vec2(TITLE_FONT_SIZE_F32.0, TITLE_FONT_SIZE_F32.1),
        }
    }
}

pub enum TilesetId {
    Glyph,
    BodyFont,
    TitleFont,
}

pub struct GlyphStyle {
    pub fg1: Color,
    pub fg2: Color,
    pub bg: Color,
    pub outline: Color,
}

impl Glyph {
    pub fn new<T: Into<u32>>(idx: usize, fg1: T, fg2: T) -> Self {
        Self {
            idx,
            fg1: Some(fg1.into()),
            fg2: Some(fg2.into()),
            bg: None,
            outline: None,
        }
    }

    pub fn get_style(&self) -> GlyphStyle {
        GlyphStyle {
            bg: self
                .bg
                .map(|x| x.to_macroquad_color())
                .unwrap_or(TRANSPARENT),
            fg1: self
                .fg1
                .map(|x| x.to_macroquad_color())
                .unwrap_or(TRANSPARENT),
            fg2: self
                .fg2
                .map(|x| x.to_macroquad_color())
                .unwrap_or(TRANSPARENT),
            outline: self
                .outline
                .map(|x| x.to_macroquad_color())
                .unwrap_or(Palette::Black.to_macroquad_color()),
        }
    }
}

pub fn render_glyphs(
    q_glyphs: Query<(&Glyph, &Position)>,
    mut renderer: ResMut<Renderer>,
) {
    for (glyph, position) in q_glyphs.iter() {
        let style = glyph.get_style();

        renderer.draw(Renderable {
            idx: glyph.idx,
            fg1: style.fg1,
            fg2: style.fg2,
            bg: style.bg,
            outline: style.outline,
            tileset_id: TilesetId::Glyph,
            x: position.x * TILE_SIZE_F32.0,
            y: position.y * TILE_SIZE_F32.1,
        });
    }
}

pub async fn load_tilesets() -> TilesetTextures {
    let glyph_texture_fut = load_texture("./src/assets/textures/cowboy.png");
    let font_body_texture_fut = load_texture("./src/assets/textures/tocky_8x12.png");
    let font_title_texture_fut = load_texture("./src/assets/textures/nix8810_8x24.png");

    let glyph_texture = glyph_texture_fut.await.unwrap();
    let font_body_texture = font_body_texture_fut.await.unwrap();
    let font_title_texture = font_title_texture_fut.await.unwrap();

    TilesetTextures {
        glyph_texture,
        font_body_texture,
        font_title_texture,
    }
}
