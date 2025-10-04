use std::collections::HashSet;

use bevy_ecs::prelude::*;
use quadboy_macros::profiled_system;

use crate::{
    common::{END_SEQ, FLAG_SEQ, Palette, PaletteSequence, START_SEQ, cp437_idx},
    domain::IgnoreLighting,
    engine::Time,
    rendering::{GlyphTextureId, Visibility},
    tracy_span,
};

use super::{Glyph, Layer, Position};

#[derive(Component)]
#[require(Visibility)]
pub struct Text {
    pub value: String,
    pub bg: Option<u32>,
    pub fg1: Option<u32>,
    pub fg2: Option<u32>,
    pub outline: Option<u32>,
    pub layer_id: Layer,
    pub glyphs: Vec<Entity>,
    pub texture_id: GlyphTextureId,
    pub cached_glyphs: Vec<Glyph>,
    pub cached_tick: usize,
    pub update_every_frame: bool,
}

#[allow(dead_code)]
impl Text {
    pub fn new(value: &str) -> Self {
        Self {
            value: value.into(),
            bg: None,
            fg1: Some(Palette::White.into()),
            fg2: Some(Palette::Black.into()),
            outline: Some(Palette::Black.into()),
            layer_id: Layer::Ui,
            glyphs: vec![],
            texture_id: GlyphTextureId::BodyFont,
            cached_glyphs: vec![],
            cached_tick: 0,
            update_every_frame: false,
        }
    }

    pub fn layer(mut self, layer_id: Layer) -> Self {
        self.layer_id = layer_id;
        self
    }

    pub fn bg<T: Into<u32>>(mut self, bg: T) -> Self {
        self.bg = Some(bg.into());
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

    pub fn outline<T: Into<u32>>(mut self, outline: T) -> Self {
        self.outline = Some(outline.into());
        self
    }

    pub fn texture(mut self, texture_id: GlyphTextureId) -> Self {
        self.texture_id = texture_id;
        self
    }

    pub fn get_glyphs(&self, tick: usize) -> Vec<Glyph> {
        let mut in_seq = false;
        let mut in_flags = false;
        let mut seq_setting = String::new();
        let mut seq_value = String::new();

        self.value
            .chars()
            .filter_map(|c| {
                if c == START_SEQ {
                    in_seq = true;
                    in_flags = true;
                    return None;
                }

                if in_seq && c == END_SEQ {
                    in_seq = false;
                    in_flags = false;

                    let mut seq = PaletteSequence::new(std::mem::take(&mut seq_setting));
                    let glyphs = seq.apply_to(std::mem::take(&mut seq_value), self, tick);

                    return Some(glyphs);
                }

                if in_seq && c == FLAG_SEQ {
                    in_flags = false;
                    return None;
                }

                if in_flags {
                    seq_setting.push(c);
                    return None;
                }

                if in_seq {
                    seq_value.push(c);
                    return None;
                }

                Some(vec![Glyph {
                    idx: cp437_idx(c).unwrap_or(0),
                    fg1: self.fg1,
                    fg2: self.fg2,
                    bg: self.bg,
                    outline: self.outline,
                    outline_override: None,
                    position_offset: None,
                    layer_id: self.layer_id,
                    texture_id: self.texture_id,
                    is_dormant: false,
                    scale: (1.0, 1.0),
                    alpha: 1.0,
                }])
            })
            .flatten()
            .collect()
    }
}

#[profiled_system]
pub fn render_text(
    mut cmds: Commands,
    mut q_text: ParamSet<(
        Query<Entity, Or<(Changed<Text>, Changed<Visibility>, Changed<Position>)>>,
        Query<(
            Entity,
            &mut Text,
            &Position,
            &Visibility,
            Option<&IgnoreLighting>,
        )>,
    )>,
    time: Res<Time>,
) {
    let tick = (time.fixed_t * 10.).floor() as usize;

    let changed = {
        tracy_span!("collect_changed_entities");
        q_text.p0().iter().collect::<HashSet<_>>()
    };

    {
        for (entity, mut text, position, visibility, ignore_lighting_opt) in q_text.p1().iter_mut()
        {
            let needs_regeneration = text.update_every_frame
                || changed.contains(&entity)
                || text.cached_tick != tick;

            if !needs_regeneration {
                continue;
            }

            let ignore_lighting = ignore_lighting_opt.is_some();

            let glyphs = {
                tracy_span!("generate_glyphs");
                let new_glyphs = text.get_glyphs(tick);
                text.cached_tick = tick;

                if new_glyphs == text.cached_glyphs && !changed.contains(&entity) {
                    continue;
                }

                text.cached_glyphs = new_glyphs.clone();
                new_glyphs
            };

            {
                tracy_span!("update_glyph_entities");
                let old_len = text.glyphs.len();
                let new_len = glyphs.len();

                if new_len > old_len {
                    tracy_span!("spawn_new_glyphs");
                    for i in old_len..new_len {
                        let g = &glyphs[i];
                        let mut ecmds = cmds.spawn((
                            *g,
                            Position::new_f32(position.x + (i as f32 * 0.5), position.y, position.z),
                            *visibility,
                            ChildOf(entity),
                        ));

                        if ignore_lighting {
                            ecmds.insert(IgnoreLighting);
                        }

                        text.glyphs.push(ecmds.id());
                    }
                } else if new_len < old_len {
                    tracy_span!("despawn_extra_glyphs");
                    for glyph_id in text.glyphs.drain(new_len..) {
                        cmds.entity(glyph_id).despawn();
                    }
                }

                {
                    tracy_span!("update_existing_glyphs");
                    for (i, (glyph_id, g)) in text.glyphs.iter().zip(glyphs.iter()).enumerate() {
                        cmds.entity(*glyph_id).insert((
                            *g,
                            Position::new_f32(position.x + (i as f32 * 0.5), position.y, position.z),
                            *visibility,
                        ));
                    }
                }
            }
        }
    }
}

pub fn text_content_length(value: &str) -> usize {
    let mut in_seq = false;
    let mut in_flags = false;
    let mut length = 0;

    for c in value.chars() {
        if c == START_SEQ {
            in_seq = true;
            in_flags = true;
            continue;
        }

        if in_seq && c == END_SEQ {
            in_seq = false;
            in_flags = false;
            continue;
        }

        if in_seq && c == FLAG_SEQ {
            in_flags = false;
            continue;
        }

        if in_flags {
            continue;
        }

        length += 1;
    }

    length
}
