use bevy_ecs::prelude::*;
use macroquad::math::Vec2;

use super::core::{GlyphAnimation, Particle, ParticleSpawner, ParticleTrail};
use super::curves::{AlphaCurve, ColorCurve, VelocityCurve};
use super::spawn_areas::{Distribution, SpawnArea};
use crate::common::Rand;
use crate::domain::{MaterialType, PlayerPosition};
use crate::rendering::{world_to_zone_idx, world_to_zone_local_f32};

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
            base_direction: Some(direction),
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
    rand: &mut Rand,
    spawn_blood: bool,
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
            // values: vec![0xF3D9BB, 0x261766],
            values: vec![0x666666, 0x333333, 0x000000],
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
        .glyph_animation(GlyphAnimation::Sequence {
            glyphs: vec!['◦', '○', '*'],
            duration_per_glyph: 0.1,
        })
        .color_curve(ColorCurve::Constant(0xFF4400))
        .bg_curve(ColorCurve::Linear {
            values: vec![0xFFFF00, 0xFF4400, 0xE6E6E6],
        })
        .spawn_area(SpawnArea::Arc {
            radius: 1.5,
            angle_start: -25.0,
            angle_end: 25.0,
            radial_distribution: Distribution::Gaussian,
            base_direction: Some(direction),
        })
        .velocity_curve(VelocityCurve::EaseOut {
            values: vec![direction * 5.0, direction * 2.0],
        })
        // .gravity(Vec2::new(0.0, 2.0))
        .priority(180)
        .lifetime_range(0.2..0.3)
        .burst(40)
        .spawn_world(world);

    // Delayed blood spray at impact (only if hit)
    if spawn_blood {
        spawn_delayed_blood_impact(world, target_pos, direction, travel_time);
    }
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
            base_direction: Some(bullet_direction),
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
            base_direction: Some(direction),
        })
        .gravity(Vec2::new(0.0, 3.0))
        .priority(180)
        .lifetime_range(0.4..0.7)
        .burst(spark_count)
        .spawn_world(world);
}

// Helper functions for converting world coordinates
pub fn spawn_throw_trail(
    world: &mut World,
    start_pos: Vec2,
    target_pos: Vec2,
    throw_speed: f32,
    item_glyph: char,
    item_color: u32,
    rand: &mut Rand,
) {
    let direction = (target_pos - start_pos).normalize();
    let distance = start_pos.distance(target_pos);
    let travel_time = distance / throw_speed;

    // Main thrown item particle
    let mut item_particle = Particle {
        age: 0.0,
        max_age: travel_time + 0.1,
        pos: start_pos,
        initial_pos: start_pos,

        glyph_animation: GlyphAnimation::Static(item_glyph),
        color_curve: Some(ColorCurve::Constant(item_color)),
        bg_curve: Some(ColorCurve::EaseOut {
            values: vec![0x444444, 0x222222, 0x000000],
        }),
        alpha_curve: Some(AlphaCurve::Constant(1.0)),
        velocity_curve: Some(VelocityCurve::Constant(direction * throw_speed)),
        gravity: Vec2::ZERO,

        current_velocity: direction * throw_speed,
        current_glyph: item_glyph,
        current_color: item_color,
        current_bg_color: 0x444444,
        current_alpha: 1.0,

        priority: 150,
        target_pos: Some(target_pos),
        max_distance: Some(distance + 0.5),
    };

    // Initialize current values
    item_particle.update_properties(0.0, rand);

    let item_entity = world.spawn(item_particle).id();

    // Create faint trail spawner
    let trail_spawner = ParticleSpawner::new(Vec2::ZERO)
        .glyph_animation(GlyphAnimation::Static(' '))
        .bg_curve(ColorCurve::EaseOut {
            values: vec![0x666666, 0x333333, 0x000000],
        })
        .alpha_curve(AlphaCurve::Linear {
            values: vec![0.3, 0.0],
        })
        .velocity_curve(VelocityCurve::Linear {
            values: vec![Vec2::new(0.0, 0.0), Vec2::new(0.0, 0.5)],
        })
        .priority(80)
        .lifetime_range(0.3..0.8)
        .burst(1);

    let trail = ParticleTrail::new(150.0, trail_spawner);
    world.entity_mut(item_entity).insert(trail);
}

pub fn spawn_throw_trail_in_world(
    world: &mut World,
    start_world: (usize, usize, usize),
    target_world: (usize, usize, usize),
    throw_speed: f32,
    item_glyph: char,
    item_color: u32,
    rand: &mut Rand,
) {
    // Check if either start or target position is in active zone
    if let Some(player_pos) = world.get_resource::<PlayerPosition>() {
        let start_zone_idx = world_to_zone_idx(start_world.0, start_world.1, start_world.2);
        let target_zone_idx = world_to_zone_idx(target_world.0, target_world.1, target_world.2);
        let active_zone = player_pos.zone_idx();

        if start_zone_idx != active_zone && target_zone_idx != active_zone {
            return; // Don't spawn particles if neither position is in active zone
        }
    }

    let start_pos = Vec2::new(
        world_to_zone_local_f32(start_world.0 as f32, start_world.1 as f32).0,
        world_to_zone_local_f32(start_world.0 as f32, start_world.1 as f32).1,
    );
    let target_pos = Vec2::new(
        world_to_zone_local_f32(target_world.0 as f32, target_world.1 as f32).0,
        world_to_zone_local_f32(target_world.0 as f32, target_world.1 as f32).1,
    );

    spawn_throw_trail(
        world,
        start_pos,
        target_pos,
        throw_speed,
        item_glyph,
        item_color,
        rand,
    );
}

pub fn spawn_bullet_trail_in_world(
    world: &mut World,
    start_world: (usize, usize, usize),
    target_world: (usize, usize, usize),
    speed: f32,
    rand: &mut Rand,
    spawn_blood: bool,
) {
    // Check if either start or target position is in active zone
    if let Some(player_pos) = world.get_resource::<PlayerPosition>() {
        let start_zone_idx = world_to_zone_idx(start_world.0, start_world.1, start_world.2);
        let target_zone_idx = world_to_zone_idx(target_world.0, target_world.1, target_world.2);
        let active_zone = player_pos.zone_idx();

        if start_zone_idx != active_zone && target_zone_idx != active_zone {
            return; // Don't spawn particles if neither position is in active zone
        }
    }

    let start_local =
        world_to_zone_local_f32(start_world.0 as f32 + 0.5, start_world.1 as f32 + 0.5);
    let start_pos = Vec2::new(start_local.0, start_local.1);

    let target_local =
        world_to_zone_local_f32(target_world.0 as f32 + 0.5, target_world.1 as f32 + 0.5);
    let target_pos = Vec2::new(target_local.0, target_local.1);

    spawn_bullet_trail(world, start_pos, target_pos, speed, rand, spawn_blood);
}

pub fn spawn_material_hit_in_world(
    world: &mut World,
    world_pos: (usize, usize, usize),
    material: MaterialType,
    direction: Vec2,
) {
    // Check if position is in active zone
    if let Some(player_pos) = world.get_resource::<PlayerPosition>() {
        let zone_idx = world_to_zone_idx(world_pos.0, world_pos.1, world_pos.2);
        if zone_idx != player_pos.zone_idx() {
            return; // Don't spawn particles outside active zone
        }
    }

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
    // Check if position is in active zone
    if let Some(player_pos) = world.get_resource::<PlayerPosition>() {
        let zone_idx = world_to_zone_idx(world_pos.0, world_pos.1, world_pos.2);
        if zone_idx != player_pos.zone_idx() {
            return; // Don't spawn particles outside active zone
        }
    }

    let local_pos = world_to_zone_local_f32(world_pos.0 as f32 + 0.5, world_pos.1 as f32 + 0.5);
    let pos = Vec2::new(local_pos.0, local_pos.1);

    spawn_blood_spray(world, pos, direction, intensity);
}

/// Explosion effect for top-down view - radial debris and smoke
pub fn spawn_explosion_effect(world: &mut World, world_pos: (usize, usize, usize), radius: usize) {
    // Check if position is in active zone
    if let Some(player_pos) = world.get_resource::<PlayerPosition>() {
        let zone_idx = world_to_zone_idx(world_pos.0, world_pos.1, world_pos.2);
        if zone_idx != player_pos.zone_idx() {
            return; // Don't spawn particles outside active zone
        }
    }

    let local_pos = world_to_zone_local_f32(world_pos.0 as f32 + 0.5, world_pos.1 as f32 + 0.5);
    let pos = Vec2::new(local_pos.0, local_pos.1);

    let radius_f = radius as f32;
    let scale = (radius_f / 3.0).max(0.5); // Scale effects based on radius (dynamite = radius 3)

    // 1. CENTRAL FLASH - Bright expanding core with intense background glow
    ParticleSpawner::new(pos)
        .glyph_animation(GlyphAnimation::Sequence {
            glyphs: vec!['◉', '◎', '○', '◦', ' '],
            duration_per_glyph: 0.1,
        })
        .color_curve(ColorCurve::Linear {
            values: vec![0xFFFFFF, 0xFFDD00, 0xFF8800],
        })
        .bg_curve(ColorCurve::Linear {
            values: vec![0xFFDD00, 0xFF8800, 0xFF4400, 0x880000],
        })
        .alpha_curve(AlphaCurve::EaseOut {
            values: vec![1.0, 0.0],
        })
        .priority(220)
        .lifetime_range(0.3..0.5)
        .burst(1)
        .spawn_world(world);

    // 2. SHOCKWAVE RING - Expanding blast wave
    let shockwave_count = (8.0 * scale) as u32;
    ParticleSpawner::new(pos)
        .glyph_animation(GlyphAnimation::Sequence {
            glyphs: vec!['█', '▓', '▒', '░', ' '],
            duration_per_glyph: 0.08,
        })
        .color_curve(ColorCurve::Constant(0x888888))
        .bg_curve(ColorCurve::EaseOut {
            values: vec![0xFFFFFF, 0xFFDD00, 0xFF8800, 0x000000],
        })
        .alpha_curve(AlphaCurve::EaseOut {
            values: vec![0.9, 0.0],
        })
        .velocity_curve(VelocityCurve::EaseOut {
            values: vec![Vec2::ZERO, Vec2::ZERO], // Will be overridden by spawn area
        })
        .spawn_area(SpawnArea::Circle {
            radius: radius_f * 0.8,
            distribution: Distribution::EdgeOnly,
        })
        .priority(210)
        .lifetime_range(0.4..0.6)
        .burst(shockwave_count)
        .spawn_world(world);

    // 3. DEBRIS/SHRAPNEL - Fast fragments flying outward
    let debris_count = (50.0 * scale) as u32;
    ParticleSpawner::new(pos)
        .glyph_animation(GlyphAnimation::RandomPool {
            glyphs: vec!['*', '·', ',', '`', '\'', '.'],
            change_rate: Some(20.0),
            last_change: 0.0,
        })
        .color_curve(ColorCurve::EaseOut {
            values: vec![0xFFFFFF, 0xFFDD00, 0xFF8800, 0x888888],
        })
        .bg_curve(ColorCurve::EaseOut {
            values: vec![0xFF8800, 0xFF4400, 0x880000, 0x000000],
        })
        .alpha_curve(AlphaCurve::EaseOut {
            values: vec![1.0, 0.0],
        })
        .velocity_curve(VelocityCurve::EaseOut {
            values: vec![
                Vec2::ZERO, // Will be set by spawn area to radial outward
                Vec2::ZERO, // Slows down over time
            ],
        })
        .spawn_area(SpawnArea::Circle {
            radius: radius_f * 1.2,
            distribution: Distribution::Uniform,
        })
        .gravity(Vec2::ZERO) // No gravity in top-down view
        .priority(180)
        .lifetime_range(0.8..2.0)
        .burst(debris_count)
        .spawn_world(world);

    // 4. FIRE/SPARKS - Hot particles spreading outward
    let fire_count = (35.0 * scale) as u32;
    ParticleSpawner::new(pos)
        .glyph_animation(GlyphAnimation::RandomPool {
            glyphs: vec!['*', '✦', '●', '○', '•'],
            change_rate: Some(15.0),
            last_change: 0.0,
        })
        .color_curve(ColorCurve::EaseOut {
            values: vec![0xFFDD00, 0xFF8800, 0xFF4400, 0x880000],
        })
        .bg_curve(ColorCurve::EaseOut {
            values: vec![0xFF8800, 0x880000, 0x440000, 0x000000],
        })
        .alpha_curve(AlphaCurve::EaseOut {
            values: vec![0.9, 0.0],
        })
        .velocity_curve(VelocityCurve::EaseOut {
            values: vec![Vec2::ZERO, Vec2::ZERO], // Radial from spawn area
        })
        .spawn_area(SpawnArea::Circle {
            radius: radius_f * 0.8,
            distribution: Distribution::Gaussian,
        })
        .gravity(Vec2::ZERO)
        .priority(200)
        .lifetime_range(0.5..1.5)
        .burst(fire_count)
        .spawn_world(world);

    // 5. SMOKE PUFFS - Slower expanding smoke using background colors
    let smoke_count = (25.0 * scale) as u32;
    ParticleSpawner::new(pos)
        .glyph_animation(GlyphAnimation::RandomPool {
            glyphs: vec![' ', '░', '▒'], // Space character for pure background smoke
            change_rate: Some(8.0),
            last_change: 0.0,
        })
        .color_curve(ColorCurve::Constant(0x666666))
        .bg_curve(ColorCurve::EaseOut {
            values: vec![0x888888, 0x666666, 0x444444, 0x000000],
        })
        .alpha_curve(AlphaCurve::EaseOut {
            values: vec![0.7, 0.0],
        })
        .velocity_curve(VelocityCurve::EaseOut {
            values: vec![Vec2::ZERO, Vec2::ZERO], // Very slow expansion
        })
        .spawn_area(SpawnArea::Circle {
            radius: radius_f * 0.6,
            distribution: Distribution::Gaussian,
        })
        .gravity(Vec2::ZERO)
        .priority(100)
        .lifetime_range(2.0..4.0)
        .burst(smoke_count)
        .spawn_world(world);

    // 6. GROUND SCORCH MARKS - Lingering burn marks
    let scorch_count = (15.0 * scale) as u32;
    ParticleSpawner::new(pos)
        .glyph_animation(GlyphAnimation::RandomPool {
            glyphs: vec!['#', '%', '&', '@', '▓'],
            change_rate: Some(2.0),
            last_change: 0.0,
        })
        .color_curve(ColorCurve::EaseOut {
            values: vec![0xFF4400, 0x884400, 0x444444, 0x222222],
        })
        .bg_curve(ColorCurve::EaseOut {
            values: vec![0x880000, 0x442200, 0x221100, 0x000000],
        })
        .alpha_curve(AlphaCurve::EaseOut {
            values: vec![0.8, 0.0],
        })
        .velocity_curve(VelocityCurve::Constant(Vec2::ZERO)) // Stationary scorch marks
        .spawn_area(SpawnArea::Circle {
            radius: radius_f * 1.0,
            distribution: Distribution::Uniform,
        })
        .gravity(Vec2::ZERO)
        .priority(80)
        .lifetime_range(3.0..5.0)
        .burst(scorch_count)
        .spawn_world(world);
}

/// Level up celebration effect - golden sparkles and light burst
pub fn spawn_level_up_celebration(world: &mut World, world_pos: (usize, usize, usize)) {
    // Check if position is in active zone
    if let Some(player_pos) = world.get_resource::<PlayerPosition>() {
        let zone_idx = world_to_zone_idx(world_pos.0, world_pos.1, world_pos.2);
        if zone_idx != player_pos.zone_idx() {
            return; // Don't spawn particles outside active zone
        }
    }

    let local_pos = world_to_zone_local_f32(world_pos.0 as f32 + 0.5, world_pos.1 as f32 + 0.5);
    let pos = Vec2::new(local_pos.0, local_pos.1);

    // Golden sparkles radiating outward
    ParticleSpawner::new(pos)
        .glyph_animation(GlyphAnimation::RandomPool {
            glyphs: vec!['*', '✦', '✧', '◆', '◇', '○', '●'],
            change_rate: Some(12.0),
            last_change: 0.0,
        })
        .color_curve(ColorCurve::EaseOut {
            values: vec![0xFFD700, 0xFFA500, 0xFF6347], // Gold to orange to red
        })
        .alpha_curve(AlphaCurve::EaseOut {
            values: vec![1.0, 0.0],
        })
        .velocity_curve(VelocityCurve::EaseOut {
            values: vec![Vec2::new(0.0, -4.0), Vec2::new(0.0, 2.0)],
        })
        .spawn_area(SpawnArea::Circle {
            radius: 2.0,
            distribution: Distribution::Uniform,
        })
        .gravity(Vec2::new(0.0, 1.0))
        .priority(200)
        .lifetime_range(1.0..2.0)
        .burst(25)
        .spawn_world(world);

    // Bright central flash
    ParticleSpawner::new(pos)
        .glyph_animation(GlyphAnimation::Sequence {
            glyphs: vec!['◉', '◎', '○', '◦', ' '],
            duration_per_glyph: 0.15,
        })
        .color_curve(ColorCurve::Linear {
            values: vec![0xFFFFFF, 0xFFD700, 0xFFA500],
        })
        .bg_curve(ColorCurve::EaseOut {
            values: vec![0xFFD700, 0x000000],
        })
        .alpha_curve(AlphaCurve::EaseOut {
            values: vec![1.0, 0.0],
        })
        .priority(210)
        .lifetime_range(0.8..0.8)
        .burst(1)
        .spawn_world(world);

    // Rising golden motes
    ParticleSpawner::new(pos)
        .glyph_animation(GlyphAnimation::RandomPool {
            glyphs: vec!['•', '·', '⋅'],
            change_rate: Some(8.0),
            last_change: 0.0,
        })
        .color_curve(ColorCurve::Constant(0xFFD700))
        .alpha_curve(AlphaCurve::Linear {
            values: vec![0.8, 0.0],
        })
        .velocity_curve(VelocityCurve::Constant(Vec2::new(0.0, -3.0)))
        .spawn_area(SpawnArea::Circle {
            radius: 1.0,
            distribution: Distribution::Gaussian,
        })
        .priority(180)
        .lifetime_range(1.5..2.5)
        .burst(15)
        .spawn_world(world);
}
