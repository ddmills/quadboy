use macroquad::math::Vec4;

use crate::{
    common::{color::MacroquadColorable, cp437_idx},
    rendering::{Glyph, Text},
};

#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Palette {
    White = 0xFFEAC5,
    Black = 0x140E08,
    Gray = 0x7D91A7,
    DarkGray = 0x575057,
    Green = 0x7DB441,
    DarkGreen = 0x395C20,
    Brown = 0x994318,
    DarkBrown = 0x632E15,
    Blue = 0x53ACE7,
    DarkBlue = 0x1E435C,
    Red = 0xC91121,
    DarkRed = 0x58241F,
    Orange = 0xFF7A27,
    DarkOrange = 0xC2722D,
    Yellow = 0xFFD208,
    DarkYellow = 0xD68910,
    Purple = 0xC467EC,
    DarkPurple = 0x8E44AD,
    Cyan = 0x43D4B7,
    DarkCyan = 0x15755D,
    Clear = 0xFFFF00,
}

impl From<Palette> for macroquad::prelude::Color {
    fn from(value: Palette) -> Self {
        value.to_macroquad_color()
    }
}

impl MacroquadColorable for Palette {
    #[inline]
    fn to_macroquad_color(&self) -> macroquad::prelude::Color {
        (*self as u32).to_macroquad_color()
    }

    #[inline]
    fn to_macroquad_color_a(&self, a: f32) -> macroquad::prelude::Color {
        (*self as u32).to_macroquad_color_a(a)
    }

    #[inline]
    fn to_rgba(&self, a: f32) -> [f32; 4] {
        (*self as u32).to_rgba(a)
    }

    #[inline]
    fn to_vec4_a(&self, a: f32) -> Vec4 {
        (*self as u32).to_vec4_a(a)
    }
}

impl std::convert::From<Palette> for u32 {
    fn from(val: Palette) -> Self {
        val as u32
    }
}

pub const START_SEQ: char = '{';
pub const END_SEQ: char = '}';
pub const FLAG_SEQ: char = '|';

fn get_seq_color(ch: &str) -> Palette {
    match ch {
        "W" => Palette::White,
        "w" => Palette::White,
        "R" => Palette::Red,
        "r" => Palette::DarkRed,
        "U" => Palette::Gray,
        "u" => Palette::DarkGray,
        "G" => Palette::Green,
        "g" => Palette::DarkGreen,
        "B" => Palette::Blue,
        "b" => Palette::DarkBlue,
        "Y" => Palette::Yellow,
        "y" => Palette::DarkYellow,
        "C" => Palette::Cyan,
        "c" => Palette::DarkCyan,
        "O" => Palette::Orange,
        "o" => Palette::DarkOrange,
        "P" => Palette::Purple,
        "p" => Palette::DarkPurple,
        "X" => Palette::Brown,
        "x" => Palette::DarkBrown,
        _ => Palette::White,
    }
}

enum PaletteSequenceType {
    Solid,
    Repeat,
    Stretch,
    Border,
    Flash,
    Scroll,
    ScrollFast,
}

impl PaletteSequenceType {
    pub fn from_str(val: &str) -> PaletteSequenceType {
        match val {
            "solid" => Self::Solid,
            "repeat" => Self::Repeat,
            "stretch" => Self::Stretch,
            "border" => Self::Border,
            "flash" => Self::Flash,
            "scroll" => Self::Scroll,
            "scrollf" => Self::ScrollFast,
            _ => Self::Solid,
        }
    }
}

pub struct PaletteSequence {
    seq_type: PaletteSequenceType,
    seq_colors: Vec<Palette>,
}

impl PaletteSequence {
    pub fn new(value: String) -> Self {
        let split = value.split(' ').collect::<Vec<_>>();
        let mut seq_type = PaletteSequenceType::Repeat;
        let mut seq_colors = value.clone();

        if split.len() == 2 {
            seq_type = PaletteSequenceType::from_str(split[1]);
            seq_colors = split[0].to_string();
        }

        let mut colors = seq_colors.split('-').map(get_seq_color).collect::<Vec<_>>();

        if colors.is_empty() {
            colors = vec![Palette::White];
        }

        Self {
            seq_colors: colors,
            seq_type,
        }
    }

    pub fn apply_to(&mut self, value: String, text: &Text, tick: usize) -> Vec<Glyph> {
        let color_len = self.seq_colors.len();
        let value_len = value.len();

        value
            .chars()
            .enumerate()
            .map(|(idx, c)| {
                let fg1 = match self.seq_type {
                    PaletteSequenceType::Solid => *self.seq_colors.first().unwrap(),
                    PaletteSequenceType::Repeat => *self.seq_colors.get(idx % color_len).unwrap(),
                    PaletteSequenceType::Stretch => {
                        let dist = idx as f32 / value_len as f32;
                        let new_idx = (dist * color_len as f32).floor() as usize;
                        *self.seq_colors.get(new_idx).unwrap()
                    }
                    PaletteSequenceType::Border => {
                        if idx == 0 || idx == value_len - 1 {
                            *self.seq_colors.first().unwrap()
                        } else {
                            *self.seq_colors.get(1 % color_len).unwrap()
                        }
                    }
                    PaletteSequenceType::Scroll => {
                        *self.seq_colors.get((idx + tick / 2) % color_len).unwrap()
                    }
                    PaletteSequenceType::ScrollFast => {
                        *self.seq_colors.get((idx + tick) % color_len).unwrap()
                    }
                    PaletteSequenceType::Flash => *self.seq_colors.get((tick) % color_len).unwrap(),
                };

                Glyph {
                    idx: cp437_idx(c).unwrap_or(0),
                    fg1: Some(fg1.into()),
                    fg2: text.fg2,
                    bg: text.bg,
                    outline: text.outline,
                    outline_override: None,
                    position_offset: None,
                    layer_id: text.layer_id,
                    texture_id: text.texture_id,
                    is_dormant: false,
                    scale: (1.0, 1.0),
                    alpha: 1.0,
                }
            })
            .collect()
    }
}
