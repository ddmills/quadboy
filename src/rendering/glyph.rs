use bevy_ecs::prelude::*;
use macroquad::prelude::*;

use crate::common::{MacroquadColorable, Palette};

use super::{get_render_offset, get_render_target_size, GlyphMaterial, Position};

const TRANSPARENT: Color = Color::new(0., 0., 0., 0.);

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
    material: Res<GlyphMaterial>,
    tilesets: Res<TilesetTextures>,
) {
    gl_use_material(&material.0);

    let texture = tilesets.glyph_texture.weak_clone();

    // let target_size = get_render_target_size();
    // let is_x_even = target_size.x % 2 == 0;
    // let is_y_even = target_size.y % 2 == 0;

    // let offset_x = if is_x_even { 0.0 } else { 0.5 };
    // let offset_y = if is_y_even { 0.0 } else { 0.5 };

    let offset = get_render_offset();

    for (glyph, position) in q_glyphs.iter() {
        let style = glyph.get_style();

        material.0.set_uniform("fg1", style.fg1);
        material.0.set_uniform("fg2", style.fg2);
        material.0.set_uniform("outline", style.outline);
        material.0.set_uniform("bg", style.bg);
        material.0.set_uniform("idx", glyph.idx as f32);

        draw_texture_ex(
            &texture,
            position.x + offset.x,
            position.y + offset.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(16., 24.)),
                source: None,
                rotation: 0.,
                flip_x: false,
                flip_y: false,
                pivot: None,
            },
        );
    }

    gl_use_default_material();
}

pub async fn load_tilesets() -> TilesetTextures {
    let glyph_texture = load_texture("./src/assets/textures/cowboy.png")
        .await
        .unwrap();

    TilesetTextures { glyph_texture }
}
