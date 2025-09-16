use bevy_ecs::prelude::*;

use crate::{domain::HitBlink, engine::Time, rendering::Glyph, tracy_plot, tracy_span};

pub fn hit_blink_system(
    mut cmds: Commands,
    mut q_hit_blink: Query<(Entity, &mut HitBlink, &mut Glyph)>,
    time: Res<Time>,
) {
    tracy_span!("hit_blink_system");

    let hit_blink_count = q_hit_blink.iter().count() as f64;
    tracy_plot!("Hit Blink Entities", hit_blink_count);

    let mut entities_to_remove = Vec::new();

    for (entity, mut hit_blink, mut glyph) in q_hit_blink.iter_mut() {
        if hit_blink.duration_remaining > 0.0 {
            hit_blink.duration_remaining -= time.dt;
            glyph.outline_override = Some(hit_blink.color);
        } else {
            glyph.outline_override = None;
            entities_to_remove.push(entity);
        }
    }

    for entity in entities_to_remove {
        cmds.entity(entity).remove::<HitBlink>();
    }
}
