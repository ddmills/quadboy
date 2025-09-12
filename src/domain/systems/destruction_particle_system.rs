use bevy_ecs::prelude::*;

use crate::{
    domain::MaterialType,
    rendering::{
        AlphaCurve, ColorCurve, Distribution, GlyphAnimation, ParticleSpawner, SpawnArea,
        VelocityCurve, world_to_zone_local_f32,
    },
};
use macroquad::math::Vec2;

use super::destruction_system::EntityDestroyedEvent;

// Implementation using new effect-specific functions
pub fn on_entity_destroyed_particles_deferred(
    mut e_destroyed: EventReader<EntityDestroyedEvent>,
    mut cmds: Commands,
) {
    for event in e_destroyed.read() {
        // Use material type from event instead of querying component
        if let Some(material_type) = event.material_type {
            match material_type {
                MaterialType::Stone => {
                    // Convert world coordinates to zone-local coordinates
                    let local_pos = world_to_zone_local_f32(
                        event.position.0 as f32 + 0.5,
                        event.position.1 as f32 + 0.5,
                    );
                    let pos = Vec2::new(local_pos.0, local_pos.1);

                    // Spawn stone debris particles using spawner

                    cmds.spawn(
                        ParticleSpawner::new(pos)
                            .glyph_animation(GlyphAnimation::Static('.'))
                            .color_curve(ColorCurve::Linear {
                                values: vec![0xB1B1B1, 0x404040],
                            })
                            .spawn_area(SpawnArea::Circle {
                                radius: 1.0,
                                distribution: Distribution::Uniform,
                            })
                            .velocity_curve(VelocityCurve::EaseOut {
                                values: vec![Vec2::new(3.0, -2.0), Vec2::new(0.0, 5.0)],
                            })
                            .gravity(Vec2::new(0.0, 3.0))
                            .priority(140)
                            .lifetime_range(1.0..2.0)
                            .burst(6),
                    );
                }
                MaterialType::Wood => {
                    // Wood splinter effect
                    let local_pos = world_to_zone_local_f32(
                        event.position.0 as f32 + 0.5,
                        event.position.1 as f32 + 0.5,
                    );
                    let pos = Vec2::new(local_pos.0, local_pos.1);

                    cmds.spawn(
                        ParticleSpawner::new(pos)
                            .glyph_animation(GlyphAnimation::Static(','))
                            .color_curve(ColorCurve::Linear {
                                values: vec![0xFF6600, 0x664400],
                            })
                            .spawn_area(SpawnArea::Circle {
                                radius: 0.5,
                                distribution: Distribution::Uniform,
                            })
                            .velocity_curve(VelocityCurve::EaseOut {
                                values: vec![Vec2::new(2.0, -1.0), Vec2::new(0.0, 3.0)],
                            })
                            .gravity(Vec2::new(0.0, 2.0))
                            .priority(130)
                            .lifetime_range(0.5..1.5)
                            .burst(4),
                    );
                }
                MaterialType::Flesh => {
                    // Blood splatter effect
                    let local_pos = world_to_zone_local_f32(
                        event.position.0 as f32 + 0.5,
                        event.position.1 as f32 + 0.5,
                    );
                    let pos = Vec2::new(local_pos.0, local_pos.1);

                    cmds.spawn(
                        ParticleSpawner::new(pos)
                            .glyph_animation(GlyphAnimation::RandomPool {
                                glyphs: vec!['*', '•', '·', '○'],
                                change_rate: Some(8.0),
                                last_change: 0.0,
                            })
                            .color_curve(ColorCurve::EaseOut {
                                values: vec![0xCC0000, 0x440000],
                            })
                            .alpha_curve(AlphaCurve::EaseOut {
                                values: vec![0.8, 0.0],
                            })
                            .velocity_curve(VelocityCurve::EaseOut {
                                values: vec![Vec2::new(3.0, -1.0), Vec2::new(1.0, 2.0)],
                            })
                            .spawn_area(SpawnArea::Circle {
                                radius: 0.8,
                                distribution: Distribution::Uniform,
                            })
                            .gravity(Vec2::new(0.0, 1.0))
                            .priority(150)
                            .lifetime_range(0.5..1.0)
                            .burst(5),
                    );
                }
            }
        }
    }
}
