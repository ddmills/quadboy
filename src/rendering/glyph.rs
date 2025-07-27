use bevy_ecs::prelude::*;
use macroquad::{
    prelude::*,
    telemetry::{self},
};

use crate::{
    cfg::TILE_SIZE_F32, common::{MacroquadColorable, Palette}, domain::ZoneStatus, rendering::{GlyphTextureId, IsVisible, RenderTargetType, Visibility}
};

use super::{GameCamera, Layers, Position, RenderLayer, Renderable, ScreenSize};

#[derive(Component, Default)]
#[require(Visibility)]
pub struct Glyph {
    pub idx: usize,
    pub fg1: Option<u32>,
    pub fg2: Option<u32>,
    pub bg: Option<u32>,
    pub outline: Option<u32>,
    pub layer_id: RenderLayer,
    pub texture_id: GlyphTextureId,
    pub is_dormant: bool,
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
pub const SHROUD_COLOR: Vec4 = Vec4::new(0.227, 0.243, 0.247, 1.0);

impl Glyph {
    pub fn new<T: Into<u32>>(idx: usize, fg1: T, fg2: T) -> Self {
        Self {
            idx,
            fg1: Some(fg1.into()),
            fg2: Some(fg2.into()),
            bg: None,
            outline: Some(Palette::Black.into()),
            layer_id: RenderLayer::default(),
            texture_id: GlyphTextureId::Cowboy,
            is_dormant: false,
        }
    }

    pub fn layer(mut self, layer_id: RenderLayer) -> Self {
        self.layer_id = layer_id;
        self
    }

    pub fn texture(mut self, texture_id: GlyphTextureId) -> Self {
        self.texture_id = texture_id;
        self
    }

    pub fn bg<T: Into<u32>>(mut self, bg: T) -> Self {
        self.bg = Some(bg.into());
        self
    }

    pub fn outline<T: Into<u32>>(mut self, outline: T) -> Self {
        self.outline = Some(outline.into());
        self
    }

    pub fn fg1<T: Into<u32>>(mut self, fg1: T) -> Self {
        self.fg1 = Some(fg1.into());
        self
    }

    pub fn fg2<T: Into<u32>>(mut self, fg2: T) -> Self {
        self.fg2 = Some(fg2.into());
        self
    }

    pub fn get_style(&self) -> GlyphStyle {
        if self.is_dormant {
            return GlyphStyle {
                bg: TRANSPARENT,
                fg1: SHROUD_COLOR,
                fg2: SHROUD_COLOR,
                outline: self.outline.map(|x| x.to_vec4_a(1.)).unwrap_or(TRANSPARENT),
            };
        }

        GlyphStyle {
            bg: self.bg.map(|x| x.to_vec4_a(1.)).unwrap_or(TRANSPARENT),
            fg1: self.fg1.map(|x| x.to_vec4_a(1.)).unwrap_or(TRANSPARENT),
            fg2: self.fg2.map(|x| x.to_vec4_a(1.)).unwrap_or(TRANSPARENT),
            outline: self.outline.map(|x| x.to_vec4_a(1.)).unwrap_or(TRANSPARENT),
        }
    }
}

pub fn render_glyphs(
    q_glyphs: Query<(&Glyph, &Position), With<IsVisible>>,
    mut layers: ResMut<Layers>,
    camera: Res<GameCamera>,
    screen: Res<ScreenSize>,
) {
    for layer in layers.get_all().iter_mut() {
        layer.clear();
    }

    telemetry::begin_zone("set-glyphs");

    let screen_w = screen.width as f32;
    let screen_h = screen.height as f32;

    let cam_x = (camera.x * TILE_SIZE_F32.0).floor();
    let cam_y = (camera.y * TILE_SIZE_F32.1).floor();

    q_glyphs.iter().for_each(|(glyph, pos)| {
        let texture_id = glyph.texture_id;
        let mut x = pos.x * TILE_SIZE_F32.0;
        let mut y = pos.y * TILE_SIZE_F32.1;
        let w = texture_id.get_glyph_width();
        let h = texture_id.get_glyph_height();
        let layer = layers.get_layer(glyph.layer_id);

        if layer.target_type == RenderTargetType::World {
            x -= cam_x;
            y -= cam_y;

            if x + w < 0. || x - w > camera.width || y + h < 0. || y - h > camera.height {
                return;
            }
        } else if x + w < 0. || x > screen_w || y + h < 0. || y > screen_h {
            return;
        }

        let style = glyph.get_style();

        layer.add(Renderable {
            idx: glyph.idx,
            fg1: style.fg1,
            fg2: style.fg2,
            bg: style.bg,
            outline: style.outline,
            x,
            y,
            w,
            h,
            tex_idx: texture_id.get_texture_idx(),
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

pub fn on_zone_status_change(mut q_changed: Query<(&mut Glyph, &ZoneStatus), Changed<ZoneStatus>>) {
    for (mut glyph, status) in q_changed.iter_mut() {
        glyph.is_dormant = *status == ZoneStatus::Dormant;
    }
}
