use crate::{
    domain::{Health, StatType, Stats},
    engine::Clock,
};
use bevy_ecs::prelude::*;
use macroquad::prelude::trace;

pub fn armor_regen_system(mut q_health: Query<(&mut Health, &Stats)>, clock: Res<Clock>) {
    let current_tick = clock.current_tick();

    for (mut health, stats) in q_health.iter_mut() {
        let max_armor = stats.get_stat(StatType::Armor);

        if health.current_armor >= max_armor {
            continue;
        }

        let ticks_since_damage = current_tick.saturating_sub(health.last_damage_tick);

        if ticks_since_damage < 800 {
            continue;
        }

        let armor_regen_stat = stats.get_stat(StatType::ArmorRegen) as f32;
        let regen_per_tick = (1.0 + armor_regen_stat) / 5000.0;

        // Accumulate fractional progress (stored as u32 representing thousandths)
        let fractional_amount = clock.tick_delta() as f32 * regen_per_tick;
        let progress_to_add = (fractional_amount * 1000.0) as u32;

        health.armor_regen_progress += progress_to_add;

        // Apply full armor points when we've accumulated 1000 progress (1.0 armor)
        while health.armor_regen_progress >= 1000 {
            health.armor_regen_progress -= 1000;
            health.restore_armor(1, stats);

            if health.current_armor >= max_armor {
                break;
            }
        }
    }
}
