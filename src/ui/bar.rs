use bevy_ecs::prelude::*;
use macroquad::prelude::trace;
use serde::{Deserialize, Serialize};

use crate::{
    common::{Palette, palette_to_char},
    engine::{SerializableComponent, Time},
    rendering::Text,
};

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct Bar {
    pub current: usize,
    pub max: usize,
    pub width: usize,
    pub color: Palette,
    pub color_empty: Palette,
    pub bg: Option<Palette>,
    pub display_value: f32,
    pub animation_speed: f32,
}

impl Bar {
    pub fn new(
        current: usize,
        max: usize,
        width: usize,
        color: Palette,
        color_empty: Palette,
    ) -> Self {
        trace!("New bar: {}/{} width={}", current, max, width);
        Self {
            current,
            max,
            width,
            color,
            color_empty,
            bg: None,
            display_value: current as f32,
            animation_speed: 5.0,
        }
    }

    pub fn with_bg(mut self, bg: Palette) -> Self {
        self.bg = Some(bg);
        self
    }

    pub fn generate_bar_text(&self) -> String {
        if self.max == 0 || self.width == 0 {
            return String::new();
        }

        let display_current = self.display_value.round() as usize;
        let sections_filled = ((display_current * self.width * 2) / self.max).min(self.width * 2);
        let full_sections = sections_filled / 2;
        let has_half_section = sections_filled % 2 == 1;
        let empty_sections = self.width - full_sections - if has_half_section { 1 } else { 0 };

        let color_char = palette_to_char(self.color);
        let empty_color_char = palette_to_char(self.color_empty);

        let mut result = String::new();

        if full_sections > 0 {
            result.push_str(&format!("{{{}|{}}}", color_char, "½".repeat(full_sections)));
        }

        if has_half_section {
            result.push_str(&format!("{{{}|¼}}", color_char));
        }

        if empty_sections > 0 {
            result.push_str(&format!(
                "{{{}|{}}}",
                empty_color_char,
                "½".repeat(empty_sections)
            ));
        }

        result
    }

    pub fn update_values(&mut self, current: usize, max: usize) {
        self.current = current;
        self.max = max;
    }
}

pub fn update_bars(mut q_bars: Query<(&mut Bar, &mut Text)>, time: Res<Time>) {
    for (mut bar, mut text) in q_bars.iter_mut() {
        // Interpolate display_value toward current
        let target = bar.current as f32;
        let diff = target - bar.display_value;

        // Only interpolate if there's a significant difference
        if diff.abs() > 0.01 {
            bar.display_value += diff * bar.animation_speed * time.dt;

            // Snap to target if we're very close
            if diff.abs() < 0.1 {
                bar.display_value = target;
            }
        }

        let new_text = bar.generate_bar_text();
        if text.value != new_text {
            text.value = new_text;
        }
        text.fg1 = Some(bar.color.into());
        text.fg2 = Some(bar.color_empty.into());
        text.bg = bar.bg.map(|x| x.into());
    }
}
