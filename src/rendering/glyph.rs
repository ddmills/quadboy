use crate::engine::SerializableComponent;
use crate::{
    cfg::TILE_SIZE_F32,
    common::{MacroquadColorable, Palette},
    domain::{ApplyVisibilityEffects, IsExplored, IsVisible, Player, ZoneStatus},
    rendering::{GlyphTextureId, RenderTargetType, Visibility},
    ui::UiLayout,
};
use bevy_ecs::prelude::*;
use macroquad::{prelude::*, telemetry};
use serde::{Deserialize, Serialize};

use super::{GameCamera, Layer, Layers, Position, Renderable, ScreenSize};

#[derive(Component, Default, Serialize, Deserialize, Clone, SerializableComponent)]
#[require(Visibility)]
pub struct Glyph {
    pub idx: usize,
    pub fg1: Option<u32>,
    pub fg2: Option<u32>,
    pub bg: Option<u32>,
    pub outline: Option<u32>,
    pub layer_id: Layer,
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
pub const SHROUD_FG_COLOR: u32 = 0x8F8F8F;
pub const SHROUD_BG_COLOR: u32 = 0x535353;
pub const SHROUD_OUTLINE_COLOR: u32 = 0x1F1F1F;

#[allow(dead_code)]
impl Glyph {
    pub fn idx(idx: usize) -> Self {
        Self {
            idx,
            fg1: None,
            fg2: None,
            bg: None,
            outline: Some(Palette::Clear.into()),
            layer_id: Layer::default(),
            texture_id: GlyphTextureId::Cowboy,
            is_dormant: false,
        }
    }

    pub fn new<T: Into<u32>>(idx: usize, fg1: T, fg2: T) -> Self {
        Self {
            idx,
            fg1: Some(fg1.into()),
            fg2: Some(fg2.into()),
            bg: None,
            outline: Some(Palette::Clear.into()),
            layer_id: Layer::default(),
            texture_id: GlyphTextureId::Cowboy,
            is_dormant: false,
        }
    }

    pub fn layer(mut self, layer_id: Layer) -> Self {
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

    pub fn bg_opt<T: Into<u32>>(mut self, bg: Option<T>) -> Self {
        self.bg = bg.map(|v| v.into());
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

    pub fn fg1_opt<T: Into<u32>>(mut self, fg1: Option<T>) -> Self {
        self.fg1 = fg1.map(|v| v.into());
        self
    }

    pub fn fg2<T: Into<u32>>(mut self, fg2: T) -> Self {
        self.fg2 = Some(fg2.into());
        self
    }

    pub fn get_style(&self) -> GlyphStyle {
        if self.is_dormant {
            return GlyphStyle {
                bg: self
                    .bg
                    .map(|_| SHROUD_BG_COLOR.to_vec4_a(1.0))
                    .unwrap_or(TRANSPARENT),
                fg1: SHROUD_FG_COLOR.to_vec4_a(1.),
                fg2: SHROUD_FG_COLOR.to_vec4_a(1.),
                outline: self
                    .outline
                    .map(|_| SHROUD_OUTLINE_COLOR.to_vec4_a(1.))
                    .unwrap_or(TRANSPARENT),
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
    q_glyphs: Query<(
        &Glyph,
        &Position,
        Option<&IsVisible>,
        Option<&IsExplored>,
        Option<&ApplyVisibilityEffects>,
    )>,
    mut layers: ResMut<Layers>,
    camera: Res<GameCamera>,
    screen: Res<ScreenSize>,
    ui: Res<UiLayout>,
    player: Query<&Position, With<Player>>,
) {
    layers.iter_mut().for_each(|layer| {
        layer.clear();
    });

    telemetry::begin_zone("render_glyphs");

    let screen_w = screen.width as f32;
    let screen_h = screen.height as f32;
    let tile_w = TILE_SIZE_F32.0;
    let tile_h = TILE_SIZE_F32.1;
    let cam_x = (camera.x * tile_w).floor();
    let cam_y = (camera.y * tile_h).floor();
    let camera_width = camera.width;
    let camera_height = camera.height;
    let ui_panel_x = (ui.game_panel.x as f32) * tile_w;
    let ui_panel_y = (ui.game_panel.y as f32) * tile_h;
    let player_z = player.single().map(|p| p.z.floor()).unwrap_or(0.);

    q_glyphs.iter().for_each(
        |(glyph, pos, is_visible, is_explored, apply_visibility_effects)| {
            let texture_id = glyph.texture_id;

            let mut x = (pos.x * tile_w).floor();
            let mut y = (pos.y * tile_h).floor();
            let w = texture_id.get_glyph_width();
            let h = texture_id.get_glyph_height();
            let layer = layers.get_mut(glyph.layer_id);

            if layer.target_type == RenderTargetType::World {
                if pos.z.floor() != player_z {
                    return;
                }

                // Skip rendering if glyph has ApplyVisibilityEffects but is not explored
                if apply_visibility_effects.is_some() && is_explored.is_none() {
                    return;
                }

                x -= cam_x;
                y -= cam_y;

                if x + w < 0. || x - w > camera_width || y + h < 0. || y - h > camera_height {
                    return;
                }

                x += ui_panel_x;
                y += ui_panel_y;
            } else if x + w < 0. || x > screen_w || y + h < 0. || y > screen_h {
                return;
            }

            let style = glyph.get_style();

            let is_shrouded =
                apply_visibility_effects.is_some() && is_explored.is_some() && is_visible.is_none();

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
                is_shrouded: is_shrouded as u32,
            });
        },
    );

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
