use macroquad::math::Vec4;

#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Palette {
    White = 0xD2DBDB,
    Black = 0x161A1F,
    Green = 0x2F812F,
    LightGreen = 0x1AAF1A,
    Brown = 0x664D3C,
    Blue = 0x294E94,
    LightBlue = 0x608ED3,
    Red = 0xA83A3A,
    Orange = 0xE79519,
    Yellow = 0xEBCC21,
    Purple = 0xAF0BB4,
    Cyan = 0x0EC9E2,
    DarkCyan = 0x2C7983,
}

pub fn hex(r: u8, g: u8, b: u8) -> u32 {
    ((r as u32) << 16) + ((g as u32) << 8) + (b as u32)
}

pub trait MacroquadColorable {
    fn to_macroquad_color(&self) -> macroquad::prelude::Color;
    fn to_macroquad_color_a(&self, a: f32) -> macroquad::prelude::Color;
    fn to_rgba(&self, a: f32) -> [f32; 4];
    fn to_vec4_a(&self, a: f32) -> Vec4;
    fn to_vec4(&self) -> Vec4;
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
            a
        ]
    }
    
    #[inline]
    fn to_vec4_a(&self, a: f32) -> Vec4 {
        Vec4::new(
            ((self >> 16) & 0xff) as f32 / 255.,
            ((self >> 8) & 0xff) as f32 / 255.,
            (self & 0xff) as f32 / 255.,
            a
        )
    }

    #[inline]
    fn to_vec4(&self) -> Vec4 {
        self.to_vec4_a(1.)
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
    
    #[inline]
    fn to_vec4(&self) -> Vec4 {
        (*self as u32).to_vec4_a(1.0)
    }
}

impl std::convert::From<Palette> for u32 {
    fn from(val: Palette) -> Self {
        val as u32
    }
}
