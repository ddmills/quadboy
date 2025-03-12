use std::{u32, vec};

use bevy_ecs::prelude::*;
use macroquad::{prelude::*, rand::ChooseRandom, telemetry::{self, ZoneGuard}};

use crate::{cfg::{BODY_FONT_SIZE_F32, TILE_SIZE_F32, TITLE_FONT_SIZE_F32}, common::{MacroquadColorable, Palette}, ecs::Time};

use super::{get_render_target_size, GlyphBatch, Position, Renderable};

#[derive(Component, Default)]
pub struct Glyph {
    pub idx: usize,
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
            outline: None,
        }
    }

    pub fn get_style(&self) -> GlyphStyle {
        GlyphStyle {
            bg: self
                .bg
                .map(|x| x.to_vec4_a(1.))
                .unwrap_or(TRANSPARENT),
            fg1: self
                .fg1
                .map(|x| x.to_vec4_a(1.))
                .unwrap_or(TRANSPARENT),
            fg2: self
                .fg2
                .map(|x| x.to_vec4_a(1.))
                .unwrap_or(TRANSPARENT),
            outline: self
                .outline
                .map(|x| x.to_vec4_a(1.))
                .unwrap_or(TRANSPARENT),
        }
    }
}

pub fn render_glyphs(
    q_glyphs: Query<(&Glyph, &Position)>,
    mut glyph_batch: Single<&mut GlyphBatch>,
    time: Res<Time>
) {
    let options = vec![Palette::Yellow, Palette::Red, Palette::Purple, Palette::Green];
    let fg1 = options.choose().unwrap();
    let fg2 = options.choose().unwrap();

    let screen = get_render_target_size().as_vec2();
    let w = TILE_SIZE_F32.0;
    let h = TILE_SIZE_F32.1;

    telemetry::begin_zone("get-renderables");
    let renderables = q_glyphs
        .iter()
        .enumerate()
        .filter_map(|(idx, (glyph, pos))| {
            let x = (pos.x + (time.start as f32).sin()) * TILE_SIZE_F32.0;
            let y = (pos.y + (time.start as f32).sin()) * TILE_SIZE_F32.1;

            if x + w < 0. || x > screen.x || y + h < 0. || y > screen.y {
                return None;
            }

            let style = glyph.get_style();


            let t1 = (idx + (time.start.floor() as usize)) % options.len();
            let t2= (idx + (time.start.floor() as usize) + 1) % options.len();

            let fg1: Palette = options[t1];
            let fg2: Palette = options[t2];

            Some(Renderable {
                idx: glyph.idx,
                // fg1: style.fg1,
                // fg2: style.fg2,
                fg1: fg1.to_vec4(),
                fg2: fg2.to_vec4(),
                bg: style.bg,
                // bg: fg2.to_macroquad_color(),
                // outline: style.outline,
                outline: Palette::Black.to_vec4(),
                x,
                y,
                w: TILE_SIZE_F32.0,
                h: TILE_SIZE_F32.1,
            })
        });

    telemetry::end_zone();

    {
        let _z = ZoneGuard::new("set-glyphs");
        glyph_batch.set_glyphs(renderables);
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
