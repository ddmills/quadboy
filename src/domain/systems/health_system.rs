use bevy_ecs::prelude::*;

use crate::{
    domain::{Health, Level, StatType, Stats},
    tracy_span,
};

/// Update health and armor for entities that have Level and Stats components
/// This system clamps current HP and armor to new maximums when Level or Stats change
pub fn update_health_system(
    mut q_health: Query<(&mut Health, &Level, &Stats), Or<(Changed<Level>, Changed<Stats>)>>,
) {
    tracy_span!("update_health_system");
    for (mut health, level, stats) in q_health.iter_mut() {
        let max_hp = Health::get_max_hp(level, stats);
        let max_armor = stats.get_stat(StatType::Armor);

        // If this is a new entity with i32::MAX current HP, set it to full health
        if health.current == i32::MAX {
            health.current = max_hp;
        } else {
            // Clamp current HP to new maximum (prevents over-healing on level up)
            health.clamp_to_max(max_hp);
        }

        // If this is a new entity with i32::MAX current armor, set it to full armor
        if health.current_armor == i32::MAX {
            health.current_armor = max_armor;
        } else {
            // Clamp current armor to new maximum
            health.clamp_armor_to_max(max_armor);
        }
    }
}
