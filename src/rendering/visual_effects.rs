use bevy_ecs::prelude::*;
use macroquad::prelude::trace;

use super::{Layer, Position};
use crate::{
    common::{
        MacroquadColorable, Palette, Rand, algorithm::bresenham::bresenham_line, lerp_u32_colors,
    },
    domain::IgnoreLighting,
    engine::Time,
    rendering::{Glyph, Visibility},
};

// Debug speed modifier - affects all particle timing (movement, fading, lifetime)
// 1.0 = normal speed, 2.0 = 2x faster, 0.5 = half speed
const DEBUG_SPEED_MOD: f32 = 1.0;

// Zone size constants (assuming standard zone dimensions)
const ZONE_WIDTH: usize = 80;
const ZONE_HEIGHT: usize = 40;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ParticlePriority {
    Smoke = 10,
    Impact = 90,
    Bullet = 100,
}

#[derive(Clone)]
pub struct ParticleData {
    pub particle_type: ParticleType,
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub base_color: Palette,
    pub fade_color: Palette,
    pub fade_curve: FadeCurve,
    pub alpha: f32,
    pub priority: ParticlePriority,
    pub position: (usize, usize, usize), // World coordinates
    pub velocity: Option<ParticleVelocityData>, // For moving particles
}

#[derive(Clone)]
pub struct ParticleVelocityData {
    pub path: Vec<(usize, usize, usize)>,
    pub speed: f32,
    pub path_index: usize,
}

#[derive(Resource)]
pub struct ParticleGrid {
    // 2D grid for each Z level - maps (x, y, z) to particles at that position
    grids: std::collections::HashMap<usize, Vec<Vec<Vec<ParticleData>>>>, // z -> y -> x -> particles
}

impl ParticleGrid {
    pub fn new() -> Self {
        Self {
            grids: std::collections::HashMap::new(),
        }
    }

    pub fn ensure_z_level(&mut self, z: usize) {
        if !self.grids.contains_key(&z) {
            // Create 2D grid for this Z level
            let mut grid = Vec::with_capacity(ZONE_HEIGHT);
            for _ in 0..ZONE_HEIGHT {
                let mut row = Vec::with_capacity(ZONE_WIDTH);
                for _ in 0..ZONE_WIDTH {
                    row.push(Vec::new());
                }
                grid.push(row);
            }
            self.grids.insert(z, grid);
        }
    }

    pub fn add_particle(&mut self, particle: ParticleData) {
        let (x, y, z) = particle.position;
        if x >= ZONE_WIDTH || y >= ZONE_HEIGHT {
            return; // Out of bounds
        }
        
        self.ensure_z_level(z);
        if let Some(grid) = self.grids.get_mut(&z) {
            grid[y][x].push(particle);
        }
    }

    pub fn clear_all(&mut self) {
        for (_, grid) in self.grids.iter_mut() {
            for row in grid.iter_mut() {
                for cell in row.iter_mut() {
                    cell.clear();
                }
            }
        }
    }

    pub fn get_highest_priority_particle(&self, x: usize, y: usize, z: usize) -> Option<&ParticleData> {
        if x >= ZONE_WIDTH || y >= ZONE_HEIGHT {
            return None;
        }
        
        if let Some(grid) = self.grids.get(&z) {
            let particles = &grid[y][x];
            particles.iter().max_by_key(|p| p.priority)
        } else {
            None
        }
    }

    pub fn iter_positions(&self, z: usize) -> impl Iterator<Item = (usize, usize, &ParticleData)> + '_ {
        self.grids.get(&z).into_iter().flat_map(move |grid| {
            grid.iter().enumerate().flat_map(move |(y, row)| {
                row.iter().enumerate().filter_map(move |(x, particles)| {
                    particles.iter().max_by_key(|p| p.priority).map(|p| (x, y, p))
                })
            })
        })
    }
}

#[derive(Resource)]
pub struct ParticleEntityPool {
    available_entities: Vec<Entity>,
    active_entities: std::collections::HashMap<(usize, usize, usize), Entity>, // position -> entity
}

impl ParticleEntityPool {
    pub fn new() -> Self {
        Self {
            available_entities: Vec::new(),
            active_entities: std::collections::HashMap::new(),
        }
    }

    pub fn get_or_spawn_entity(
        &mut self, 
        commands: &mut Commands, 
        position: (usize, usize, usize)
    ) -> Entity {
        if let Some(entity) = self.active_entities.get(&position) {
            *entity
        } else {
            let entity = if let Some(entity) = self.available_entities.pop() {
                entity
            } else {
                commands.spawn_empty().id()
            };
            self.active_entities.insert(position, entity);
            entity
        }
    }

    pub fn release_entity(&mut self, position: (usize, usize, usize)) {
        if let Some(entity) = self.active_entities.remove(&position) {
            self.available_entities.push(entity);
        }
    }

    pub fn clear_all(&mut self) {
        for (_, entity) in self.active_entities.drain() {
            self.available_entities.push(entity);
        }
    }
}

impl ParticleType {
    pub fn get_priority(&self) -> ParticlePriority {
        match self {
            ParticleType::Bullet { .. } => ParticlePriority::Bullet,
            ParticleType::Smoke { .. } => ParticlePriority::Smoke,
            ParticleType::Explosion { .. } => ParticlePriority::Impact, // Treat as impact priority
            ParticleType::Spark { .. } => ParticlePriority::Impact,
            ParticleType::Impact { .. } => ParticlePriority::Impact,
        }
    }
}

impl ParticleData {
    pub fn new(
        particle_type: ParticleType,
        lifetime: f32,
        max_lifetime: f32,
        base_color: Palette,
        fade_color: Palette,
        fade_curve: FadeCurve,
        position: (usize, usize, usize),
        velocity: Option<ParticleVelocityData>,
    ) -> Self {
        let priority = particle_type.get_priority();
        Self {
            particle_type,
            lifetime,
            max_lifetime,
            base_color,
            fade_color,
            fade_curve,
            alpha: 1.0,
            priority,
            position,
            velocity,
        }
    }
}

// New grid-based particle update system
pub fn update_particle_grid(
    mut particle_grid: ResMut<ParticleGrid>,
    time: Res<Time>,
    mut rand: ResMut<Rand>,
) {
    // Clear the grid for this frame
    particle_grid.clear_all();
    
    // Update all particles and re-add them to the grid
    let mut updated_particles = Vec::new();
    
    for (z, grid) in particle_grid.grids.iter() {
        for (y, row) in grid.iter().enumerate() {
            for (x, particles) in row.iter().enumerate() {
                for particle in particles {
                    let mut updated_particle = particle.clone();
                    
                    // Update lifetime
                    updated_particle.lifetime += time.dt * DEBUG_SPEED_MOD;
                    
                    // Skip expired particles
                    if updated_particle.lifetime >= updated_particle.max_lifetime {
                        continue;
                    }
                    
                    // Handle delayed particles
                    if updated_particle.lifetime < 0.0 {
                        updated_particles.push(updated_particle);
                        continue;
                    }
                    
                    // Update particle position if it has velocity
                    if let Some(ref mut velocity) = updated_particle.velocity {
                        if !velocity.path.is_empty() {
                            let distance_traveled = velocity.speed * updated_particle.lifetime;
                            let new_path_index = (distance_traveled as usize).min(velocity.path.len() - 2);
                            velocity.path_index = new_path_index;
                            
                            if new_path_index + 1 < velocity.path.len() {
                                updated_particle.position = velocity.path[new_path_index + 1];
                            }
                        }
                    }
                    
                    // Update alpha and color based on lifetime
                    let progress = if updated_particle.max_lifetime > 0.0 {
                        (updated_particle.lifetime / updated_particle.max_lifetime).clamp(0.0, 1.0)
                    } else {
                        1.0
                    };
                    
                    let fade_progress = updated_particle.fade_curve.apply(progress);
                    
                    // Update alpha based on particle type
                    updated_particle.alpha = match updated_particle.particle_type {
                        ParticleType::Smoke { .. } => 1.0 - fade_progress,
                        ParticleType::Spark { .. } => 1.0 - fade_progress,
                        ParticleType::Impact { .. } => 1.0 - (fade_progress * 0.9).min(1.0),
                        _ => {
                            if fade_progress > 0.8 {
                                1.0 - ((fade_progress - 0.8) / 0.2)
                            } else {
                                1.0
                            }
                        }
                    };
                    
                    updated_particles.push(updated_particle);
                }
            }
        }
    }
    
    // Re-add all updated particles to the grid
    for particle in updated_particles {
        particle_grid.add_particle(particle);
    }
}

// Render particles from the grid using entity pool
pub fn render_particle_grid(
    mut commands: Commands,
    particle_grid: Res<ParticleGrid>,
    mut entity_pool: ResMut<ParticleEntityPool>,
    mut rand: ResMut<Rand>,
    // Query existing particle entities to update them
    mut q_particles: Query<(&mut Glyph, &mut Position), With<Particle>>,
) {
    // Clear previous frame's active entities
    let previous_positions: Vec<_> = entity_pool.active_entities.keys().cloned().collect();
    for pos in previous_positions {
        entity_pool.release_entity(pos);
    }
    
    // Render highest priority particle for each position across all Z levels
    for (z, _) in particle_grid.grids.iter() {
        for (x, y, particle_data) in particle_grid.iter_positions(*z) {
            let position = (x, y, *z);
            let entity = entity_pool.get_or_spawn_entity(&mut commands, position);
            
            // Get the current glyph to render
            let glyph_idx = particle_data.particle_type.get_glyph(&mut rand);
            
            // Calculate current color
            let current_color = match particle_data.particle_type {
                ParticleType::Smoke { .. } => particle_data.base_color.into(),
                _ => {
                    let progress = if particle_data.max_lifetime > 0.0 {
                        (particle_data.lifetime / particle_data.max_lifetime).clamp(0.0, 1.0)
                    } else {
                        1.0
                    };
                    let fade_progress = particle_data.fade_curve.apply(progress);
                    let start_color: u32 = particle_data.base_color.into();
                    let end_color: u32 = particle_data.fade_color.into();
                    lerp_u32_colors(start_color, end_color, fade_progress)
                }
            };
            
            // Update or insert the particle components
            if let Ok((mut glyph, mut pos)) = q_particles.get_mut(entity) {
                // Update existing particle
                glyph.idx = glyph_idx;
                glyph.fg1 = Some(current_color);
                glyph.alpha = particle_data.alpha;
                pos.x = x as f32;
                pos.y = y as f32;
                pos.z = *z as f32;
            } else {
                // Insert new components
                commands.entity(entity).insert((
                    Glyph::idx(glyph_idx)
                        .fg1(current_color)
                        .layer(Layer::Overlay),
                    Position::new(x, y, *z),
                    Visibility::Visible,
                    IgnoreLighting,
                    Particle {
                        particle_type: particle_data.particle_type.clone(),
                        lifetime: particle_data.lifetime,
                        max_lifetime: particle_data.max_lifetime,
                        base_color: particle_data.base_color,
                        fade_color: particle_data.fade_color,
                        fade_curve: particle_data.fade_curve.clone(),
                        alpha: particle_data.alpha,
                    },
                ));
            }
        }
    }
}

// Grid-based particle spawn functions
pub fn spawn_bullet_to_grid(
    particle_grid: &mut ParticleGrid,
    origin: (usize, usize, usize),
    target: (usize, usize, usize),
) {
    // Calculate Bresenham path from origin to target
    let path_2d = bresenham_line((origin.0, origin.1), (target.0, target.1));
    let path_3d: Vec<(usize, usize, usize)> =
        path_2d.iter().map(|(x, y)| (*x, *y, origin.2)).collect();

    if path_3d.len() < 2 {
        return;
    }

    let start_pos = path_3d[1];
    let velocity = Some(ParticleVelocityData {
        path: path_3d.clone(),
        speed: 32.0,
        path_index: 0,
    });

    let particle = ParticleData::new(
        ParticleType::Bullet { glyph: 140 },
        0.0,
        calculate_travel_time_path(&path_3d, 32.0) / DEBUG_SPEED_MOD,
        Palette::Red,
        Palette::Red,
        FadeCurve::Linear,
        start_pos,
        velocity,
    );

    particle_grid.add_particle(particle);
}


pub fn spawn_impact_effects_to_grid(
    particle_grid: &mut ParticleGrid,
    impact_pos: (usize, usize, usize),
    rand: &mut Rand,
) {
    // Spawn spark particles
    let spark_glyphs = vec![138, 139];
    let spark_count = 2;
    let colors = (Palette::Yellow, Palette::Orange);

    for _ in 0..spark_count {
        let offset_x = rand.range_n(-1, 2) as f32;
        let offset_y = rand.range_n(-1, 2) as f32;
        let spark_x = (impact_pos.0 as f32 + offset_x).max(0.0) as usize;
        let spark_y = (impact_pos.1 as f32 + offset_y).max(0.0) as usize;

        let glyph = spark_glyphs[rand.range_n(0, spark_glyphs.len() as i32) as usize];
        let spark_pos = (spark_x, spark_y, impact_pos.2);

        let particle = ParticleData::new(
            ParticleType::Spark { glyphs: spark_glyphs.clone() },
            0.0,
            (0.2 + rand.random() * 0.3) / DEBUG_SPEED_MOD,
            colors.0,
            colors.1,
            FadeCurve::EaseOut,
            spark_pos,
            None,
        );

        particle_grid.add_particle(particle);
    }

    // Add central impact particle
    let impact_particle = ParticleData::new(
        ParticleType::Impact { glyphs: vec![137] },
        0.0,
        0.4 / DEBUG_SPEED_MOD,
        colors.0,
        colors.1,
        FadeCurve::Linear,
        impact_pos,
        None,
    );

    particle_grid.add_particle(impact_particle);
}

// New grid-based weapon spawn functions
pub fn spawn_rifle_shot_grid(
    mut particle_grid: ResMut<ParticleGrid>,
    origin: (usize, usize, usize),
    target: (usize, usize, usize),
    rand: &mut Rand,
) {
    spawn_bullet_to_grid(&mut particle_grid, origin, target);
    spawn_impact_effects_to_grid(&mut particle_grid, target, rand);
}

pub fn spawn_pistol_shot_grid(
    mut particle_grid: ResMut<ParticleGrid>,
    origin: (usize, usize, usize),
    target: (usize, usize, usize),
    rand: &mut Rand,
) {
    spawn_bullet_to_grid(&mut particle_grid, origin, target);
    spawn_impact_effects_to_grid(&mut particle_grid, target, rand);
}

pub fn spawn_shotgun_blast_grid(
    mut particle_grid: ResMut<ParticleGrid>,
    origin: (usize, usize, usize),
    target: (usize, usize, usize),
    rand: &mut Rand,
) {
    spawn_bullet_to_grid(&mut particle_grid, origin, target);
    spawn_impact_effects_to_grid(&mut particle_grid, target, rand);
}

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
    pub speed: f32,        // grid cells per second
    pub path_index: usize, // current position along path
}

#[derive(Component)]
pub struct SmokeTrail {
    pub bullet_entity: Entity,
    pub last_position: Option<(usize, usize, usize)>,
    pub bullet_path_length: usize,
    pub weapon_type: WeaponType,
}

#[derive(Clone, Copy)]
pub enum WeaponType {
    Pistol,
    Rifle,
    Shotgun,
}

#[derive(Component)]
pub struct SmokeSpawnTimer {
    pub delay: f32,
    pub position: (usize, usize, usize),
    pub path_progress: f32, // 0.0 = start (muzzle), 1.0 = end
    pub weapon_type: WeaponType,
}

#[derive(Clone)]
pub enum ParticleType {
    Bullet { glyph: usize },
    Smoke { glyphs: Vec<usize> },
    Explosion { glyphs: Vec<usize> },
    Spark { glyphs: Vec<usize> }, // Impact sparks using symbolic glyphs
    Impact { glyphs: Vec<usize> }, // Ricochet/impact effects
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum VisualEffectId {
    PistolShot,
    RifleShot,
    ShotgunBlast,
    DynamiteExplosion,
}

#[derive(Clone)]
pub enum FadeCurve {
    Linear,
    EaseOut,
}

impl FadeCurve {
    pub fn apply(&self, progress: f32) -> f32 {
        match self {
            FadeCurve::Linear => progress,
            FadeCurve::EaseOut => 1.0 - (1.0 - progress) * (1.0 - progress),
        }
    }
}

impl ParticleType {
    pub fn get_glyph(&self, rand: &mut Rand) -> usize {
        match self {
            ParticleType::Bullet { glyph } => *glyph,
            ParticleType::Smoke { glyphs }
            | ParticleType::Explosion { glyphs }
            | ParticleType::Spark { glyphs }
            | ParticleType::Impact { glyphs } => {
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
    rand: &mut Rand,
) {
    // Calculate Bresenham path from origin to target (keep full path)
    let path_2d = bresenham_line((origin.0, origin.1), (target.0, target.1));
    let path_3d: Vec<(usize, usize, usize)> =
        path_2d.iter().map(|(x, y)| (*x, *y, origin.2)).collect();

    // Need at least 2 positions (origin + at least one step)
    if path_3d.len() < 2 {
        return; // No path to follow or adjacent shot
    }

    // Spawn bullet particle starting at second position (skip origin)
    let start_pos = path_3d[1];
    let path_length = path_3d.len();
    let bullet_entity = commands
        .spawn((
            Particle {
                particle_type: ParticleType::Bullet { glyph: 140 },
                lifetime: 0.0,
                max_lifetime: calculate_travel_time_path(&path_3d, 32.0) / DEBUG_SPEED_MOD, // 32 grid cells per second
                base_color: Palette::Red,
                fade_color: Palette::Red,
                fade_curve: FadeCurve::Linear,
                alpha: 1.0, // Bullets start fully opaque
            },
            ParticleVelocity {
                path: path_3d,
                speed: 32.0,
                path_index: 0, // Track from beginning, but bullet starts at path[1]
            },
            Position::new(start_pos.0, start_pos.1, start_pos.2),
            Glyph::idx(140)
                .fg1(Palette::Red as u32)
                .layer(Layer::Overlay),
            Visibility::Visible,
            IgnoreLighting,
        ))
        .id();

    // Spawn impact effects at target
    spawn_impact_effects(commands, target, WeaponType::Pistol, rand);
    
    // Spawn smoke trail tracker
    commands.spawn(SmokeTrail {
        bullet_entity,
        last_position: None,
        bullet_path_length: path_length,
        weapon_type: WeaponType::Rifle,
    });
}

pub fn spawn_pistol_shot(
    commands: &mut Commands,
    origin: (usize, usize, usize),
    target: (usize, usize, usize),
    rand: &mut Rand,
) {
    // Calculate Bresenham path from origin to target (keep full path)
    let path_2d = bresenham_line((origin.0, origin.1), (target.0, target.1));
    let path_3d: Vec<(usize, usize, usize)> =
        path_2d.iter().map(|(x, y)| (*x, *y, origin.2)).collect();

    // Need at least 2 positions (origin + at least one step)
    if path_3d.len() < 2 {
        return; // No path to follow or adjacent shot
    }

    // Similar to rifle but faster, different colors
    let start_pos = path_3d[1]; // Start at second position (skip origin)
    let path_length = path_3d.len();
    let bullet_entity = commands
        .spawn((
            Particle {
                particle_type: ParticleType::Bullet { glyph: 140 },
                lifetime: 0.0,
                max_lifetime: calculate_travel_time_path(&path_3d, 32.0) / DEBUG_SPEED_MOD, // 32 grid cells per second (4x faster)
                base_color: Palette::Red,
                fade_color: Palette::Red,
                fade_curve: FadeCurve::Linear,
                alpha: 1.0, // Bullets start fully opaque
            },
            ParticleVelocity {
                path: path_3d,
                speed: 32.0,
                path_index: 0, // Track from beginning, but bullet starts at path[1]
            },
            Position::new(start_pos.0, start_pos.1, start_pos.2),
            Glyph::idx(140)
                .fg1(Palette::Red as u32)
                .layer(Layer::Overlay),
            Visibility::Visible,
            IgnoreLighting,
        ))
        .id();

    // Spawn impact effects at target
    spawn_impact_effects(commands, target, WeaponType::Pistol, rand);
    
    // Spawn smoke trail tracker
    commands.spawn(SmokeTrail {
        bullet_entity,
        last_position: None,
        bullet_path_length: path_length,
        weapon_type: WeaponType::Pistol,
    });
}

pub fn spawn_shotgun_blast(
    commands: &mut Commands,
    origin: (usize, usize, usize),
    target: (usize, usize, usize),
    rand: &mut Rand,
) {
    // Calculate Bresenham path from origin to target (keep full path)
    let path_2d = bresenham_line((origin.0, origin.1), (target.0, target.1));
    let path_3d: Vec<(usize, usize, usize)> =
        path_2d.iter().map(|(x, y)| (*x, *y, origin.2)).collect();

    // Need at least 2 positions (origin + at least one step)
    if path_3d.len() < 2 {
        return; // No path to follow or adjacent shot
    }

    // Spawn single bullet particle (same as pistol)
    let start_pos = path_3d[1];
    let path_length = path_3d.len();
    let bullet_entity = commands
        .spawn((
            Particle {
                particle_type: ParticleType::Bullet { glyph: 140 },
                lifetime: 0.0,
                max_lifetime: calculate_travel_time_path(&path_3d, 32.0) / DEBUG_SPEED_MOD, // 32 grid cells per second
                base_color: Palette::Red,
                fade_color: Palette::Red,
                fade_curve: FadeCurve::Linear,
                alpha: 1.0, // Bullets start fully opaque
            },
            ParticleVelocity {
                path: path_3d,
                speed: 32.0,
                path_index: 0, // Track from beginning, but bullet starts at path[1]
            },
            Position::new(start_pos.0, start_pos.1, start_pos.2),
            Glyph::idx(140)
                .fg1(Palette::Red as u32)
                .layer(Layer::Overlay),
            Visibility::Visible,
            IgnoreLighting,
        ))
        .id();

    // Spawn impact effects at target (main target for visual feedback)
    spawn_impact_effects(commands, target, WeaponType::Pistol, rand);
    
    // Spawn smoke trail tracker
    commands.spawn(SmokeTrail {
        bullet_entity,
        last_position: None,
        bullet_path_length: path_length,
        weapon_type: WeaponType::Pistol,
    });
}

fn calculate_travel_time_path(path: &[(usize, usize, usize)], speed: f32) -> f32 {
    if path.len() <= 1 {
        return 0.1; // minimum time
    }
    // Path length is number of steps - 1 (since we count transitions)
    let distance = (path.len() - 1) as f32;
    distance / speed
}



fn spawn_impact_effects(
    commands: &mut Commands,
    impact_pos: (usize, usize, usize),
    _weapon_type: WeaponType, // Unused but kept for API compatibility
    rand: &mut Rand,
) {
    // All weapons use same impact effects (pistol style)
    let spark_glyphs = vec![138, 139]; // * and + symbols
    let spark_count = 2;
    let colors = (Palette::Yellow, Palette::Orange);

    // Create spark particles around impact point
    for _ in 0..spark_count {
        // Smaller random offset around impact point
        let offset_x = rand.range_n(-1, 2) as f32;
        let offset_y = rand.range_n(-1, 2) as f32;
        let spark_x = (impact_pos.0 as f32 + offset_x).max(0.0) as usize;
        let spark_y = (impact_pos.1 as f32 + offset_y).max(0.0) as usize;

        let glyph = spark_glyphs[rand.range_n(0, spark_glyphs.len() as i32) as usize];
        
        commands.spawn((
            Particle {
                particle_type: ParticleType::Spark { glyphs: spark_glyphs.clone() },
                lifetime: 0.0,
                max_lifetime: (0.2 + rand.random() * 0.3) / DEBUG_SPEED_MOD, // Slower sparks
                base_color: colors.0,
                fade_color: colors.1,
                fade_curve: FadeCurve::EaseOut,
                alpha: 1.0,
            },
            Position::new(spark_x, spark_y, impact_pos.2),
            Glyph::idx(glyph)
                .fg1(colors.0 as u32)
                .layer(Layer::Overlay),
            Visibility::Visible,
            IgnoreLighting,
        ));
    }

    // Add a central impact particle
    commands.spawn((
        Particle {
            particle_type: ParticleType::Impact { glyphs: vec![137] }, // # symbol for impact
            lifetime: 0.0,
            max_lifetime: 0.4 / DEBUG_SPEED_MOD,
            base_color: colors.0,
            fade_color: colors.1,
            fade_curve: FadeCurve::Linear,
            alpha: 1.0,
        },
        Position::new(impact_pos.0, impact_pos.1, impact_pos.2),
        Glyph::idx(137)
            .fg1(colors.0 as u32)
            .layer(Layer::Overlay),
        Visibility::Visible,
        IgnoreLighting,
    ));
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
                            glyphs: vec![42, 43, 35, 46], // *, +, #, .
                        },
                        lifetime: 0.0,
                        max_lifetime: lifetime,
                        base_color: Palette::Red,
                        fade_color: Palette::DarkRed,
                        fade_curve: FadeCurve::EaseOut,
                        alpha: 1.0, // Explosions start fully opaque
                    },
                    Position::new(position.0, position.1, position.2),
                    Glyph::idx(glyph as usize)
                        .fg1(Palette::Red as u32)
                        .layer(Layer::Overlay),
                    Visibility::Visible,
                    IgnoreLighting,
                ));
            }
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
            world.resource_scope(|world, mut rand: Mut<Rand>| {
                spawn_pistol_shot(&mut world.commands(), origin, target, &mut rand);
            });
        }
        VisualEffectId::RifleShot => {
            let target = target.expect("RifleShot requires target position");
            world.resource_scope(|world, mut rand: Mut<Rand>| {
                spawn_rifle_shot(&mut world.commands(), origin, target, &mut rand);
            });
        }
        VisualEffectId::ShotgunBlast => {
            let target = target.expect("ShotgunBlast requires target position");
            world.resource_scope(|world, mut rand: Mut<Rand>| {
                spawn_shotgun_blast(&mut world.commands(), origin, target, &mut rand);
            });
        }
        VisualEffectId::DynamiteExplosion => {
            world.resource_scope(|world, mut rand: Mut<Rand>| {
                spawn_explosion(&mut world.commands(), origin, 3, &mut rand);
            });
        }
    }
}

// Update particle positions and handle bullet movement
pub fn update_particle_movement(
    mut q_bullets: Query<
        (&mut Position, &mut ParticleVelocity, &mut Particle),
        With<ParticleVelocity>,
    >,
    time: Res<Time>,
) {
    for (mut position, mut velocity, mut particle) in q_bullets.iter_mut() {
        // Update lifetime
        particle.lifetime += time.dt * DEBUG_SPEED_MOD;

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
        particle.lifetime += time.dt * DEBUG_SPEED_MOD;
        
        // Handle delayed particles (negative lifetime = not started yet)
        if particle.lifetime < 0.0 {
            // Hide particle and continue to next iteration
            particle.alpha = 0.0;
            glyph.fg1 = Some((particle.base_color as u32 & 0xFFFFFF) | (0x00 << 24)); // Fully transparent
            continue;
        }
        
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
                // All smoke fades uniformly
                1.0 - fade_progress
            }
            ParticleType::Spark { .. } => {
                // Sparks fade more gradually now
                1.0 - fade_progress // Linear fade
            }
            ParticleType::Impact { .. } => {
                // Impact effects fade more gradually
                1.0 - (fade_progress * 0.9).min(1.0) // Slower, more visible fade
            }
            _ => {
                // Other particles (bullets, explosions) maintain opacity until near end
                if fade_progress > 0.8 {
                    1.0 - ((fade_progress - 0.8) / 0.2) // Fade out in last 20% of lifetime
                } else {
                    1.0 // Stay fully opaque for first 80% of lifetime
                }
            }
        };

        // Handle color transitions for different particle types
        let current_color = match particle.particle_type {
            ParticleType::Smoke { .. } => {
                // Smoke particles keep base color and only fade alpha
                particle.base_color.into()
            }
            _ => {
                // Other particles interpolate colors over lifetime
                let start_color: u32 = particle.base_color.into();
                let end_color: u32 = particle.fade_color.into();
                lerp_u32_colors(start_color, end_color, fade_progress)
            }
        };

        // Update glyph color and alpha
        glyph.fg1 = Some(current_color);
        // glyph.bg = Some(current_color);
        glyph.alpha = particle.alpha;

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
    _rand: ResMut<Rand>,
) {
    // Temporarily disabled to make bullet trails more visible
    // return;
    for mut smoke_trail in q_smoke_trails.iter_mut() {
        if let Ok(bullet_position) = q_bullets.get(smoke_trail.bullet_entity) {
            let current_pos = (
                bullet_position.x.round() as usize,
                bullet_position.y.round() as usize,
                bullet_position.z.round() as usize,
            );

            // Check if bullet has moved to a new position
            if smoke_trail.last_position.is_none() || smoke_trail.last_position != Some(current_pos)
            {
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
                        weapon_type: smoke_trail.weapon_type,
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
    // Temporarily disabled to make bullet trails more visible
    // return;
    for (entity, mut timer) in q_timers.iter_mut() {
        timer.delay -= time.dt * DEBUG_SPEED_MOD;

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

            // All weapons use same smoke (pistol style)
            let particle_type = ParticleType::Smoke {
                glyphs: vec![228, 229, 230], // Light glyphs
            };
            let lifetime_multiplier = 0.7; // Shorter lasting
            
            // Spawn the actual smoke particle
            commands.spawn((
                Particle {
                    particle_type,
                    lifetime: 0.0,
                    max_lifetime: (base_lifetime * lifetime_multiplier) / DEBUG_SPEED_MOD,
                    base_color: Palette::White,
                    fade_color: Palette::White,
                    fade_curve: FadeCurve::EaseOut,
                    alpha: 1.0, // Smoke starts fully opaque, will fade to transparent
                },
                Position::new(timer.position.0, timer.position.1, timer.position.2),
                Glyph::idx(228)
                    .fg1(Palette::White as u32)
                    .layer(Layer::Overlay),
                Visibility::Visible,
                IgnoreLighting,
            ));

            // Remove the timer entity
            commands.entity(entity).despawn();
        }
    }
}
