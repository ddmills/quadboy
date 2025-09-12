use bevy_ecs::prelude::*;
use macroquad::math::Vec2;

use crate::{
    cfg::ZONE_SIZE,
    common::{Grid, Palette},
    rendering::{Glyph, GlyphTextureId, Position, Visibility},
};

use crate::rendering::Layer;
use super::curves::{ColorCurve, VelocityCurve, AlphaCurve, CurveEvaluator};
use super::spawn_areas::SpawnArea;

#[derive(Resource)]
pub struct ParticleGrid {
    pub data: Grid<Vec<Fragment>>,
}

impl Default for ParticleGrid {
    fn default() -> Self {
        Self {
            data: Grid::init_fill(ZONE_SIZE.0 * 2, ZONE_SIZE.1 * 2, |_, _| vec![]),
        }
    }
}

impl ParticleGrid {
    pub fn get(&self, x: usize, y: usize) -> Option<&Fragment> {
        let fragments = self.data.get(x, y)?;

        fragments.iter().max_by_key(|x| x.priority)
    }

    pub fn clear(&mut self) {
        for fragments in self.data.iter_mut() {
            fragments.clear();
        }
    }

    pub fn push(&mut self, x: usize, y: usize, fragment: Fragment) {
        let Some(fragments) = self.data.get_mut(x, y) else {
            return;
        };

        fragments.push(fragment);
    }
}

pub struct Fragment {
    pub priority: u8,
    pub glyph_idx: usize,
    pub fg1: Option<u32>,
    pub bg: Option<u32>,
    pub alpha: f32,
}

#[derive(Clone, Debug)]
pub enum GlyphAnimation {
    Static(char),
    RandomPool { glyphs: Vec<char>, change_rate: Option<f32>, last_change: f32 },
    Sequence { glyphs: Vec<char>, duration_per_glyph: f32 },
    TimedCurve { keyframes: Vec<(f32, char)> },
}

#[derive(Component)]
pub struct Particle {
    pub age: f32,
    pub max_age: f32,
    pub pos: Vec2,
    pub initial_pos: Vec2,
    
    // Animation curves
    pub glyph_animation: GlyphAnimation,
    pub color_curve: Option<ColorCurve>,
    pub bg_curve: Option<ColorCurve>,
    pub alpha_curve: Option<AlphaCurve>,
    pub velocity_curve: Option<VelocityCurve>,
    pub gravity: Vec2,
    
    // Current state (evaluated from curves)
    pub current_velocity: Vec2,
    pub current_glyph: char,
    pub current_color: u32,
    pub current_bg_color: u32,
    pub current_alpha: f32,
    
    pub priority: u8,
    pub target_pos: Option<Vec2>,
    pub max_distance: Option<f32>,
}

impl Particle {
    pub fn progress(&self) -> f32 {
        (self.age / self.max_age).clamp(0., 1.)
    }

    pub fn update_properties(&mut self, dt: f32) {
        let progress = self.progress();
        
        // Update curves
        if let Some(curve) = &self.color_curve {
            self.current_color = curve.evaluate(progress);
        }
        if let Some(curve) = &self.bg_curve {
            self.current_bg_color = curve.evaluate(progress);
        }
        if let Some(curve) = &self.alpha_curve {
            self.current_alpha = curve.evaluate(progress);
        }
        if let Some(curve) = &self.velocity_curve {
            self.current_velocity = curve.evaluate(progress);
        }
        
        // Update glyph animation
        self.update_glyph_animation(dt);
    }
    
    fn update_glyph_animation(&mut self, dt: f32) {
        match &mut self.glyph_animation {
            GlyphAnimation::Static(glyph) => {
                self.current_glyph = *glyph;
            }
            GlyphAnimation::RandomPool { glyphs, change_rate, last_change } => {
                if glyphs.is_empty() {
                    self.current_glyph = '*';
                    return;
                }
                
                if let Some(rate) = change_rate {
                    *last_change += dt;
                    if *last_change >= 1.0 / *rate {
                        use macroquad::rand::gen_range;
                        let index = gen_range(0, glyphs.len());
                        self.current_glyph = glyphs[index];
                        *last_change = 0.0;
                    }
                } else {
                    // Change every frame if no rate specified
                    use macroquad::rand::gen_range;
                    let index = gen_range(0, glyphs.len());
                    self.current_glyph = glyphs[index];
                }
            }
            GlyphAnimation::Sequence { glyphs, duration_per_glyph } => {
                if glyphs.is_empty() {
                    self.current_glyph = '*';
                    return;
                }
                
                let total_duration = *duration_per_glyph * glyphs.len() as f32;
                let cycle_time = self.age % total_duration;
                let index = (cycle_time / *duration_per_glyph) as usize;
                let clamped_index = index.min(glyphs.len() - 1);
                self.current_glyph = glyphs[clamped_index];
            }
            GlyphAnimation::TimedCurve { keyframes } => {
                if keyframes.is_empty() {
                    self.current_glyph = '*';
                    return;
                }
                
                // Find the appropriate keyframe for current time
                for (time, glyph) in keyframes.iter() {
                    if self.age <= *time {
                        self.current_glyph = *glyph;
                        return;
                    }
                }
                
                // If past all keyframes, use the last one
                self.current_glyph = keyframes.last().unwrap().1;
            }
        }
    }
    
    // Keep these for compatibility with existing rendering system
    pub fn current_glyph(&self) -> char {
        self.current_glyph
    }
    
    pub fn current_alpha(&self) -> f32 {
        self.current_alpha
    }
    
    pub fn current_fg1(&self) -> Option<u32> {
        Some(self.current_color)
    }
    
    pub fn current_bg(&self) -> Option<u32> {
        if self.bg_curve.is_some() {
            Some(self.current_bg_color)
        } else {
            None
        }
    }
}

#[derive(Resource)]
pub struct ParticleGlyphPool {
    pub free_glyphs: Vec<Entity>,
    pub used_glyphs: Vec<Entity>,
}

impl FromWorld for ParticleGlyphPool {
    fn from_world(world: &mut World) -> Self {
        let glyphs = (0..500)
            .map(|_| {
                world
                    .spawn((
                        Glyph::new(0, Palette::Black, Palette::Black)
                            .layer(Layer::Particles)
                            .texture(GlyphTextureId::BodyFont),
                        Position::new_f32(0., 0., 0.),
                        ParticleGlyph,
                        Visibility::Hidden,
                    ))
                    .id()
            })
            .collect();

        Self {
            free_glyphs: glyphs,
            used_glyphs: vec![],
        }
    }
}

#[derive(Component)]
pub struct ParticleGlyph;

#[derive(Component)]
pub struct ParticleSpawner {
    pub timer: f32,
    pub spawn_rate: f32,
    pub burst_count: Option<u32>,
    pub position: Vec2,
    
    // Animation configuration
    pub glyph_animation: GlyphAnimation,
    pub color_curve: Option<ColorCurve>,
    pub bg_curve: Option<ColorCurve>,
    pub alpha_curve: Option<AlphaCurve>,
    pub velocity_curve: Option<VelocityCurve>,
    
    // Spawn area
    pub spawn_area: SpawnArea,
    
    // Physics
    pub gravity: Vec2,
    pub priority: u8,
    
    // Lifecycle
    pub lifetime_min: f32,
    pub lifetime_max: f32,
    pub spawn_delay: f32,
}

impl ParticleSpawner {
    pub fn new(position: Vec2) -> Self {
        Self {
            timer: 0.0,
            spawn_rate: 10.0,
            burst_count: None,
            position,
            glyph_animation: GlyphAnimation::Static('*'),
            color_curve: Some(ColorCurve::Constant(0xFFFFFF)),
            bg_curve: None,
            alpha_curve: Some(AlphaCurve::Linear { start: 1.0, end: 0.0 }),
            velocity_curve: Some(VelocityCurve::Constant(Vec2::new(0.0, -1.0))),
            spawn_area: SpawnArea::Point,
            gravity: Vec2::new(0.0, 2.0),
            priority: 128,
            lifetime_min: 1.0,
            lifetime_max: 3.0,
            spawn_delay: 0.0,
        }
    }

    pub fn burst(mut self, count: u32) -> Self {
        self.burst_count = Some(count);
        self
    }

    pub fn spawn_rate(mut self, rate: f32) -> Self {
        self.spawn_rate = rate;
        self
    }

    pub fn glyph_animation(mut self, animation: GlyphAnimation) -> Self {
        self.glyph_animation = animation;
        self
    }

    pub fn color_curve(mut self, curve: ColorCurve) -> Self {
        self.color_curve = Some(curve);
        self
    }

    pub fn bg_curve(mut self, curve: ColorCurve) -> Self {
        self.bg_curve = Some(curve);
        self
    }

    pub fn alpha_curve(mut self, curve: AlphaCurve) -> Self {
        self.alpha_curve = Some(curve);
        self
    }

    pub fn velocity_curve(mut self, curve: VelocityCurve) -> Self {
        self.velocity_curve = Some(curve);
        self
    }

    pub fn spawn_area(mut self, area: SpawnArea) -> Self {
        self.spawn_area = area;
        self
    }

    pub fn gravity(mut self, gravity: Vec2) -> Self {
        self.gravity = gravity;
        self
    }

    pub fn priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    pub fn lifetime_range(mut self, range: std::ops::Range<f32>) -> Self {
        self.lifetime_min = range.start;
        self.lifetime_max = range.end;
        self
    }

    pub fn delay(mut self, delay: f32) -> Self {
        self.spawn_delay = delay;
        self
    }
    
    pub fn spawn_world(self, world: &mut World) -> Entity {
        world.spawn(self).id()
    }
}

#[derive(Component)]
pub struct ParticleTrail {
    pub last_spawn_time: f32,
    pub spawn_rate: f32,
    pub trail_spawner: ParticleSpawner,
}

impl ParticleTrail {
    pub fn new(spawn_rate: f32, trail_spawner: ParticleSpawner) -> Self {
        Self {
            last_spawn_time: 0.0,
            spawn_rate,
            trail_spawner,
        }
    }
}
