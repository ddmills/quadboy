use bevy_ecs::prelude::*;
use macroquad::math::Vec2;

use super::core::{GlyphAnimation, ParticleSpawner};
use super::curves::{AlphaCurve, ColorCurve, VelocityCurve};
use super::spawn_areas::{Distribution, SpawnArea};
use crate::domain::ConditionType;

impl ConditionType {
    /// Create a particle spawner configuration for this condition type
    /// Returns None if this condition type doesn't have visual effects
    pub fn create_particle_spawner(&self) -> Option<ParticleSpawner> {
        match self {
            ConditionType::Poisoned { .. } => {
                Some(ParticleSpawner::new(Vec2::ZERO)
                    .glyph_animation(GlyphAnimation::RandomPool {
                        glyphs: vec!['○', '◦', '•', 'o'],
                        change_rate: Some(3.0),
                        last_change: 0.0,
                    })
                    .color_curve(ColorCurve::Linear {
                        values: vec![0x00FF00, 0x00AA00], // Bright to darker green
                    })
                    .alpha_curve(AlphaCurve::EaseOut {
                        values: vec![0.7, 0.0],
                    })
                    .velocity_curve(VelocityCurve::Linear {
                        values: vec![Vec2::new(0.0, -1.0), Vec2::new(0.0, -0.5)], // Float upward slowly
                    })
                    .spawn_area(SpawnArea::Circle {
                        radius: 0.5,
                        distribution: Distribution::Uniform,
                    })
                    .priority(140) // Above normal particles but below UI
                    .spawn_rate(2.0) // 2 particles per second
                    .lifetime_range(1.0..2.0))
            },

            ConditionType::Burning { .. } => {
                Some(ParticleSpawner::new(Vec2::ZERO)
                    .glyph_animation(GlyphAnimation::RandomPool {
                        glyphs: vec!['*', '✦', '●', '○'],
                        change_rate: Some(8.0),
                        last_change: 0.0,
                    })
                    .color_curve(ColorCurve::Linear {
                        values: vec![0xFFAA00, 0xFF4400], // Orange to red flames
                    })
                    .alpha_curve(AlphaCurve::EaseOut {
                        values: vec![0.8, 0.0],
                    })
                    .velocity_curve(VelocityCurve::Linear {
                        values: vec![Vec2::new(0.0, -1.5), Vec2::new(0.0, -0.8)], // Rise faster than poison
                    })
                    .spawn_area(SpawnArea::Circle {
                        radius: 0.4,
                        distribution: Distribution::Gaussian, // Concentrate near center
                    })
                    .priority(145) // Slightly higher than poison
                    .spawn_rate(4.0) // More frequent than poison
                    .lifetime_range(0.8..1.5))
            },

            ConditionType::Bleeding { .. } => {
                Some(ParticleSpawner::new(Vec2::ZERO)
                    .glyph_animation(GlyphAnimation::RandomPool {
                        glyphs: vec!['*', '•', '○', '.'],
                        change_rate: Some(5.0),
                        last_change: 0.0,
                    })
                    .color_curve(ColorCurve::Linear {
                        values: vec![0xCC0000, 0x880000], // Bright to dark red
                    })
                    .alpha_curve(AlphaCurve::EaseOut {
                        values: vec![0.9, 0.0],
                    })
                    .velocity_curve(VelocityCurve::EaseOut {
                        values: vec![Vec2::new(0.0, 0.5), Vec2::new(0.0, 2.0)], // Drop down with gravity
                    })
                    .spawn_area(SpawnArea::Circle {
                        radius: 0.3,
                        distribution: Distribution::EdgeOnly, // Drip from edges
                    })
                    .gravity(Vec2::new(0.0, 1.5)) // Blood drops down
                    .priority(150) // Higher than other conditions
                    .spawn_rate(1.5) // Slower drip rate
                    .lifetime_range(1.5..3.0))
            },

            // Conditions that don't need particle effects (behavioral effects)
            ConditionType::Feared { .. } |
            ConditionType::Taunted { .. } |
            ConditionType::Confused { .. } => None,
        }
    }
}