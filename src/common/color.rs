#![allow(dead_code)]

use macroquad::math::{Vec3, Vec4};

pub fn hex(r: u8, g: u8, b: u8) -> u32 {
    ((r as u32) << 16) + ((g as u32) << 8) + (b as u32)
}

pub trait MacroquadColorable {
    fn to_macroquad_color(&self) -> macroquad::prelude::Color;
    fn to_macroquad_color_a(&self, a: f32) -> macroquad::prelude::Color;
    fn to_rgba(&self, a: f32) -> [f32; 4];
    fn to_vec4_a(&self, a: f32) -> Vec4;
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

pub fn rgba_to_vec4(r: f32, g: f32, b: f32, a: f32) -> Vec4 {
    Vec4::new(r, g, b, a)
}

pub fn vec3_to_vec4(vec3: Vec3, a: f32) -> Vec4 {
    Vec4::new(vec3.x, vec3.y, vec3.z, a)
}

pub fn hex_to_vec4(hex: u32, a: f32) -> Vec4 {
    hex.to_vec4_a(a)
}

pub fn u32_to_vec4(color: u32, a: f32) -> Vec4 {
    color.to_vec4_a(a)
}

pub fn vec4_to_u32(color: Vec4) -> u32 {
    let r = (color.x * 255.0).clamp(0.0, 255.0).round() as u32;
    let g = (color.y * 255.0).clamp(0.0, 255.0).round() as u32;
    let b = (color.z * 255.0).clamp(0.0, 255.0).round() as u32;
    (r << 16) | (g << 8) | b
}

pub fn rgba_u8_to_u32(r: u8, g: u8, b: u8, _a: u8) -> u32 {
    ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

pub fn extract_rgb_components(color: u32) -> (u8, u8, u8) {
    let r = ((color >> 16) & 0xFF) as u8;
    let g = ((color >> 8) & 0xFF) as u8;
    let b = (color & 0xFF) as u8;
    (r, g, b)
}

pub fn dim_and_desaturate_color(color: Vec4, dim_factor: f32, desat_factor: f32) -> Vec4 {
    let desaturated = desaturate_color(color, desat_factor);
    dim_color(desaturated, dim_factor)
}

fn extract_rgb_f32(color: u32) -> (f32, f32, f32) {
    let r = ((color >> 16) & 0xFF) as f32;
    let g = ((color >> 8) & 0xFF) as f32;
    let b = (color & 0xFF) as f32;
    (r, g, b)
}

fn gamma_mix_component(p1: f32, p2: f32, t: f32) -> f32 {
    let v = (1.0 - t) * p1.powi(2) + t * p2.powi(2);
    v.sqrt()
}

fn clamp_to_u8(value: f32) -> u32 {
    (value.clamp(0.0, 255.0).round() as u32).min(255)
}

pub fn lerp_u32_colors(start_color: u32, end_color: u32, progress: f32) -> u32 {
    let progress = progress.clamp(0.0, 1.0);

    let (start_r, start_g, start_b) = extract_rgb_f32(start_color);
    let (end_r, end_g, end_b) = extract_rgb_f32(end_color);

    let r = clamp_to_u8(gamma_mix_component(start_r, end_r, progress));
    let g = clamp_to_u8(gamma_mix_component(start_g, end_g, progress));
    let b = clamp_to_u8(gamma_mix_component(start_b, end_b, progress));

    (r << 16) | (g << 8) | b
}

pub fn lerp_colors(color1: Vec4, color2: Vec4, t: f32) -> Vec4 {
    let t = t.clamp(0.0, 1.0);
    Vec4::new(
        color1.x + (color2.x - color1.x) * t,
        color1.y + (color2.y - color1.y) * t,
        color1.z + (color2.z - color1.z) * t,
        color1.w + (color2.w - color1.w) * t,
    )
}

pub fn gamma_correct_mix(color1: f32, color2: f32, t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    let gamma = 2.2;
    (color1.powf(gamma) * (1.0 - t) + color2.powf(gamma) * t).powf(1.0 / gamma)
}

pub fn luminance(color: Vec4) -> f32 {
    0.299 * color.x + 0.587 * color.y + 0.114 * color.z
}

pub fn desaturate_color(color: Vec4, factor: f32) -> Vec4 {
    let factor = factor.clamp(0.0, 1.0);
    let lum = luminance(color);

    Vec4::new(
        color.x * (1.0 - factor) + lum * factor,
        color.y * (1.0 - factor) + lum * factor,
        color.z * (1.0 - factor) + lum * factor,
        color.w,
    )
}

pub fn dim_color(color: Vec4, factor: f32) -> Vec4 {
    let factor = factor.clamp(0.0, 1.0);
    Vec4::new(
        color.x * factor,
        color.y * factor,
        color.z * factor,
        color.w,
    )
}

pub fn apply_alpha_u32(color: u32, alpha: f32) -> u32 {
    // For u32 colors, we can't store alpha directly, so this converts to RGBA format
    // This is mainly useful for conversion workflows
    let (r, g, b) = extract_rgb_components(color);
    let a = (alpha.clamp(0.0, 1.0) * 255.0).round() as u32;
    (r as u32) << 24 | (g as u32) << 16 | (b as u32) << 8 | a
}
