use bevy_ecs::prelude::*;

use crate::common::{cp437_idx, Palette};

use super::{Glyph, Position, RenderLayer};

#[derive(Component)]
pub struct Text {
    pub value: String,
    pub bg: Option<u32>,
    pub fg1: Option<u32>,
    pub fg2: Option<u32>,
    pub outline: Option<u32>,
    pub layer_id: RenderLayer,
    glyphs: Vec<Entity>,
}

impl Text {
    pub fn new(value: &str) -> Self {
        Self {
            value: value.into(),
            bg: None,
            fg1: Some(Palette::White.into()),
            fg2: None,
            outline: None,
            layer_id: RenderLayer::Text,
            glyphs: vec![],
        }
    }

    pub fn bg<T: Into<u32>>(mut self, bg: T) -> Self
    {
        self.bg = Some(bg.into());
        self
    }
}

pub fn render_text(
    mut cmds: Commands,
    mut q_text: Query<(&mut Text, &Position), Changed<Text>>,
) {
    for (mut text, position) in q_text.iter_mut() {
        for glyph_id in text.glyphs.iter() {
            cmds.entity(*glyph_id).despawn();
        }

        text.glyphs = text.value.chars().enumerate().map(|(i, c)| {
            cmds.spawn((
                Glyph {
                    idx: cp437_idx(c).unwrap_or(0),
                    fg1: text.fg1,
                    fg2: text.fg2,
                    bg: text.bg,
                    outline: text.outline,
                    layer_id: text.layer_id,
                },
                Position::new_f32(position.x + (i as f32 * 0.5), position.y),
            )).id()
        }).collect();
    }
}
