use bevy_ecs::prelude::*;
use macroquad::math::Vec2;

use super::core::{GlyphAnimation, ParticleSpawner, SequenceEasing, SequenceTiming};
use super::curves::{AlphaCurve, ColorCurve, VelocityCurve};
use super::spawn_areas::{Distribution, SpawnArea};
use crate::domain::ConditionType;

impl ConditionType {
    /// Create a particle spawner configuration for this condition type
    /// Returns None if this condition type doesn't have visual effects
    pub fn create_particle_spawner(&self) -> Option<ParticleSpawner> {
        match self {
            ConditionType::Poisoned { .. } => Some(
                ParticleSpawner::new(Vec2::ZERO)
                    .glyph_animation(GlyphAnimation::RandomPool {
                        glyphs: vec!['·', '•', '◦', '○'],
                        change_rate: Some(0.5),
                        last_change: 0.0,
                    })
                    .color_curve(ColorCurve::Linear {
                        values: vec![0x06DD06, 0x074D43],
                    })
                    .alpha_curve(AlphaCurve::EaseOut {
                        values: vec![0.7, 0.3],
                    })
                    .velocity_curve(VelocityCurve::Linear {
                        values: vec![Vec2::new(0.0, -1.0), Vec2::new(0.0, -0.5)],
                    })
                    .spawn_area(SpawnArea::Circle {
                        radius: 0.5,
                        distribution: Distribution::Uniform,
                    })
                    .priority(140)
                    .spawn_rate(10.0)
                    .lifetime_range(1.0..2.0),
            ),

            ConditionType::Burning { .. } => {
                Some(
                    ParticleSpawner::new(Vec2::ZERO)
                        .glyph_animation(GlyphAnimation::RandomPool {
                            glyphs: vec!['*', '✦', '●', '○'],
                            change_rate: Some(8.0),
                            last_change: 0.0,
                        })
                        .color_curve(ColorCurve::Linear {
                            values: vec![0xFF1100, 0xFFD000], // Orange to red flames
                        })
                        .bg_curve(ColorCurve::Linear {
                            values: vec![0xFF1100, 0xFFD000], // Orange to red flames
                        })
                        .alpha_curve(AlphaCurve::EaseOut {
                            values: vec![0.8, 0.3],
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
                        .lifetime_range(0.8..1.5),
                )
            }

            ConditionType::Bleeding { .. } => {
                Some(
                    ParticleSpawner::new(Vec2::ZERO)
                        .glyph_animation(GlyphAnimation::Sequence {
                            // glyphs: vec!['•', ',', '.'],
                            glyphs: vec!['•', '☻'],
                            timing: SequenceTiming::LifetimeOnce {
                                easing: SequenceEasing::EaseIn,
                            },
                        })
                        .color_curve(ColorCurve::Linear {
                            values: vec![0xC43131, 0x880000], // Bright to dark red
                        })
                        .alpha_curve(AlphaCurve::EaseOut {
                            values: vec![0.9, 0.25],
                        })
                        .velocity_curve(VelocityCurve::EaseOut {
                            values: vec![Vec2::new(0.0, 0.0), Vec2::new(0.0, 2.5)], // Drop down with gravity
                        })
                        .spawn_area(SpawnArea::Circle {
                            radius: 0.3,
                            distribution: Distribution::Gaussian,
                        })
                        // .gravity(Vec2::new(0.0, 1.5)) // Blood drops down
                        .priority(150) // Higher than other conditions
                        .spawn_rate(5.) // Slower drip rate
                        .lifetime_range(0.8..1.0),
                )
            }

            // Conditions that don't need particle effects (behavioral effects)
            ConditionType::Feared { .. }
            | ConditionType::Taunted { .. }
            | ConditionType::Confused { .. } => None,
        }
    }
}
