use bevy_ecs::prelude::*;
use macroquad::math::{Vec3, Vec4};

use crate::{cfg::ZONE_SIZE, common::Grid};

#[derive(Clone, Copy, Default)]
pub struct LightValue {
    pub rgb: Vec3,
    pub intensity: f32,
    pub flicker: f32,
}

impl LightValue {}

#[derive(Resource)]
pub struct LightingData {
    light_map: Grid<LightValue>,
    ambient_color: u32,
    ambient_intensity: f32,
}

impl Default for LightingData {
    fn default() -> Self {
        Self {
            light_map: Grid::init(ZONE_SIZE.0, ZONE_SIZE.1, LightValue::default()),
            ambient_color: 0x1A131A,
            ambient_intensity: 0.1,
        }
    }
}

impl LightingData {
    pub fn set_ambient(&mut self, color: u32, intensity: f32) {
        self.ambient_color = color;
        self.ambient_intensity = intensity;
    }

    pub fn clear(&mut self) {
        self.light_map.clear(LightValue::default());
    }

    pub fn blend_light(
        &mut self,
        x: i32,
        y: i32,
        r: f32,
        g: f32,
        b: f32,
        intensity: f32,
        flicker: f32,
    ) {
        if x < 0 || y < 0 || x >= ZONE_SIZE.0 as i32 || y >= ZONE_SIZE.1 as i32 {
            return;
        }

        if let Some(current) = self.light_map.get_mut(x as usize, y as usize) {
            let curr_intensity = current.intensity;
            let new_total = curr_intensity + intensity;

            if new_total > 0.0 {
                // Blend colors weighted by intensity
                let curr_weight = curr_intensity / new_total;
                let new_weight = intensity / new_total;

                current.rgb.x = current.rgb.x * curr_weight + r * new_weight;
                current.rgb.y = current.rgb.y * curr_weight + g * new_weight;
                current.rgb.z = current.rgb.z * curr_weight + b * new_weight;
                current.intensity = new_total.min(1.0); // Cap at 1.0

                // Blend flicker parameters weighted by light intensity
                if flicker > 0.0 {
                    current.flicker = (current.flicker * curr_weight) + (flicker * new_weight);
                }
            }
        }
    }

    pub fn get_light(&self, x: usize, y: usize) -> Option<&LightValue> {
        self.light_map.get(x, y)
    }

    pub fn get_ambient_color(&self) -> u32 {
        self.ambient_color
    }

    pub fn get_ambient_intensity(&self) -> f32 {
        self.ambient_intensity
    }

    pub fn get_ambient_vec4(&self) -> Vec4 {
        let r = ((self.ambient_color >> 16) & 0xFF) as f32 / 255.0;
        let g = ((self.ambient_color >> 8) & 0xFF) as f32 / 255.0;
        let b = (self.ambient_color & 0xFF) as f32 / 255.0;
        Vec4::new(r, g, b, self.ambient_intensity)
    }
}
