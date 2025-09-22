use bevy_ecs::prelude::*;
use quadboy_macros::profiled_system;

use crate::{domain::KnockbackAnimation, engine::Time, rendering::Glyph};

#[profiled_system]
pub fn knockback_animation_system(
    mut cmds: Commands,
    mut q_knockback: Query<(Entity, &mut KnockbackAnimation, &mut Glyph)>,
    time: Res<Time>,
) {

    let mut entities_to_remove = Vec::new();

    for (entity, mut knockback, mut glyph) in q_knockback.iter_mut() {
        if knockback.duration_remaining > 0.0 {
            knockback.duration_remaining -= time.dt;

            // Calculate animation progress (0.0 = start, 1.0 = end)
            let progress = 1.0 - (knockback.duration_remaining / knockback.total_duration);

            // Use smoothstep for smooth animation (same as original knockback)
            let t_smooth = progress * progress * (3.0 - 2.0 * progress);

            // Interpolate from start_offset to (0, 0)
            let offset_x = knockback.start_offset.0 * (1.0 - t_smooth);
            let offset_y = knockback.start_offset.1 * (1.0 - t_smooth);

            glyph.position_offset = Some((offset_x, offset_y));
        } else {
            // Animation complete - clear offset and mark for removal
            glyph.position_offset = None;
            entities_to_remove.push(entity);
        }
    }

    // Remove completed animation components
    for entity in entities_to_remove {
        cmds.entity(entity).remove::<KnockbackAnimation>();
    }
}
