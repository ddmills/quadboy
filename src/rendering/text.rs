use bevy_ecs::prelude::*;
use macroquad::telemetry;

use crate::{
    common::{END_SEQ, FLAG_SEQ, Palette, PaletteSequence, START_SEQ, cp437_idx},
    engine::Time,
    rendering::{GlyphTextureId, Visibility},
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
}

#[allow(dead_code)]
impl Text {
    pub fn new(value: &str) -> Self {
        Self {
            value: value.into(),
            bg: None,
            fg1: Some(Palette::White.into()),
            fg2: None,
            outline: Some(Palette::Black.into()),
            layer_id: Layer::Ui,
            glyphs: vec![],
            texture_id: GlyphTextureId::BodyFont,
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

    pub fn outline<T: Into<u32>>(mut self, outline: T) -> Self {
        self.outline = Some(outline.into());
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

                    let mut seq = PaletteSequence::new(seq_setting.clone());
                    let glyphs = seq.apply_to(seq_value.clone(), self, tick);

                    seq_setting = String::new();
                    seq_value = String::new();

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
                    layer_id: self.layer_id,
                    texture_id: self.texture_id,
                    is_dormant: false,
                }])
            })
            .flatten()
            .collect()
    }
}

pub fn render_text(
    mut cmds: Commands,
    mut q_text: ParamSet<(
        Query<Entity, Or<(Changed<Text>, Changed<Visibility>, Changed<Position>)>>,
        Query<(Entity, &mut Text, &Position, &Visibility)>,
    )>,
    time: Res<Time>,
) {
    telemetry::begin_zone("render_text");
    let tick = (time.fixed_t * 10.).floor() as usize;

    let changed = q_text.p0().iter().collect::<Vec<_>>();

    for (entity, mut text, position, visibility) in q_text.p1().iter_mut() {
        let is_scroller = text.value.contains("scroll");

        if !(is_scroller || changed.contains(&entity)) {
            continue;
        }

        // TODO update so this re-uses existing entities instead of re-spawning.
        for glyph_id in text.glyphs.iter() {
            cmds.entity(*glyph_id).despawn();
        }

        text.glyphs = text
            .get_glyphs(tick)
            .iter()
            .enumerate()
            .map(|(i, g)| {
                cmds.spawn((
                    g.to_owned(),
                    Position::new_f32(position.x + (i as f32 * 0.5), position.y, position.z),
                    visibility.clone(),
                    ChildOf(entity),
                ))
                .id()
            })
            .collect();
    }

    telemetry::end_zone();
}
