use bevy_ecs::prelude::*;
use macroquad::prelude::trace;

use crate::{
    domain::{
        ExplosionEvent, ExplosiveProperties, Fuse, HitBlink,
        systems::destruction_system::EntityDestroyedEvent,
    },
    engine::Clock,
    rendering::Position,
};

pub fn fuse_system(
    mut e_explosion: EventWriter<ExplosionEvent>,
    mut e_entity_destroyed: EventWriter<EntityDestroyedEvent>,
    mut q_fused: Query<(
        Entity,
        &mut Fuse,
        &Position,
        Option<&ExplosiveProperties>,
        Option<&mut HitBlink>,
    )>,
    clock: Res<Clock>,
) {
    if !clock.has_tick_delta() {
        return;
    }

    let tick_delta = clock.get_tick_delta() as i32;

    let mut to_explode = Vec::new();

    for (entity, mut fuse, position, explosive_props, hit_blink) in q_fused.iter_mut() {
        fuse.tick_down(tick_delta);

        // Update blink rate based on remaining ticks
        if let Some(mut hit_blink) = hit_blink {
            let new_blink_rate = if fuse.remaining_ticks > 200 {
                2.0 // slow blink
            } else if fuse.remaining_ticks > 100 {
                4.0 // medium blink
            } else if fuse.remaining_ticks > 50 {
                8.0 // fast blink
            } else {
                16.0 // very fast blink
            };

            hit_blink.blink_rate = Some(new_blink_rate);
        }

        if fuse.is_expired() {
            // Schedule for explosion
            let audio = explosive_props.and_then(|props| props.explosion_audio);
            to_explode.push((
                entity,
                position.world(),
                fuse.explosion_radius,
                fuse.explosion_damage,
                audio,
            ));
        }
    }

    // Process explosions
    for (entity, position, radius, damage, audio) in to_explode {
        // Send explosion event
        e_explosion
            .write(ExplosionEvent::new(position, radius, damage, 0.3, audio).with_source(entity));

        // Send entity destroyed event to properly clean up from zones
        e_entity_destroyed.write(EntityDestroyedEvent::environmental(entity, position, None));
    }
}
