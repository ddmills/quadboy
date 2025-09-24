use bevy_ecs::prelude::*;
use macroquad::{math::Vec2, prelude::trace};

use crate::{
    cfg::ZONE_SIZE,
    domain::{
        Destructible, Health, PlayerPosition, Zone,
        systems::destruction_system::EntityDestroyedEvent,
    },
    engine::{Audio, AudioKey, Clock},
    rendering::{
        AlphaCurve, ColorCurve, Distribution, GlyphAnimation, ParticleSpawner, Position, SpawnArea,
        world_to_zone_idx, world_to_zone_local, world_to_zone_local_f32,
    },
    states::CleanupStatePlay,
};

#[derive(Event)]
pub struct ExplosionEvent {
    pub position: (usize, usize, usize),
    pub radius: usize,
    pub damage: i32,
    pub falloff_rate: f32,
    pub source_entity: Option<Entity>,
    pub destroys_terrain: bool,
    pub audio: Option<AudioKey>,
}

impl ExplosionEvent {
    pub fn new(
        position: (usize, usize, usize),
        radius: usize,
        damage: i32,
        falloff_rate: f32,
        audio: Option<AudioKey>,
    ) -> Self {
        Self {
            position,
            radius,
            damage,
            falloff_rate,
            source_entity: None,
            destroys_terrain: true,
            audio,
        }
    }

    pub fn with_source(mut self, source_entity: Entity) -> Self {
        self.source_entity = Some(source_entity);
        self
    }

    pub fn with_audio(mut self, audio: AudioKey) -> Self {
        self.audio = Some(audio);
        self
    }

    pub fn calculate_damage_at_distance(&self, distance: f32) -> i32 {
        if distance <= 0.0 {
            return self.damage;
        }

        let damage_multiplier = (1.0 - (distance * self.falloff_rate)).max(0.0);
        (self.damage as f32 * damage_multiplier) as i32
    }
}

pub fn explosion_system(
    mut cmds: Commands,
    mut e_explosion: EventReader<ExplosionEvent>,
    mut e_entity_destroyed: EventWriter<EntityDestroyedEvent>,
    q_zones: Query<&Zone>,
    mut q_health: Query<&mut Health>,
    mut q_destructible: Query<&mut Destructible>,
    q_positions: Query<&Position>,
    clock: Res<Clock>,
    audio: Option<Res<Audio>>,
    player_pos: Option<Res<PlayerPosition>>,
) {
    for explosion in e_explosion.read() {
        trace!("Explosion event?");
        // Play explosion sound if available
        if let Some(audio_key) = explosion.audio
            && let Some(audio) = &audio
            && let Some(player_pos) = &player_pos
        {
            audio.play_at_position(audio_key, 0.5, explosion.position, player_pos);
        }

        // Spawn explosion particle effects
        let local_pos = world_to_zone_local_f32(
            explosion.position.0 as f32 + 0.5,
            explosion.position.1 as f32 + 0.5,
        );
        let pos = Vec2::new(local_pos.0, local_pos.1);
        let radius_f = explosion.radius as f32;
        let scale = (radius_f / 3.0).max(0.8); // Higher minimum scale

        // 1. MASSIVE CENTRAL FLASH - Multiple overlapping flashes
        cmds.spawn((
            ParticleSpawner::new(pos)
                .glyph_animation(GlyphAnimation::Sequence {
                    glyphs: vec!['█', '◉', '◎', '○', '◦', ' '],
                    duration_per_glyph: 0.08,
                })
                .color_curve(ColorCurve::Linear {
                    values: vec![0xFFFFFF, 0xFFFFFF, 0xFFDD00, 0xFF8800],
                })
                .bg_curve(ColorCurve::Linear {
                    values: vec![0xFFFFFF, 0xFFDD00, 0xFF4400, 0xFF0000, 0x880000],
                })
                .alpha_curve(AlphaCurve::EaseOut {
                    values: vec![1.0, 0.0],
                })
                .priority(230)
                .lifetime_range(0.4..0.6)
                .burst(1),
            CleanupStatePlay,
        ));

        // 2. EXPANDING SHOCKWAVE RING
        let shockwave_count = (12.0 * scale) as u32;
        cmds.spawn((
            ParticleSpawner::new(pos)
                .glyph_animation(GlyphAnimation::Sequence {
                    glyphs: vec!['█', '▓', '▒', '░', '·', ' '],
                    duration_per_glyph: 0.06,
                })
                .color_curve(ColorCurve::Constant(0xFFFFFF))
                .bg_curve(ColorCurve::EaseOut {
                    values: vec![0xFFFFFF, 0xFFDD00, 0xFF8800, 0xFF4400, 0x000000],
                })
                .alpha_curve(AlphaCurve::EaseOut {
                    values: vec![1.0, 0.0],
                })
                .spawn_area(SpawnArea::Circle {
                    radius: radius_f * 1.0,
                    distribution: Distribution::EdgeOnly,
                })
                .priority(225)
                .lifetime_range(0.3..0.5)
                .burst(shockwave_count),
            CleanupStatePlay,
        ));

        // 3. INTENSE FIRE BURST - Much more particles
        let fire_count = (80.0 * scale) as u32;
        cmds.spawn((
            ParticleSpawner::new(pos)
                .glyph_animation(GlyphAnimation::RandomPool {
                    glyphs: vec!['*', '✦', '●', '◆', '○', '•', '◉', '▲'],
                    change_rate: Some(25.0),
                    last_change: 0.0,
                })
                .color_curve(ColorCurve::EaseOut {
                    values: vec![0xFFFFFF, 0xFFDD00, 0xFF8800, 0xFF4400, 0x880000],
                })
                .bg_curve(ColorCurve::EaseOut {
                    values: vec![0xFFDD00, 0xFF8800, 0xFF4400, 0x880000, 0x000000],
                })
                .alpha_curve(AlphaCurve::EaseOut {
                    values: vec![1.0, 0.0],
                })
                .spawn_area(SpawnArea::Circle {
                    radius: radius_f * 0.8,
                    distribution: Distribution::Uniform,
                })
                .priority(220)
                .lifetime_range(0.8..2.0)
                .burst(fire_count),
            CleanupStatePlay,
        ));

        // 4. FLYING DEBRIS - Fast moving fragments
        let debris_count = (60.0 * scale) as u32;
        cmds.spawn((
            ParticleSpawner::new(pos)
                .glyph_animation(GlyphAnimation::RandomPool {
                    glyphs: vec!['*', '·', ',', '`', '\'', '.', '"', '^'],
                    change_rate: Some(30.0),
                    last_change: 0.0,
                })
                .color_curve(ColorCurve::EaseOut {
                    values: vec![0xFFFFFF, 0xFFDD00, 0xFF8800, 0x888888, 0x444444],
                })
                .bg_curve(ColorCurve::EaseOut {
                    values: vec![0xFF8800, 0xFF4400, 0x880000, 0x440000, 0x000000],
                })
                .alpha_curve(AlphaCurve::EaseOut {
                    values: vec![1.0, 0.0],
                })
                .spawn_area(SpawnArea::Circle {
                    radius: radius_f * 1.2,
                    distribution: Distribution::Uniform,
                })
                .priority(210)
                .lifetime_range(1.0..2.5)
                .burst(debris_count),
            CleanupStatePlay,
        ));

        // 5. MASSIVE SMOKE CLOUDS - Much denser
        let smoke_count = (50.0 * scale) as u32;
        cmds.spawn((
            ParticleSpawner::new(pos)
                .glyph_animation(GlyphAnimation::RandomPool {
                    glyphs: vec![' ', '░', '▒', '▓', '#', '%'],
                    change_rate: Some(12.0),
                    last_change: 0.0,
                })
                .color_curve(ColorCurve::EaseOut {
                    values: vec![0x888888, 0x666666, 0x444444],
                })
                .bg_curve(ColorCurve::EaseOut {
                    values: vec![0xAAAAAA, 0x888888, 0x666666, 0x444444, 0x000000],
                })
                .alpha_curve(AlphaCurve::EaseOut {
                    values: vec![0.9, 0.0],
                })
                .spawn_area(SpawnArea::Circle {
                    radius: radius_f * 1.0,
                    distribution: Distribution::Gaussian,
                })
                .priority(100)
                .lifetime_range(2.5..5.0)
                .burst(smoke_count),
            CleanupStatePlay,
        ));

        // 6. SECONDARY SPARKS - Extra sparkly effects
        let spark_count = (40.0 * scale) as u32;
        cmds.spawn((
            ParticleSpawner::new(pos)
                .glyph_animation(GlyphAnimation::RandomPool {
                    glyphs: vec!['✦', '✧', '◆', '◇', '❋', '✳', '※'],
                    change_rate: Some(20.0),
                    last_change: 0.0,
                })
                .color_curve(ColorCurve::EaseOut {
                    values: vec![0xFFFFFF, 0xFFDD00, 0xFF8800, 0xFF4400],
                })
                .bg_curve(ColorCurve::EaseOut {
                    values: vec![0xFFDD00, 0xFF8800, 0xFF4400, 0x000000],
                })
                .alpha_curve(AlphaCurve::EaseOut {
                    values: vec![1.0, 0.0],
                })
                .spawn_area(SpawnArea::Circle {
                    radius: radius_f * 1.0,
                    distribution: Distribution::Gaussian,
                })
                .priority(215)
                .lifetime_range(0.6..1.8)
                .burst(spark_count),
            CleanupStatePlay,
        ));

        let explosion_zone_idx = world_to_zone_idx(
            explosion.position.0,
            explosion.position.1,
            explosion.position.2,
        );

        // Find the zone containing the explosion
        let Some(zone) = q_zones.iter().find(|z| z.idx == explosion_zone_idx) else {
            continue;
        };

        let (explosion_local_x, explosion_local_y) =
            world_to_zone_local(explosion.position.0, explosion.position.1);

        // Check all tiles within the explosion radius
        let radius = explosion.radius as i32;
        for dx in -radius..=radius {
            for dy in -radius..=radius {
                let target_local_x = explosion_local_x as i32 + dx;
                let target_local_y = explosion_local_y as i32 + dy;

                // Check bounds
                if target_local_x < 0
                    || target_local_x >= ZONE_SIZE.0 as i32
                    || target_local_y < 0
                    || target_local_y >= ZONE_SIZE.1 as i32
                {
                    continue;
                }

                let target_local_x = target_local_x as usize;
                let target_local_y = target_local_y as usize;

                // Calculate distance and damage
                let distance = ((dx * dx + dy * dy) as f32).sqrt();
                if distance > explosion.radius as f32 {
                    continue;
                }

                let damage = explosion.calculate_damage_at_distance(distance);
                if damage <= 0 {
                    continue;
                }

                // Get entities at this position
                if let Some(entities_at_pos) = zone.entities.get(target_local_x, target_local_y) {
                    for &entity in entities_at_pos {
                        // Apply damage to entities with Health
                        if let Ok(mut health) = q_health.get_mut(entity) {
                            health.take_damage(damage, clock.get_tick());

                            if health.is_dead()
                                && let Ok(pos) = q_positions.get(entity)
                            {
                                e_entity_destroyed.write(EntityDestroyedEvent::environmental(
                                    entity,
                                    pos.world(),
                                    None,
                                ));
                            }
                        }

                        // Apply damage to destructible objects if explosion destroys terrain
                        if explosion.destroys_terrain
                            && let Ok(mut destructible) = q_destructible.get_mut(entity)
                        {
                            destructible.take_damage(damage);

                            if destructible.is_destroyed()
                                && let Ok(pos) = q_positions.get(entity)
                            {
                                e_entity_destroyed.write(EntityDestroyedEvent::environmental(
                                    entity,
                                    pos.world(),
                                    Some(destructible.material_type),
                                ));
                            }
                        }
                    }
                }
            }
        }
    }
}
