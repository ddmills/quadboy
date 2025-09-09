use bevy_ecs::prelude::*;

use crate::{domain::BumpAttack, engine::Time, rendering::Glyph};

pub fn bump_attack_system(
    mut cmds: Commands,
    mut q_bump_attack: Query<(Entity, &mut BumpAttack, &mut Glyph)>,
    time: Res<Time>,
) {
    let mut entities_to_remove = Vec::new();

    for (entity, mut bump_attack, mut glyph) in q_bump_attack.iter_mut() {
        if bump_attack.duration_remaining > 0.0 {
            bump_attack.duration_remaining -= time.dt;

            // Calculate animation progress (0.0 = start, 1.0 = end)
            let progress = 1.0 - (bump_attack.duration_remaining / bump_attack.total_duration);

            // Use quadratic ease-out for smooth animation
            let ease_factor = 1.0 - (1.0 - progress).powi(2);
            let offset_factor = 1.0 - ease_factor; // Start at 1.0, end at 0.0

            // Maximum bump distance (in tiles)
            const MAX_BUMP_DISTANCE: f32 = 0.3;

            let offset_x = bump_attack.direction.0 * offset_factor * MAX_BUMP_DISTANCE;
            let offset_y = bump_attack.direction.1 * offset_factor * MAX_BUMP_DISTANCE;

            glyph.position_offset = Some((offset_x, offset_y));
        } else {
            glyph.position_offset = None;
            entities_to_remove.push(entity);
        }
    }

    for entity in entities_to_remove {
        cmds.entity(entity).remove::<BumpAttack>();
    }
}
