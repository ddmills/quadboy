use crate::domain::Style;
use crate::engine::SerializableComponent;
use crate::rendering::{LightValue, world_to_zone_local};
use crate::{
    cfg::TILE_SIZE_F32,
    common::{MacroquadColorable, Palette},
    domain::{
        ApplyVisibilityEffects, HideWhenNotVisible, IgnoreLighting, IsExplored, IsVisible, Player,
        ZoneStatus,
    },
    rendering::{GlyphTextureId, RenderTargetType, Visibility},
    ui::{DialogState, UiLayout},
};
use bevy_ecs::prelude::*;
use macroquad::{prelude::*, telemetry};
use serde::{Deserialize, Serialize};

use super::{GameCamera, Layer, Layers, LightingData, Position, Renderable, ScreenSize};

#[derive(Component, Default, Serialize, Deserialize, Clone, SerializableComponent)]
#[require(Visibility)]
pub struct Glyph {
    pub idx: usize,
    pub fg1: Option<u32>,
    pub fg2: Option<u32>,
    pub bg: Option<u32>,
    pub outline: Option<u32>,
    pub scale: (f32, f32),
    pub layer_id: Layer,
    pub texture_id: GlyphTextureId,
    pub is_dormant: bool,
}

#[derive(Resource)]
pub struct TilesetRegistry {
    pub glyph_texture: Texture2D,
    pub font_body_texture: Texture2D,
}

impl TilesetRegistry {
    pub async fn load() -> Self {
        let glyph_texture_fut = load_texture("./src/assets/textures/cowboy.png");
        // let font_body_texture_fut = load_texture("./src/assets/textures/tocky_2_8x12.png");
        let font_body_texture_fut = load_texture("./src/assets/textures/acer_8x12.png");
        // let font_body_texture_fut = load_texture("./src/assets/textures/tamzen_8x12.png");

        let glyph_texture = glyph_texture_fut.await.unwrap();
        let font_body_texture = font_body_texture_fut.await.unwrap();

        TilesetRegistry {
            glyph_texture,
            font_body_texture,
        }
    }
}

#[derive(Clone, Copy)]
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
pub const DIMMING_FACTOR: f32 = 0.8;

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
            scale: (1., 1.),
        }
    }

    pub fn new_from_style(style: Style) -> Self {
        Self {
            idx: style.idx,
            fg1: style.fg1.map(|x| x.into()),
            fg2: style.fg2.map(|x| x.into()),
            bg: style.bg.map(|x| x.into()),
            outline: style.outline.map(|x| x.into()),
            layer_id: Layer::default(),
            texture_id: GlyphTextureId::Cowboy,
            is_dormant: false,
            scale: (1., 1.),
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
            scale: (1., 1.),
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

    pub fn scale(mut self, value: (f32, f32)) -> Self {
        self.scale = value;
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

/// Apply dimming and desaturation effect to glyph style when dialog is open
fn dim_glyph_style(style: GlyphStyle) -> GlyphStyle {
    fn dim_and_desaturate_color(color: Vec4, dim_factor: f32, desat_factor: f32) -> Vec4 {
        // Calculate luminance (grayscale value)
        let luminance = 0.299 * color.x + 0.587 * color.y + 0.114 * color.z;

        // Mix original color with grayscale (desaturate)
        let desaturated_r = color.x * (1.0 - desat_factor) + luminance * desat_factor;
        let desaturated_g = color.y * (1.0 - desat_factor) + luminance * desat_factor;
        let desaturated_b = color.z * (1.0 - desat_factor) + luminance * desat_factor;

        // Apply dimming to the desaturated color
        Vec4::new(
            desaturated_r * dim_factor,
            desaturated_g * dim_factor,
            desaturated_b * dim_factor,
            color.w,
        )
    }

    const DESATURATION_FACTOR: f32 = 0.9;

    GlyphStyle {
        fg1: dim_and_desaturate_color(style.fg1, DIMMING_FACTOR, DESATURATION_FACTOR),
        fg2: dim_and_desaturate_color(style.fg2, DIMMING_FACTOR, DESATURATION_FACTOR),
        bg: dim_and_desaturate_color(style.bg, DIMMING_FACTOR, DESATURATION_FACTOR),
        outline: dim_and_desaturate_color(style.outline, DIMMING_FACTOR, DESATURATION_FACTOR),
    }
}

pub fn render_glyphs(
    q_glyphs: Query<(Entity, &Glyph, &Position, &Visibility)>,
    q_visibility: Query<
        (
            Option<&IsVisible>,
            Option<&IsExplored>,
            Option<&ApplyVisibilityEffects>,
            Option<&HideWhenNotVisible>,
            Option<&IgnoreLighting>,
        ),
        With<Glyph>,
    >,
    mut layers: ResMut<Layers>,
    camera: Res<GameCamera>,
    screen: Res<ScreenSize>,
    ui: Res<UiLayout>,
    player: Query<&Position, With<Player>>,
    dialog_state: Res<DialogState>,
    lighting_data: Res<LightingData>,
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

    let world_left = -tile_w;
    let world_right = camera_width + tile_w;
    let world_top = -tile_h;
    let world_bottom = camera_height + tile_h;

    for (entity, glyph, pos, visibility) in q_glyphs.iter() {
        if *visibility == Visibility::Hidden {
            continue;
        }
        let is_world_layer = glyph.layer_id.get_target_type() == RenderTargetType::World;

        if glyph.is_dormant || (is_world_layer && pos.z.floor() != player_z) {
            continue;
        }

        let texture_id = glyph.texture_id;
        let w = texture_id.get_glyph_width() * glyph.scale.0;
        let h = texture_id.get_glyph_height() * glyph.scale.1;

        let mut x = (pos.x * tile_w).floor();
        let mut y = (pos.y * tile_h).floor();

        let mut is_shrouded = false;
        let mut ignore_lighting = true;

        if is_world_layer {
            let Ok((
                is_visible,
                is_explored,
                apply_visibility_effects,
                hide_when_not_visible,
                ignore_lighting_opt,
            )) = q_visibility.get(entity)
            else {
                continue;
            };

            ignore_lighting = ignore_lighting_opt.is_some();

            if (hide_when_not_visible.is_some() && is_visible.is_none())
                || (apply_visibility_effects.is_some() && is_explored.is_none())
            {
                continue;
            }

            let world_x = x - cam_x;
            let world_y = y - cam_y;

            if world_x + w < world_left
                || world_x - w > world_right
                || world_y + h < world_top
                || world_y - h > world_bottom
            {
                continue;
            }

            x = world_x + ui_panel_x;
            y = world_y + ui_panel_y;

            is_shrouded =
                apply_visibility_effects.is_some() && is_explored.is_some() && is_visible.is_none();
        } else if x + w < 0. || x > screen_w || y + h < 0. || y > screen_h {
            continue;
        }

        let mut style = glyph.get_style();

        // Dim non-dialog layers when dialog is open
        if dialog_state.is_open
            && glyph.layer_id != Layer::DialogPanels
            && glyph.layer_id != Layer::DialogContent
        {
            style = dim_glyph_style(style);
        }

        let light_value = if is_world_layer && !ignore_lighting {
            let world_pos = pos.world();
            let (local_x, local_y) = world_to_zone_local(world_pos.0, world_pos.1);

            lighting_data
                .get_light(local_x, local_y)
                .cloned()
                .unwrap_or_else(|| LightValue {
                    rgba: Vec4::new(1.0, 1.0, 1.0, 1.0),
                    flicker_params: Vec2::ZERO,
                })
        } else {
            LightValue {
                rgba: Vec4::new(1.0, 1.0, 1.0, 1.0),
                flicker_params: Vec2::ZERO,
            }
        };

        let layer = layers.get_mut(glyph.layer_id);

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
            light_rgba: light_value.rgba,
            light_flicker: light_value.flicker_params.x,
            ignore_lighting: if ignore_lighting { 1.0 } else { 0.0 },
        });
    }

    telemetry::end_zone();
}

pub fn on_zone_status_change(mut q_changed: Query<(&mut Glyph, &ZoneStatus), Changed<ZoneStatus>>) {
    for (mut glyph, status) in q_changed.iter_mut() {
        glyph.is_dormant = *status == ZoneStatus::Dormant;
    }
}
