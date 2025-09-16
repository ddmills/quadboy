use crate::{
    domain::{
        AttributePoints, GameFormulas, Health, Level, Stats,
        systems::destruction_system::{DestructionCause, EntityDestroyedEvent},
    },
    rendering::{Position, spawn_level_up_celebration},
    tracy_plot, tracy_span,
};
use bevy_ecs::prelude::*;

#[derive(Resource, Default)]
pub struct LevelUpParticleQueue {
    pub requests: Vec<(usize, usize, usize)>,
}

#[derive(Event)]
pub struct XPGainEvent {
    pub recipient_entity: Entity,
    pub xp_amount: u32,
    pub source_entity: Entity,
    pub source_description: String,
}

#[derive(Event)]
pub struct LevelUpEvent {
    pub entity: Entity,
    pub old_level: u32,
    pub new_level: u32,
    pub levels_gained: u32,
}

/// Award XP to any entity with a Level component when they kill another leveled entity
pub fn award_xp_on_kill(
    mut e_entity_destroyed: EventReader<EntityDestroyedEvent>,
    mut e_xp_gain: EventWriter<XPGainEvent>,
    q_levels: Query<&Level>,
) {
    tracy_span!("award_xp_on_kill");

    let destruction_event_count = e_entity_destroyed.len() as f64;
    tracy_plot!("Entity Destruction Events", destruction_event_count);

    for destroyed_event in e_entity_destroyed.read() {
        // Only process attack-based deaths
        if let DestructionCause::Attack { attacker } = destroyed_event.cause {
            // Check if attacker has Level component (can gain XP)
            if let Ok(attacker_level) = q_levels.get(attacker) {
                // Check if destroyed entity had Level component (gives XP)
                if let Ok(victim_level) = q_levels.get(destroyed_event.entity) {
                    let xp_gained = GameFormulas::calculate_xp_gain(
                        attacker_level.current_level,
                        victim_level.current_level,
                    );

                    e_xp_gain.write(XPGainEvent {
                        recipient_entity: attacker,
                        xp_amount: xp_gained,
                        source_entity: destroyed_event.entity,
                        source_description: format!(
                            "Defeated Level {} Enemy",
                            victim_level.current_level
                        ),
                    });
                }
            }
        }
    }
}

/// Apply XP gains to entities with Level components
pub fn apply_xp_gain(
    mut e_xp_gain: EventReader<XPGainEvent>,
    mut e_level_up: EventWriter<LevelUpEvent>,
    mut q_levels: Query<&mut Level>,
    mut q_attribute_points: Query<&mut AttributePoints>,
) {
    tracy_span!("apply_xp_gain");

    let xp_event_count = e_xp_gain.len() as f64;
    tracy_plot!("XP Gain Events", xp_event_count);

    for xp_event in e_xp_gain.read() {
        if let Ok(mut level) = q_levels.get_mut(xp_event.recipient_entity) {
            let old_level = level.current_level;
            let leveled_up = level.add_xp(xp_event.xp_amount);

            if leveled_up {
                let levels_gained = level.current_level - old_level;

                // Emit level up event
                e_level_up.write(LevelUpEvent {
                    entity: xp_event.recipient_entity,
                    old_level,
                    new_level: level.current_level,
                    levels_gained,
                });

                // Grant additional attribute points for level up
                if let Ok(mut attribute_points) =
                    q_attribute_points.get_mut(xp_event.recipient_entity)
                {
                    attribute_points.available += levels_gained;
                    println!(
                        "Entity leveled up from {} to {}! (+{} XP, +{} attribute points)",
                        old_level, level.current_level, xp_event.xp_amount, levels_gained
                    );
                } else {
                    println!(
                        "Entity leveled up from {} to {}! (+{} XP)",
                        old_level, level.current_level, xp_event.xp_amount
                    );
                }
            } else {
                println!("Entity gained {} XP", xp_event.xp_amount);
            }
        }
    }
}

/// Handle level up events by restoring HP to full and spawning celebration particles
pub fn handle_level_up(
    mut e_level_up: EventReader<LevelUpEvent>,
    mut q_health_stats: Query<(&mut Health, &Level, &Stats)>,
    q_positions: Query<&Position>,
    mut particle_queue: ResMut<LevelUpParticleQueue>,
) {
    tracy_span!("handle_level_up");

    for level_up_event in e_level_up.read() {
        if let Ok((mut health, level, stats)) = q_health_stats.get_mut(level_up_event.entity) {
            // Restore HP to 100%
            let max_hp = Health::get_max_hp(level, stats);
            health.current = max_hp;

            // Queue particle effect at entity position
            if let Ok(position) = q_positions.get(level_up_event.entity) {
                let world_pos = position.world();
                particle_queue.requests.push(world_pos);
            }

            println!(
                "LEVEL UP! Level {} reached - HP restored to {}/{}!",
                level_up_event.new_level, health.current, max_hp
            );
        }
    }
}

/// Process level up particle requests by spawning particles (exclusive system)
pub fn process_level_up_particles(world: &mut World) {
    tracy_span!("process_level_up_particles");

    let mut requests = Vec::new();
    if let Some(mut particle_queue) = world.get_resource_mut::<LevelUpParticleQueue>() {
        requests.extend(particle_queue.requests.drain(..));
    }

    for world_pos in requests {
        spawn_level_up_celebration(world, world_pos);
    }
}
