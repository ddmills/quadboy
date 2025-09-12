use bevy_ecs::prelude::*;
use macroquad::math::Vec2;

use crate::{
    common::cp437_idx,
    domain::PlayerPosition,
    engine::Time,
    rendering::{Glyph, Position, Visibility, zone_local_to_world_f32},
};

use super::core::{
    Fragment, Particle, ParticleGlyph, ParticleGlyphPool, ParticleGrid, ParticleSpawner,
    ParticleTrail,
};

pub fn update_particles(mut particle_grid: ResMut<ParticleGrid>, q_particles: Query<&Particle>) {
    particle_grid.clear();

    for particle in q_particles.iter() {
        let grid_x = (particle.pos.x * 2.0) as usize;
        let grid_y = (particle.pos.y * 2.0) as usize;

        let alpha = particle.current_alpha();
        let glyph_idx = cp437_idx(particle.current_glyph()).unwrap_or(0);

        let fragment = Fragment {
            priority: particle.priority,
            glyph_idx,
            fg1: particle.current_fg1(),
            bg: particle.current_bg(),
            alpha,
        };

        particle_grid.push(grid_x, grid_y, fragment);
    }
}

pub fn render_particle_fragments(
    mut cmds: Commands,
    grid: Res<ParticleGrid>,
    mut batch: ResMut<ParticleGlyphPool>,
    mut q_glyph: Query<(&mut Glyph, &mut Position), With<ParticleGlyph>>,
    player_position: Option<Res<PlayerPosition>>,
) {
    let player_z = player_position.as_ref().map(|p| p.z).unwrap_or(0.0);
    let zone_idx = player_position.as_ref().map(|p| p.zone_idx()).unwrap_or(0);

    for x in 0..grid.data.width() {
        for y in 0..grid.data.height() {
            let Some(fragment) = grid.get(x, y) else {
                continue;
            };

            let Some(free_glyph) = batch.free_glyphs.pop() else {
                return;
            };

            batch.used_glyphs.push(free_glyph);

            let Ok((mut glyph, mut position)) = q_glyph.get_mut(free_glyph) else {
                continue;
            };

            glyph.idx = fragment.glyph_idx;
            glyph.fg1 = fragment.fg1;
            glyph.fg2 = fragment.fg1;
            glyph.bg = fragment.bg;
            glyph.alpha = fragment.alpha;

            let world_pos = zone_local_to_world_f32(zone_idx, x as f32 / 2., y as f32 / 2.);

            position.x = world_pos.0;
            position.y = world_pos.1;
            position.z = player_z;

            cmds.entity(free_glyph).insert(Visibility::Visible);
        }
    }
}

pub fn cleanup_particle_glyphs(mut cmds: Commands, mut particle_pool: ResMut<ParticleGlyphPool>) {
    let mut used = particle_pool.used_glyphs.clone();
    particle_pool.used_glyphs = vec![];

    for entity in used.iter() {
        cmds.entity(*entity).insert(Visibility::Hidden);
    }

    particle_pool.free_glyphs.append(&mut used);
}

pub fn update_particle_physics(
    mut cmds: Commands,
    mut q_particles: Query<(Entity, &mut Particle)>,
    time: Res<Time>,
    mut rand: ResMut<crate::common::Rand>,
) {
    let dt = time.dt;

    for (entity, mut particle) in q_particles.iter_mut() {
        particle.age += dt;

        if particle.age >= particle.max_age {
            cmds.entity(entity).despawn();
            continue;
        }

        // Update all curve-based properties
        particle.update_properties(dt, &mut rand);

        // Check if particle has reached its target
        if let Some(target_pos) = particle.target_pos {
            let distance_to_target = particle.pos.distance(target_pos);
            if distance_to_target <= 0.5 {
                cmds.entity(entity).despawn();
                continue;
            }
        }

        // Check if particle has traveled its maximum distance
        if let Some(max_distance) = particle.max_distance {
            let distance_traveled = particle.pos.distance(particle.initial_pos);
            if distance_traveled >= max_distance {
                cmds.entity(entity).despawn();
                continue;
            }
        }

        // Apply gravity and velocity
        let gravity = particle.gravity;
        particle.current_velocity += gravity * dt;
        let velocity = particle.current_velocity;
        particle.pos += velocity * dt;
    }
}

pub fn update_particle_spawners(
    mut cmds: Commands,
    mut q_spawners: Query<(Entity, &mut ParticleSpawner)>,
    time: Res<Time>,
    mut rand: ResMut<crate::common::Rand>,
) {
    let dt = time.dt;

    for (entity, mut spawner) in q_spawners.iter_mut() {
        spawner.timer += dt;

        // Check if we haven't reached the spawn delay yet
        if spawner.timer < spawner.spawn_delay {
            continue;
        }

        if let Some(burst_count) = spawner.burst_count {
            for _ in 0..burst_count {
                spawn_particle(&mut cmds, &spawner, &mut rand);
            }
            cmds.entity(entity).despawn();
            continue;
        }

        let spawn_interval = 1.0 / spawner.spawn_rate;
        let elapsed_since_delay = spawner.timer - spawner.spawn_delay;
        if elapsed_since_delay >= spawn_interval {
            spawn_particle(&mut cmds, &spawner, &mut rand);
            spawner.timer = spawner.spawn_delay + (elapsed_since_delay - spawn_interval);
        }
    }
}

pub fn spawn_particle(
    cmds: &mut Commands,
    spawner: &ParticleSpawner,
    rand: &mut crate::common::Rand,
) {
    use super::curves::CurveEvaluator;

    let lifetime =
        spawner.lifetime_min + rand.random() * (spawner.lifetime_max - spawner.lifetime_min);
    let spawn_position = spawner.spawn_area.generate_position(spawner.position, rand);

    // Initialize current values from curves
    let initial_velocity = spawner
        .velocity_curve
        .as_ref()
        .map(|curve| curve.evaluate(0.0))
        .unwrap_or(Vec2::new(0.0, -1.0));

    let initial_color = spawner
        .color_curve
        .as_ref()
        .map(|curve| curve.evaluate(0.0))
        .unwrap_or(0xFFFFFF);

    let initial_bg_color = spawner
        .bg_curve
        .as_ref()
        .map(|curve| curve.evaluate(0.0))
        .unwrap_or(0x000000);

    let initial_alpha = spawner
        .alpha_curve
        .as_ref()
        .map(|curve| curve.evaluate(0.0))
        .unwrap_or(1.0);

    let initial_glyph = match &spawner.glyph_animation {
        super::core::GlyphAnimation::Static(glyph) => *glyph,
        super::core::GlyphAnimation::RandomPool { glyphs, .. } => {
            if glyphs.is_empty() {
                '*'
            } else {
                glyphs[rand.pick_idx(glyphs)]
            }
        }
        super::core::GlyphAnimation::Sequence { glyphs, .. } => {
            if glyphs.is_empty() {
                '*'
            } else {
                glyphs[0]
            }
        }
        super::core::GlyphAnimation::TimedCurve { keyframes } => {
            if keyframes.is_empty() {
                '*'
            } else {
                keyframes[0].1
            }
        }
    };

    cmds.spawn(Particle {
        age: 0.0,
        max_age: lifetime,
        pos: spawn_position,
        initial_pos: spawn_position,

        // Animation curves
        glyph_animation: spawner.glyph_animation.clone(),
        color_curve: spawner.color_curve.clone(),
        bg_curve: spawner.bg_curve.clone(),
        alpha_curve: spawner.alpha_curve.clone(),
        velocity_curve: spawner.velocity_curve.clone(),
        gravity: spawner.gravity,

        // Current state (initialized)
        current_velocity: initial_velocity,
        current_glyph: initial_glyph,
        current_color: initial_color,
        current_bg_color: initial_bg_color,
        current_alpha: initial_alpha,

        priority: spawner.priority,
        target_pos: None,
        max_distance: None,
    });
}

pub fn update_particle_trails(
    mut cmds: Commands,
    mut q_particles_with_trails: Query<(&mut ParticleTrail, &Particle)>,
    time: Res<Time>,
    mut rand: ResMut<crate::common::Rand>,
) {
    let dt = time.dt;

    for (mut trail, particle) in q_particles_with_trails.iter_mut() {
        trail.last_spawn_time += dt;

        let spawn_interval = 1.0 / trail.spawn_rate;
        if trail.last_spawn_time >= spawn_interval {
            trail.trail_spawner.position = particle.pos;
            spawn_particle(&mut cmds, &trail.trail_spawner, &mut rand);
            trail.last_spawn_time = 0.0; // Reset timer
        }
    }
}
