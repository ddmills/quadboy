use crate::domain::Style;
use crate::engine::SerializableComponent;
use crate::rendering::{LightValue, world_to_zone_local};
use crate::{
    cfg::TILE_SIZE_F32,
    common::{MacroquadColorable, Palette, dim_and_desaturate_color},
    domain::{
        ApplyVisibilityEffects, HideWhenNotVisible, IgnoreLighting, IsExplored, IsVisible, Player,
        ZoneStatus,
    },
    rendering::{GlyphTextureId, RenderTargetType, Visibility},
    tracy_plot,
    tracy_span,
    ui::{DialogState, UiLayout},
};
use bevy_ecs::prelude::*;
use macroquad::prelude::*;
use quadboy_macros::profiled_system;
use serde::{Deserialize, Serialize};

use super::{GameCamera, Layer, Layers, LightingData, Position, Renderable, ScreenSize};

fn default_alpha() -> f32 {
    1.0
}

#[derive(Component, Serialize, Deserialize, Clone, Copy, PartialEq, SerializableComponent)]
#[require(Visibility)]
pub struct Glyph {
    pub idx: usize,
    pub fg1: Option<u32>,
    pub fg2: Option<u32>,
    pub bg: Option<u32>,
    pub outline: Option<u32>,
    pub outline_override: Option<u32>,
    pub position_offset: Option<(f32, f32)>,
    pub scale: (f32, f32),
    pub layer_id: Layer,
    pub texture_id: GlyphTextureId,
    pub is_dormant: bool,
    #[serde(default = "default_alpha")]
    pub alpha: f32,
}

impl Default for Glyph {
    fn default() -> Self {
        Self {
            idx: 0,
            fg1: None,
            fg2: None,
            bg: None,
            outline: None,
            outline_override: None,
            position_offset: None,
            scale: (1.0, 1.0),
            layer_id: Layer::Overlay,
            texture_id: GlyphTextureId::Cowboy,
            is_dormant: false,
            alpha: 1.0,
        }
    }
}

#[derive(Resource)]
pub struct TilesetRegistry {
    pub glyph_texture: Texture2D,
    pub font_body_texture: Texture2D,
    pub creatures_texture: Texture2D,
    pub bitmasks_texture: Texture2D,
}

impl TilesetRegistry {
    pub async fn load() -> Self {
        let glyph_texture_fut = load_texture("./src/assets/textures/cowboy.png");
        // let font_body_texture_fut = load_texture("./src/assets/textures/bizcat_8x12.png");
        // let font_body_texture_fut = load_texture("./src/assets/textures/acer_8x12.png");
        // let font_body_texture_fut = load_texture("./src/assets/textures/scroll_8x12.png");
        // let font_body_texture_fut = load_texture("./src/assets/textures/tamzen_8x12.png");
        let font_body_texture_fut = load_texture("./src/assets/textures/tocky_8x12.png");
        let creatures_texture_fut = load_texture("./src/assets/textures/creatures.png");
        let bitmasks_texture_fut = load_texture("./src/assets/textures/bitmasks.png");

        let glyph_texture = glyph_texture_fut.await.unwrap();
        let font_body_texture = font_body_texture_fut.await.unwrap();
        let creatures_texture = creatures_texture_fut.await.unwrap();
        let bitmasks_texture = bitmasks_texture_fut.await.unwrap();

        TilesetRegistry {
            glyph_texture,
            font_body_texture,
            creatures_texture,
            bitmasks_texture,
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

const DEFAULT_LIGHT: LightValue = LightValue {
    rgb: Vec3::new(1.0, 1.0, 1.0),
    intensity: 1.,
    flicker: 0.,
};

#[allow(dead_code)]
impl Glyph {
    pub fn idx(idx: usize) -> Self {
        Self {
            idx,
            fg1: None,
            fg2: None,
            bg: None,
            outline: Some(Palette::Clear.into()),
            outline_override: None,
            position_offset: None,
            layer_id: Layer::default(),
            texture_id: GlyphTextureId::Cowboy,
            is_dormant: false,
            scale: (1., 1.),
            alpha: 1.0,
        }
    }

    pub fn new_from_style(style: Style) -> Self {
        Self {
            idx: style.idx,
            fg1: style.fg1.map(|x| x.into()),
            fg2: style.fg2.map(|x| x.into()),
            bg: style.bg.map(|x| x.into()),
            outline: style.outline.map(|x| x.into()),
            outline_override: None,
            position_offset: None,
            layer_id: Layer::default(),
            texture_id: GlyphTextureId::Cowboy,
            is_dormant: false,
            scale: (1., 1.),
            alpha: 1.0,
        }
    }

    pub fn new<T: Into<u32>>(idx: usize, fg1: T, fg2: T) -> Self {
        Self {
            idx,
            fg1: Some(fg1.into()),
            fg2: Some(fg2.into()),
            bg: None,
            outline: Some(Palette::Clear.into()),
            outline_override: None,
            position_offset: None,
            layer_id: Layer::default(),
            texture_id: GlyphTextureId::Cowboy,
            is_dormant: false,
            scale: (1., 1.),
            alpha: 1.0,
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

    pub fn alpha(mut self, value: f32) -> Self {
        self.alpha = value;
        self
    }

    pub fn get_style(&self) -> GlyphStyle {
        if self.is_dormant {
            return GlyphStyle {
                bg: self
                    .bg
                    .map(|_| SHROUD_BG_COLOR.to_vec4_a(self.alpha))
                    .unwrap_or(TRANSPARENT),
                fg1: SHROUD_FG_COLOR.to_vec4_a(self.alpha),
                fg2: SHROUD_FG_COLOR.to_vec4_a(self.alpha),
                outline: self
                    .outline
                    .map(|_| SHROUD_OUTLINE_COLOR.to_vec4_a(self.alpha))
                    .unwrap_or(TRANSPARENT),
            };
        }

        GlyphStyle {
            bg: self
                .bg
                .map(|x| x.to_vec4_a(self.alpha))
                .unwrap_or(TRANSPARENT),
            fg1: self
                .fg1
                .map(|x| x.to_vec4_a(self.alpha))
                .unwrap_or(TRANSPARENT),
            fg2: self
                .fg2
                .map(|x| x.to_vec4_a(self.alpha))
                .unwrap_or(TRANSPARENT),
            outline: self
                .outline_override
                .or(self.outline)
                .map(|x| x.to_vec4_a(self.alpha))
                .unwrap_or(TRANSPARENT),
        }
    }
}

/// Apply dimming and desaturation effect to glyph style when dialog is open
fn dim_glyph_style(style: GlyphStyle) -> GlyphStyle {
    const DESATURATION_FACTOR: f32 = 0.9;

    GlyphStyle {
        fg1: dim_and_desaturate_color(style.fg1, DIMMING_FACTOR, DESATURATION_FACTOR),
        fg2: dim_and_desaturate_color(style.fg2, DIMMING_FACTOR, DESATURATION_FACTOR),
        bg: dim_and_desaturate_color(style.bg, DIMMING_FACTOR, DESATURATION_FACTOR),
        outline: Palette::Clear.to_vec4_a(1.),
    }
}

#[profiled_system]
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
    tracy_plot!("Rendered Glyphs", q_glyphs.iter().count() as f64);

    {
        tracy_span!("clear_layers");
        layers.iter_mut().for_each(|layer| {
            layer.clear();
        });
    }

    let (
        screen_w,
        screen_h,
        tile_w,
        tile_h,
        cam_x,
        cam_y,
        _camera_width,
        _camera_height,
        ui_panel_x,
        ui_panel_y,
        player_z,
        world_left,
        world_right,
        world_top,
        world_bottom,
    ) = {
        tracy_span!("setup_camera_params");
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

        (
            screen_w,
            screen_h,
            tile_w,
            tile_h,
            cam_x,
            cam_y,
            camera_width,
            camera_height,
            ui_panel_x,
            ui_panel_y,
            player_z,
            world_left,
            world_right,
            world_top,
            world_bottom,
        )
    };

    let should_dim_for_dialog = dialog_state.is_open;

    {
        tracy_span!("glyph_iteration");
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

            if let Some((offset_x, offset_y)) = glyph.position_offset {
                x += offset_x * tile_w;
                y += offset_y * tile_h;
            }

            if is_world_layer {
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
            } else if x + w < 0. || x > screen_w || y + h < 0. || y > screen_h {
                continue;
            }

            let mut is_shrouded = false;
            let mut ignore_lighting = true;

            if is_world_layer {
                tracy_span!("visibility_check");
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

                is_shrouded = apply_visibility_effects.is_some()
                    && is_explored.is_some()
                    && is_visible.is_none();
            }

            let mut style = glyph.get_style();

            if should_dim_for_dialog
                && glyph.layer_id != Layer::DialogPanels
                && glyph.layer_id != Layer::DialogContent
            {
                style = dim_glyph_style(style);
            }

            let light_value = if is_world_layer && !ignore_lighting {
                tracy_span!("lighting_lookup");
                let world_pos = pos.world();
                let (local_x, local_y) = world_to_zone_local(world_pos.0, world_pos.1);

                *lighting_data
                    .get_light(local_x, local_y)
                    .unwrap_or(&DEFAULT_LIGHT)
            } else {
                DEFAULT_LIGHT
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
                light_rgba: Vec4::new(
                    light_value.rgb.x,
                    light_value.rgb.y,
                    light_value.rgb.z,
                    light_value.intensity,
                ),
                light_flicker: light_value.flicker,
                ignore_lighting: if ignore_lighting { 1.0 } else { 0.0 },
            });
        }
    }
}

pub fn on_zone_status_change(mut q_changed: Query<(&mut Glyph, &ZoneStatus), Changed<ZoneStatus>>) {
    for (mut glyph, status) in q_changed.iter_mut() {
        glyph.is_dormant = *status == ZoneStatus::Dormant;
    }
}
