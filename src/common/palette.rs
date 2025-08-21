use macroquad::math::Vec4;

use crate::{
    common::cp437_idx,
    rendering::{Glyph, Text},
};

#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Palette {
    White = 0xFFF1E8,
    Black = 0x0F0E0F,
    Gray = 0x5A5353,
    Green = 0x3EBD3E,
    DarkGreen = 0x265C42,
    Brown = 0x8B4513,
    DarkBrown = 0x4A2C2A,
    Blue = 0x4A90E2,
    DarkBlue = 0x2C5282,
    Red = 0xE74C3C,
    DarkRed = 0x8B1538,
    Orange = 0xFF8C42,
    DarkOrange = 0xA0522D,
    Yellow = 0xF1C40F,
    DarkYellow = 0x8B7D3A,
    Purple = 0x9B59B6,
    DarkPurple = 0x663366,
    Cyan = 0x1ABC9C,
    DarkCyan = 0x16A085,
    Clear = 0x201820,
}

#[allow(dead_code)]
pub fn hex(r: u8, g: u8, b: u8) -> u32 {
    ((r as u32) << 16) + ((g as u32) << 8) + (b as u32)
}

#[allow(dead_code)]
pub trait MacroquadColorable {
    fn to_macroquad_color(&self) -> macroquad::prelude::Color;
    fn to_macroquad_color_a(&self, a: f32) -> macroquad::prelude::Color;
    fn to_rgba(&self, a: f32) -> [f32; 4];
    fn to_vec4_a(&self, a: f32) -> Vec4;
}

impl From<Palette> for macroquad::prelude::Color {
    fn from(value: Palette) -> Self {
        value.to_macroquad_color()
    }
}

impl MacroquadColorable for u32 {
    fn to_macroquad_color(&self) -> macroquad::prelude::Color {
        let b = (self & 0xff) as u8;
        let g = ((self >> 8) & 0xff) as u8;
        let r = ((self >> 16) & 0xff) as u8;

        macroquad::prelude::Color::from_rgba(r, g, b, 255)
    }

    #[inline]
    fn to_macroquad_color_a(&self, a: f32) -> macroquad::prelude::Color {
        self.to_macroquad_color().with_alpha(a)
    }

    #[inline]
    fn to_rgba(&self, a: f32) -> [f32; 4] {
        [
            ((self >> 16) & 0xff) as f32 / 255.,
            ((self >> 8) & 0xff) as f32 / 255.,
            (self & 0xff) as f32 / 255.,
            a,
        ]
    }

    #[inline]
    fn to_vec4_a(&self, a: f32) -> Vec4 {
        Vec4::new(
            ((self >> 16) & 0xff) as f32 / 255.,
            ((self >> 8) & 0xff) as f32 / 255.,
            (self & 0xff) as f32 / 255.,
            a,
        )
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
        _ => Palette::White,
    }
}

enum PaletteSequenceType {
    Solid,
    Repeat,
    Stretch,
    Border,
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
                };

                Glyph {
                    idx: cp437_idx(c).unwrap_or(0),
                    fg1: Some(fg1.into()),
                    fg2: text.fg2,
                    bg: text.bg,
                    outline: text.outline,
                    layer_id: text.layer_id,
                    texture_id: text.texture_id,
                    is_dormant: false,
                }
            })
            .collect()
    }
}
