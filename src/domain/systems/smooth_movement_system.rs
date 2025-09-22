use bevy_ecs::prelude::*;
use quadboy_macros::profiled_system;

use crate::{domain::SmoothMovement, engine::Time, rendering::Glyph};

#[profiled_system]
pub fn smooth_movement_system(
    mut cmds: Commands,
    mut q_smooth_movement: Query<(Entity, &mut SmoothMovement, &mut Glyph)>,
    time: Res<Time>,
) {

    let mut entities_to_remove = Vec::new();

    for (entity, mut smooth_movement, mut glyph) in q_smooth_movement.iter_mut() {
        if smooth_movement.duration_remaining > 0.0 {
            smooth_movement.duration_remaining -= time.dt;

            // Calculate animation progress (0.0 = start, 1.0 = end)
            let progress =
                1.0 - (smooth_movement.duration_remaining / smooth_movement.total_duration);

            // Use smoothstep for smooth interpolation
            let t_smooth = progress * progress * (3.0 - 2.0 * progress);

            // Calculate current offset from end position back to start position
            let start_offset_x = smooth_movement.start_position.0 - smooth_movement.end_position.0;
            let start_offset_y = smooth_movement.start_position.1 - smooth_movement.end_position.1;

            // Interpolate from start_offset to (0, 0)
            let offset_x = start_offset_x * (1.0 - t_smooth);
            let offset_y = start_offset_y * (1.0 - t_smooth);

            glyph.position_offset = Some((offset_x, offset_y));
        } else {
            glyph.position_offset = None;
            entities_to_remove.push(entity);
        }
    }

    for entity in entities_to_remove {
        cmds.entity(entity).remove::<SmoothMovement>();
    }
}
