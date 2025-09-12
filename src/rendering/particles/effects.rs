use bevy_ecs::prelude::*;
use macroquad::math::Vec2;

use super::core::{GlyphAnimation, Particle, ParticleSpawner, ParticleTrail};
use super::curves::{AlphaCurve, ColorCurve, VelocityCurve};
use super::spawn_areas::{Distribution, SpawnArea};
use crate::domain::MaterialType;
use crate::rendering::world_to_zone_local_f32;

// Blood Effects - Direction/Angle Based
pub fn spawn_blood_spray(world: &mut World, position: Vec2, direction: Vec2, intensity: f32) {
    let spread_angle = 30.0 * intensity;
    let particle_count = (8.0 * intensity) as u32;

    ParticleSpawner::new(position)
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
            values: vec![
                direction.normalize() * (5.0 * intensity),
                direction.normalize() * (1.0 * intensity) + Vec2::new(0.0, 3.0),
            ],
        })
        .spawn_area(SpawnArea::Arc {
            radius: 0.5,
            angle_start: -spread_angle,
            angle_end: spread_angle,
            radial_distribution: Distribution::Uniform,
        })
        .gravity(Vec2::new(0.0, 1.0))
        .priority(150)
        .lifetime_range(0.5..1.0)
        .burst(particle_count)
        .spawn_world(world);
}

// Projectile Effects - Distance/Position Based
pub fn spawn_bullet_trail(
    world: &mut World,
    start_pos: Vec2,
    target_pos: Vec2,
    bullet_speed: f32,
    rand: &mut crate::common::Rand,
) {
    let direction = (target_pos - start_pos).normalize();
    let distance = start_pos.distance(target_pos);
    let travel_time = distance / bullet_speed;

    // Main bullet particle
    let mut bullet_particle = Particle {
        age: 0.0,
        max_age: travel_time + 0.1,
        pos: start_pos,
        initial_pos: start_pos,

        glyph_animation: GlyphAnimation::Static('*'),
        color_curve: Some(ColorCurve::Constant(0x3BD1FF)),
        bg_curve: None,
        alpha_curve: Some(AlphaCurve::Constant(1.0)),
        velocity_curve: Some(VelocityCurve::Constant(direction * bullet_speed)),
        gravity: Vec2::ZERO,

        current_velocity: direction * bullet_speed,
        current_glyph: '*',
        current_color: 0xFFFF00,
        current_bg_color: 0x000000,
        current_alpha: 1.0,

        priority: 200,
        target_pos: Some(target_pos),
        max_distance: Some(distance + 0.5),
    };

    // Initialize current values
    bullet_particle.update_properties(0.0, rand);

    let bullet_entity = world.spawn(bullet_particle).id();

    // Create smoke trail spawner
    let trail_spawner = ParticleSpawner::new(Vec2::ZERO) // Position gets overridden by trail system
        .glyph_animation(GlyphAnimation::Static(' '))
        .bg_curve(ColorCurve::EaseOut {
            values: vec![0xF3D9BB, 0x261766],
        })
        .alpha_curve(AlphaCurve::Linear {
            values: vec![0.4, 0.0],
        })
        .velocity_curve(VelocityCurve::Linear {
            values: vec![Vec2::new(0.0, -0.5), Vec2::new(0.0, 1.0)],
        })
        .priority(90)
        .lifetime_range(0.2..0.6)
        .burst(1);

    let trail = ParticleTrail::new(250.0, trail_spawner);
    world.entity_mut(bullet_entity).insert(trail);

    // Muzzle flash
    ParticleSpawner::new(start_pos)
        .glyph_animation(GlyphAnimation::RandomPool {
            glyphs: vec!['*', '◦', '○'],
            change_rate: Some(20.0),
            last_change: 0.0,
        })
        .color_curve(ColorCurve::EaseOut {
            values: vec![0xFFFF00, 0xFF4400],
        })
        .spawn_area(SpawnArea::Circle {
            radius: 0.8,
            distribution: Distribution::EdgeOnly,
        })
        .velocity_curve(VelocityCurve::EaseOut {
            values: vec![Vec2::new(3.0, -1.0), Vec2::new(0.5, 1.0)],
        })
        .gravity(Vec2::new(0.0, 2.0))
        .priority(180)
        .lifetime_range(0.1..0.3)
        .burst(4)
        .spawn_world(world);

    // Delayed blood spray at impact
    spawn_delayed_blood_impact(world, target_pos, direction, travel_time);
}

fn spawn_delayed_blood_impact(
    world: &mut World,
    impact_pos: Vec2,
    bullet_direction: Vec2,
    delay: f32,
) {
    ParticleSpawner::new(impact_pos)
        .glyph_animation(GlyphAnimation::RandomPool {
            glyphs: vec!['*', '•', '·', '○'],
            change_rate: Some(5.0),
            last_change: 0.0,
        })
        .color_curve(ColorCurve::Constant(0x440000))
        .bg_curve(ColorCurve::EaseOut {
            values: vec![0xCC0000, 0x440000],
        })
        .spawn_area(SpawnArea::Arc {
            radius: 1.0,
            angle_start: -60.0,
            angle_end: 60.0,
            radial_distribution: Distribution::Gaussian,
        })
        .velocity_curve(VelocityCurve::EaseOut {
            values: vec![
                bullet_direction * 6.0,
                bullet_direction * 1.0 + Vec2::new(0.0, 4.0),
            ],
        })
        .gravity(Vec2::new(0.0, 3.0))
        .priority(150)
        .lifetime_range(0.5..1.2)
        .delay(delay)
        .burst(8)
        .spawn_world(world);
}

// Environmental Effects - Area/Context Based
pub fn spawn_material_hit(
    world: &mut World,
    position: Vec2,
    material: MaterialType,
    direction: Vec2,
) {
    let (spark_color, spark_count, spark_glyphs, base_speed, spread_angle) = match material {
        MaterialType::Stone => (0xDEE4E4, 5, vec!['♥', '♠', '♦', '♦'], 4.0, 45.0),
        MaterialType::Wood => (0xB14D13, 4, vec![',', '"', '.', '`'], 3.0, 35.0),
        MaterialType::Flesh => (0xFF4444, 3, vec!['*', '•', '○'], 3.5, 40.0),
    };

    ParticleSpawner::new(position)
        .glyph_animation(GlyphAnimation::RandomPool {
            glyphs: spark_glyphs,
            change_rate: Some(15.0),
            last_change: 0.0,
        })
        .color_curve(ColorCurve::EaseOut {
            values: vec![spark_color, spark_color & 0x585858],
        })
        .velocity_curve(VelocityCurve::EaseOut {
            values: vec![
                direction.normalize() * base_speed,
                direction.normalize() * (base_speed * 0.3) + Vec2::new(0.0, 3.0),
            ],
        })
        .spawn_area(SpawnArea::Arc {
            radius: 0.5,
            angle_start: -spread_angle,
            angle_end: spread_angle,
            radial_distribution: Distribution::Uniform,
        })
        .gravity(Vec2::new(0.0, 3.0))
        .priority(180)
        .lifetime_range(0.4..0.7)
        .burst(spark_count)
        .spawn_world(world);
}

// Helper functions for converting world coordinates
pub fn spawn_bullet_trail_in_world(
    world: &mut World,
    start_world: (usize, usize, usize),
    target_world: (usize, usize, usize),
    speed: f32,
    rand: &mut crate::common::Rand,
) {
    let start_local =
        world_to_zone_local_f32(start_world.0 as f32 + 0.5, start_world.1 as f32 + 0.5);
    let start_pos = Vec2::new(start_local.0, start_local.1);

    let target_local =
        world_to_zone_local_f32(target_world.0 as f32 + 0.5, target_world.1 as f32 + 0.5);
    let target_pos = Vec2::new(target_local.0, target_local.1);

    spawn_bullet_trail(world, start_pos, target_pos, speed, rand);
}

pub fn spawn_material_hit_in_world(
    world: &mut World,
    world_pos: (usize, usize, usize),
    material: MaterialType,
    direction: Vec2,
) {
    let local_pos = world_to_zone_local_f32(world_pos.0 as f32 + 0.5, world_pos.1 as f32 + 0.5);
    let pos = Vec2::new(local_pos.0, local_pos.1);

    spawn_material_hit(world, pos, material, direction);
}

pub fn spawn_directional_blood_mist(
    world: &mut World,
    world_pos: (usize, usize, usize),
    direction: Vec2,
    intensity: f32,
) {
    let local_pos = world_to_zone_local_f32(world_pos.0 as f32 + 0.5, world_pos.1 as f32 + 0.5);
    let pos = Vec2::new(local_pos.0, local_pos.1);

    spawn_blood_spray(world, pos, direction, intensity);
}
