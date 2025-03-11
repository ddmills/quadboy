use bevy_ecs::prelude::*;

use crate::{cfg::{BODY_FONT_SIZE_F32, TILE_SIZE_F32}, common::{cp437_idx, MacroquadColorable, Palette}};

use super::{GlyphStyle, Position, Renderable, TRANSPARENT};

#[derive(Component)]
pub struct Text {
    pub value: String,
    pub bg: Option<u32>,
    pub fg1: Option<u32>,
    pub fg2: Option<u32>,
    pub outline: Option<u32>,
}

impl Text {
    pub fn new(value: &str) -> Self {
        Self {
            value: value.into(),
            bg: None,
            fg1: Some(Palette::White.into()),
            fg2: None,
            outline: None,
        }
    }
    
    pub fn get_style(&self) -> GlyphStyle {
        GlyphStyle {
            bg: self
                .bg
                .map(|x| x.to_vec4())
                .unwrap_or(TRANSPARENT),
            fg1: self
                .fg1
                .map(|x| x.to_vec4())
                .unwrap_or(TRANSPARENT),
            fg2: self
                .fg2
                .map(|x| x.to_vec4())
                .unwrap_or(TRANSPARENT),
            outline: self
                .outline
                .map(|x| x.to_vec4())
                .unwrap_or(Palette::Black.to_vec4()),
        }
    }
}

pub fn render_text(
    q_text: Query<(&Text, &Position)>,
) {
    for (text, position) in q_text.iter() {
        let style = text.get_style();

        for (i, c) in text.value.chars().enumerate() {
            // renderer.draw(Renderable {
            //     idx: cp437_idx(c).unwrap_or(0),
            //     fg1: style.fg1,
            //     fg2: style.fg2,
            //     bg: style.bg,
            //     outline: style.outline,
            //     tileset_id: super::TilesetId::BodyFont,
            //     x: position.x * TILE_SIZE_F32.0 + i as f32 * BODY_FONT_SIZE_F32.0,
            //     y: position.y * TILE_SIZE_F32.1,
            //     w: BODY_FONT_SIZE_F32.0,
            //     h: BODY_FONT_SIZE_F32.1,
            // });
        }
    }
}
