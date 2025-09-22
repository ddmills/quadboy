use bevy_ecs::prelude::*;
use quadboy_macros::profiled_system;

use crate::{domain::HitBlink, engine::Time, rendering::Glyph};

#[profiled_system]
pub fn hit_blink_system(
    mut cmds: Commands,
    mut q_hit_blink: Query<(Entity, &mut HitBlink, &mut Glyph)>,
    time: Res<Time>,
) {
    let mut entities_to_remove = Vec::new();

    for (entity, mut hit_blink, mut glyph) in q_hit_blink.iter_mut() {
        if let Some(blink_rate) = hit_blink.blink_rate {
            // Continuous blinking mode
            hit_blink.time_since_last_toggle += time.dt;
            let toggle_interval = 1.0 / blink_rate / 2.0; // divide by 2 for on/off cycle

            if hit_blink.time_since_last_toggle >= toggle_interval {
                hit_blink.blink_on = !hit_blink.blink_on;
                hit_blink.time_since_last_toggle = 0.0;
            }

            glyph.outline_override = if hit_blink.blink_on {
                Some(hit_blink.color)
            } else {
                None
            };
        } else {
            // Legacy one-time flash mode
            if hit_blink.duration_remaining > 0.0 {
                hit_blink.duration_remaining -= time.dt;
                glyph.outline_override = Some(hit_blink.color);
            } else {
                glyph.outline_override = None;
                entities_to_remove.push(entity);
            }
        }
    }

    for entity in entities_to_remove {
        cmds.entity(entity).remove::<HitBlink>();
    }
}
