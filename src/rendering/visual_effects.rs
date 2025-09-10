use bevy_ecs::prelude::*;
use macroquad::prelude::trace;

use super::{Layer, Position};
use crate::{
    common::{algorithm::bresenham::bresenham_line, MacroquadColorable, Palette, Rand},
    domain::IgnoreLighting,
    engine::Time,
    rendering::{Glyph, Visibility},
};

#[derive(Component)]
pub struct Particle {
    pub particle_type: ParticleType,
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub base_color: Palette,
    pub fade_color: Palette,
    pub fade_curve: FadeCurve,
    pub alpha: f32, // Current alpha value (0.0 = transparent, 1.0 = opaque)
}

#[derive(Component)]
pub struct ParticleVelocity {
    pub path: Vec<(usize, usize, usize)>,
    pub speed: f32, // grid cells per second
    pub path_index: usize, // current position along path
}

#[derive(Component)]
pub struct SmokeTrail {
    pub bullet_entity: Entity,
    pub last_position: Option<(usize, usize, usize)>,
    pub bullet_path_length: usize,
}

#[derive(Component)]
pub struct SmokeSpawnTimer {
    pub delay: f32,
    pub position: (usize, usize, usize),
    pub path_progress: f32, // 0.0 = start (muzzle), 1.0 = end
}

#[derive(Clone)]
pub enum ParticleType {
    Bullet { glyph: usize },
    Smoke { glyphs: Vec<usize> },
    Explosion { glyphs: Vec<usize> },
    Sparks { glyphs: Vec<usize> },
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum VisualEffectId {
    PistolShot,
    RifleShot,
    ShotgunBlast,
    DynamiteExplosion,
    PoisonGas,
    Smoke,
    MuzzleFlash,
    BloodSplatter,
    Sparks,
}

#[derive(Clone)]
pub enum FadeCurve {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
}

impl FadeCurve {
    pub fn apply(&self, progress: f32) -> f32 {
        match self {
            FadeCurve::Linear => progress,
            FadeCurve::EaseIn => progress * progress,
            FadeCurve::EaseOut => 1.0 - (1.0 - progress) * (1.0 - progress),
            FadeCurve::EaseInOut => {
                if progress < 0.5 {
                    2.0 * progress * progress
                } else {
                    1.0 - 2.0 * (1.0 - progress) * (1.0 - progress)
                }
            }
        }
    }
}

impl ParticleType {
    pub fn get_glyph(&self, rand: &mut Rand) -> usize {
        match self {
            ParticleType::Bullet { glyph } => *glyph,
            ParticleType::Smoke { glyphs } | 
            ParticleType::Explosion { glyphs } | 
            ParticleType::Sparks { glyphs } => {
                if glyphs.is_empty() {
                    225 // fallback
                } else {
                    glyphs[rand.range_n(0, glyphs.len() as i32) as usize]
                }
            }
        }
    }
}

pub fn spawn_rifle_shot(
    commands: &mut Commands,
    origin: (usize, usize, usize),
    target: (usize, usize, usize),
) {
    // Calculate Bresenham path from origin to target (keep full path)
    let path_2d = bresenham_line((origin.0, origin.1), (target.0, target.1));
    let path_3d: Vec<(usize, usize, usize)> = path_2d.iter().map(|(x, y)| (*x, *y, origin.2)).collect();
    
    // Need at least 2 positions (origin + at least one step)
    if path_3d.len() < 2 {
        return; // No path to follow or adjacent shot
    }
    
    // Spawn bullet particle starting at second position (skip origin)
    let start_pos = path_3d[1];
    let path_length = path_3d.len();
    let bullet_entity = commands.spawn((
        Particle {
            particle_type: ParticleType::Bullet { glyph: 225 },
            lifetime: 0.0,
            max_lifetime: calculate_travel_time_path(&path_3d, 24.0), // 24 grid cells per second (3x faster)
            base_color: Palette::Yellow,
            fade_color: Palette::Orange,
            fade_curve: FadeCurve::Linear,
            alpha: 1.0, // Bullets start fully opaque
        },
        ParticleVelocity {
            path: path_3d,
            speed: 32.0,
            path_index: 0, // Track from beginning, but bullet starts at path[1]
        },
        Position::new(start_pos.0, start_pos.1, start_pos.2),
        Glyph::idx(225).fg1(Palette::Yellow as u32).layer(Layer::Overlay),
        Visibility::Visible,
        IgnoreLighting,
    )).id();
    
    // Spawn smoke trail tracker
    commands.spawn(SmokeTrail {
        bullet_entity,
        last_position: None,
        bullet_path_length: path_length,
    });
}

pub fn spawn_pistol_shot(
    commands: &mut Commands,
    origin: (usize, usize, usize),
    target: (usize, usize, usize),
) {
    // Calculate Bresenham path from origin to target (keep full path)
    let path_2d = bresenham_line((origin.0, origin.1), (target.0, target.1));
    let path_3d: Vec<(usize, usize, usize)> = path_2d.iter().map(|(x, y)| (*x, *y, origin.2)).collect();
    
    // Need at least 2 positions (origin + at least one step)
    if path_3d.len() < 2 {
        return; // No path to follow or adjacent shot
    }
    
    // Similar to rifle but faster, different colors
    let start_pos = path_3d[1]; // Start at second position (skip origin)
    let path_length = path_3d.len();
    let bullet_entity = commands.spawn((
        Particle {
            particle_type: ParticleType::Bullet { glyph: 225 },
            lifetime: 0.0,
            max_lifetime: calculate_travel_time_path(&path_3d, 32.0), // 32 grid cells per second (4x faster)
            base_color: Palette::Yellow,
            fade_color: Palette::DarkYellow,
            fade_curve: FadeCurve::Linear,
            alpha: 1.0, // Bullets start fully opaque
        },
        ParticleVelocity {
            path: path_3d,
            speed: 32.0,
            path_index: 0, // Track from beginning, but bullet starts at path[1]
        },
        Position::new(start_pos.0, start_pos.1, start_pos.2),
        Glyph::idx(225).fg1(Palette::Yellow as u32).layer(Layer::Overlay),
        Visibility::Visible,
        IgnoreLighting,
    )).id();
    
    // Spawn smoke trail tracker
    commands.spawn(SmokeTrail {
        bullet_entity,
        last_position: None,
        bullet_path_length: path_length,
    });
}

pub fn spawn_shotgun_blast(
    commands: &mut Commands,
    origin: (usize, usize, usize),
    target: (usize, usize, usize),
) {
    let pellet_count = 7;
    let spread_angle: f32 = 35.0;
    
    // Calculate base direction
    let dx = target.0 as i32 - origin.0 as i32;
    let dy = target.1 as i32 - origin.1 as i32;
    let base_angle = (dy as f32).atan2(dx as f32);
    let range = ((dx * dx + dy * dy) as f32).sqrt() as usize;
    
    for i in 0..pellet_count {
        // Calculate spread for this pellet
        let spread_ratio = (i as f32 - (pellet_count - 1) as f32 / 2.0) / (pellet_count - 1) as f32;
        let angle_offset = spread_ratio * spread_angle.to_radians();
        let pellet_angle = base_angle + angle_offset;
        
        // Calculate pellet target
        let pellet_dx = (range as f32 * pellet_angle.cos()).round() as i32;
        let pellet_dy = (range as f32 * pellet_angle.sin()).round() as i32;
        let pellet_target = (
            (origin.0 as i32 + pellet_dx).max(0) as usize,
            (origin.1 as i32 + pellet_dy).max(0) as usize,
            origin.2,
        );
        
        // Calculate Bresenham path for this pellet (keep full path)
        let path_2d = bresenham_line((origin.0, origin.1), (pellet_target.0, pellet_target.1));
        let path_3d: Vec<(usize, usize, usize)> = path_2d.iter().map(|(x, y)| (*x, *y, origin.2)).collect();
        
        // Need at least 2 positions (origin + at least one step)
        if path_3d.len() < 2 {
            continue; // Skip this pellet if no valid path
        }
        
        let start_pos = path_3d[1]; // Start at second position (skip origin)
        let path_length = path_3d.len();
        let bullet_entity = commands.spawn((
            Particle {
                particle_type: ParticleType::Bullet { glyph: 225 },
                lifetime: 0.0,
                max_lifetime: calculate_travel_time_path(&path_3d, 30.0), // 30 grid cells per second (2x faster)
                base_color: Palette::Orange,
                fade_color: Palette::DarkOrange,
                fade_curve: FadeCurve::Linear,
                alpha: 1.0, // Bullets start fully opaque
            },
            ParticleVelocity {
                path: path_3d,
                speed: 30.0,
                path_index: 0, // Track from beginning, but bullet starts at path[1]
            },
            Position::new(start_pos.0, start_pos.1, start_pos.2),
            Glyph::idx(225).fg1(Palette::Orange as u32).layer(Layer::Overlay),
            Visibility::Visible,
            IgnoreLighting,
        )).id();
        
        // Each pellet gets its own smoke trail
        commands.spawn(SmokeTrail {
            bullet_entity,
            last_position: None,
            bullet_path_length: path_length,
        });
    }
}

fn calculate_travel_time_path(path: &[(usize, usize, usize)], speed: f32) -> f32 {
    if path.len() <= 1 {
        return 0.1; // minimum time
    }
    // Path length is number of steps - 1 (since we count transitions)
    let distance = (path.len() - 1) as f32;
    distance / speed
}

pub fn calculate_line_path(
    start: (usize, usize, usize),
    end: (usize, usize, usize),
) -> Vec<(usize, usize, usize)> {
    let mut path = Vec::new();

    let dx = end.0 as i32 - start.0 as i32;
    let dy = end.1 as i32 - start.1 as i32;

    let steps = dx.abs().max(dy.abs());

    if steps <= 1 {
        return vec![start, end];
    }

    let x_step = dx as f32 / steps as f32;
    let y_step = dy as f32 / steps as f32;

    // Include all positions from start to end
    for i in 0..=steps {
        let x = start.0 as f32 + x_step * i as f32;
        let y = start.1 as f32 + y_step * i as f32;
        path.push((x.round() as usize, y.round() as usize, start.2));
    }

    path
}

pub fn spawn_explosion(
    commands: &mut Commands,
    center: (usize, usize, usize),
    radius: usize,
    rand: &mut Rand,
) {
    let radius = radius as i32;
    
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            let distance = ((dx * dx + dy * dy) as f32).sqrt();
            if distance <= radius as f32 {
                let x = (center.0 as i32 + dx).max(0) as usize;
                let y = (center.1 as i32 + dy).max(0) as usize;
                let position = (x, y, center.2);
                
                let lifetime = 0.8 + (rand.random() - 0.5) * 0.4; // 0.6 to 1.0 seconds
                let glyph = rand.range_n(42, 47); // Various explosion glyphs
                
                commands.spawn((
                    Particle {
                        particle_type: ParticleType::Explosion { 
                            glyphs: vec![42, 43, 35, 46] // *, +, #, . 
                        },
                        lifetime: 0.0,
                        max_lifetime: lifetime,
                        base_color: Palette::Red,
                        fade_color: Palette::DarkRed,
                        fade_curve: FadeCurve::EaseOut,
                        alpha: 1.0, // Explosions start fully opaque
                    },
                    Position::new(position.0, position.1, position.2),
                    Glyph::idx(glyph as usize).fg1(Palette::Red as u32).layer(Layer::Overlay),
                    Visibility::Visible,
                    IgnoreLighting,
                ));
            }
        }
    }
}

pub fn spawn_visual_effect(
    commands: &mut Commands,
    effect_id: VisualEffectId,
    origin: (usize, usize, usize),
    target: Option<(usize, usize, usize)>,
    rand: &mut Rand,
) {
    match effect_id {
        VisualEffectId::PistolShot => {
            let target = target.expect("PistolShot requires target position");
            spawn_pistol_shot(commands, origin, target);
        }
        VisualEffectId::RifleShot => {
            let target = target.expect("RifleShot requires target position");
            spawn_rifle_shot(commands, origin, target);
        }
        VisualEffectId::ShotgunBlast => {
            let target = target.expect("ShotgunBlast requires target position");
            spawn_shotgun_blast(commands, origin, target);
        }
        VisualEffectId::DynamiteExplosion => {
            spawn_explosion(commands, origin, 3, rand);
        }
        _ => {
            // Placeholder for other effects - implement as needed
            let target = target.unwrap_or(origin);
            spawn_pistol_shot(commands, origin, target);
        }
    }
}

pub fn spawn_visual_effect_world(
    world: &mut World,
    effect_id: VisualEffectId,
    origin: (usize, usize, usize),
    target: Option<(usize, usize, usize)>,
) {
    match effect_id {
        VisualEffectId::PistolShot => {
            let target = target.expect("PistolShot requires target position");
            world.resource_scope(|world, _: Mut<Rand>| {
                spawn_pistol_shot(&mut world.commands(), origin, target);
            });
        }
        VisualEffectId::RifleShot => {
            let target = target.expect("RifleShot requires target position");
            world.resource_scope(|world, _: Mut<Rand>| {
                spawn_rifle_shot(&mut world.commands(), origin, target);
            });
        }
        VisualEffectId::ShotgunBlast => {
            let target = target.expect("ShotgunBlast requires target position");
            world.resource_scope(|world, _: Mut<Rand>| {
                spawn_shotgun_blast(&mut world.commands(), origin, target);
            });
        }
        VisualEffectId::DynamiteExplosion => {
            world.resource_scope(|world, mut rand: Mut<Rand>| {
                spawn_explosion(&mut world.commands(), origin, 3, &mut rand);
            });
        }
        _ => {
            // Placeholder for other effects - implement as needed
            let target = target.unwrap_or(origin);
            world.resource_scope(|world, _: Mut<Rand>| {
                spawn_pistol_shot(&mut world.commands(), origin, target);
            });
        }
    }
}

// Update particle positions and handle bullet movement
pub fn update_particle_movement(
    mut q_bullets: Query<(&mut Position, &mut ParticleVelocity, &mut Particle), With<ParticleVelocity>>,
    time: Res<Time>,
) {
    for (mut position, mut velocity, mut particle) in q_bullets.iter_mut() {
        // Update lifetime
        particle.lifetime += time.dt;
        
        if velocity.path.is_empty() {
            continue;
        }
        
        // Calculate how far along the path we should be based on time and speed
        let distance_traveled = velocity.speed * particle.lifetime;
        let new_path_index = (distance_traveled as usize).min(velocity.path.len() - 2); // -2 because we skip first position
        
        // Update path index if we've moved to a new position
        velocity.path_index = new_path_index;
        
        // Set position to current path position (offset by 1 to skip origin)
        let current_path_pos = velocity.path[velocity.path_index + 1];
        position.x = current_path_pos.0 as f32;
        position.y = current_path_pos.1 as f32;
        position.z = current_path_pos.2 as f32;
    }
}

// Update particle colors and handle lifetime
pub fn update_particle_colors_and_lifetime(
    mut commands: Commands,
    mut q_particles: Query<(Entity, &mut Particle, &mut Glyph), With<Particle>>,
    mut rand: ResMut<Rand>,
    time: Res<Time>,
) {
    for (entity, mut particle, mut glyph) in q_particles.iter_mut() {
        // Update lifetime first
        particle.lifetime += time.dt;
        let progress = if particle.max_lifetime > 0.0 {
            (particle.lifetime / particle.max_lifetime).clamp(0.0, 1.0)
        } else {
            1.0
        };
        
        // Apply fade curve
        let fade_progress = particle.fade_curve.apply(progress);
        
        // Update particle alpha based on lifetime - fade to transparent over time
        particle.alpha = match particle.particle_type {
            ParticleType::Smoke { .. } => {
                // Smoke fades to transparent more aggressively
                1.0 - fade_progress
            },
            _ => {
                // Other particles (bullets, explosions) maintain opacity until near end
                if fade_progress > 0.8 {
                    1.0 - ((fade_progress - 0.8) / 0.2) // Fade out in last 20% of lifetime
                } else {
                    1.0 // Stay fully opaque for first 80% of lifetime
                }
            }
        };
        
        // For smoke particles, keep the base color and only fade alpha
        let current_color = if matches!(particle.particle_type, ParticleType::Smoke { .. }) {
            particle.base_color.into()
        } else {
            // For other particles, interpolate colors as before
            let start_color: u32 = particle.base_color.into();
            let end_color: u32 = particle.fade_color.into();
            interpolate_color(start_color, end_color, fade_progress)
        };
        
        // Update glyph color and alpha
        glyph.fg1 = Some(current_color);
        glyph.alpha = particle.alpha;

        let rgb = current_color.to_rgba(particle.alpha);
        trace!("{},{},{},{}", rgb[0], rgb[1], rgb[1], rgb[3]);

        // Update glyph index for randomized particles
        let new_glyph_idx = particle.particle_type.get_glyph(&mut rand);
        glyph.idx = new_glyph_idx;
        
        // Remove expired particles
        if particle.lifetime >= particle.max_lifetime {
            commands.entity(entity).despawn();
        }
    }
}

// Handle smoke trail spawning behind bullets
pub fn update_smoke_trails(
    mut commands: Commands,
    mut q_smoke_trails: Query<&mut SmokeTrail>,
    q_bullets: Query<&Position, With<ParticleVelocity>>,
    mut rand: ResMut<Rand>,
) {
    for mut smoke_trail in q_smoke_trails.iter_mut() {
        if let Ok(bullet_position) = q_bullets.get(smoke_trail.bullet_entity) {
            let current_pos = (bullet_position.x.round() as usize, bullet_position.y.round() as usize, bullet_position.z.round() as usize);
            
            // Check if bullet has moved to a new position
            if smoke_trail.last_position.is_none() || smoke_trail.last_position != Some(current_pos) {
                // Schedule delayed smoke spawn at the previous position (not current)
                if let Some(last_pos) = smoke_trail.last_position {
                    // Calculate progress along bullet path for persistence variation
                    let path_progress = if smoke_trail.bullet_path_length > 1 {
                        // Estimate current position in path based on distance from start
                        // This is approximate since we don't have exact path tracking here
                        let total_distance = smoke_trail.bullet_path_length as f32;
                        let estimated_progress = (total_distance - 1.0) / total_distance.max(1.0);
                        estimated_progress.clamp(0.0, 1.0)
                    } else {
                        0.5 // Default middle progress
                    };
                    
                    // No delay - spawn immediately
                    let spawn_delay = 0.0;
                    
                    commands.spawn(SmokeSpawnTimer {
                        delay: spawn_delay,
                        position: last_pos,
                        path_progress,
                    });
                }
                
                smoke_trail.last_position = Some(current_pos);
            }
        } else {
            // Bullet no longer exists, despawn this smoke trail tracker
            commands.entity(smoke_trail.bullet_entity).despawn();
        }
    }
}

// Clean up smoke trail trackers when bullets are despawned
pub fn cleanup_smoke_trails(
    mut commands: Commands,
    q_smoke_trails: Query<(Entity, &SmokeTrail)>,
    q_bullets: Query<Entity, With<ParticleVelocity>>,
) {
    let active_bullets: std::collections::HashSet<Entity> = q_bullets.iter().collect();
    
    for (trail_entity, smoke_trail) in q_smoke_trails.iter() {
        if !active_bullets.contains(&smoke_trail.bullet_entity) {
            commands.entity(trail_entity).despawn();
        }
    }
}

// Handle delayed smoke spawning
pub fn update_smoke_spawn_timers(
    mut commands: Commands,
    mut q_timers: Query<(Entity, &mut SmokeSpawnTimer)>,
    mut rand: ResMut<Rand>,
    time: Res<Time>,
) {
    for (entity, mut timer) in q_timers.iter_mut() {
        timer.delay -= time.dt;
        
        if timer.delay <= 0.0 {
            // Calculate lifetime based on path progress (persistence variation)
            let base_lifetime = if timer.path_progress < 0.3 {
                // Muzzle smoke (first 30% of path): longer lifetime
                0.8 + rand.random() * 0.4 // 0.8-1.2 seconds
            } else if timer.path_progress > 0.7 {
                // End-trail smoke (last 30% of path): shorter lifetime
                0.4 + rand.random() * 0.3 // 0.4-0.7 seconds
            } else {
                // Mid-trail smoke: medium lifetime
                0.6 + rand.random() * 0.3 // 0.6-0.9 seconds
            };
            
            // Spawn the actual smoke particle
            commands.spawn((
                Particle {
                    particle_type: ParticleType::Smoke { 
                        glyphs: vec![224, 224, 225],
                    },
                    lifetime: 0.0,
                    max_lifetime: base_lifetime,
                    base_color: Palette::White,
                    fade_color: Palette::White,
                    fade_curve: FadeCurve::EaseOut,
                    alpha: 1.0, // Smoke starts fully opaque, will fade to transparent
                },
                Position::new(timer.position.0, timer.position.1, timer.position.2),
                Glyph::idx(46).fg1(Palette::White as u32).layer(Layer::Overlay),
                Visibility::Visible,
                IgnoreLighting,
            ));
            
            // Remove the timer entity
            commands.entity(entity).despawn();
        }
    }
}

fn interpolate_color(start_color: u32, end_color: u32, progress: f32) -> u32 {
    let progress = progress.clamp(0.0, 1.0);

    let start_r = ((start_color >> 16) & 0xFF) as f32;
    let start_g = ((start_color >> 8) & 0xFF) as f32;
    let start_b = (start_color & 0xFF) as f32;

    let end_r = ((end_color >> 16) & 0xFF) as f32;
    let end_g = ((end_color >> 8) & 0xFF) as f32;
    let end_b = (end_color & 0xFF) as f32;

    // Gamma-correct mixing like the Haxe Mix function
    let mix_part = |p1: f32, p2: f32, t: f32| -> f32 {
        let v = (1.0 - t) * p1.powi(2) + t * p2.powi(2);
        v.sqrt()
    };

    let r = (mix_part(start_r, end_r, progress).clamp(0.0, 255.0).round() as u32).min(255);
    let g = (mix_part(start_g, end_g, progress).clamp(0.0, 255.0).round() as u32).min(255);
    let b = (mix_part(start_b, end_b, progress).clamp(0.0, 255.0).round() as u32).min(255);

    (r << 16) | (g << 8) | b
}
