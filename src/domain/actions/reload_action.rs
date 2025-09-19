use bevy_ecs::prelude::*;

use crate::{
    domain::{
        Energy, EnergyActionType, EquipmentSlot, EquipmentSlots, StatType, Stats, Weapon,
        WeaponType, get_energy_cost,
    },
    engine::{Audio, StableIdRegistry},
};

pub struct ReloadAction {
    pub entity: Entity,
}

impl Command for ReloadAction {
    fn apply(self, world: &mut World) {
        let Some(registry) = world.get_resource::<StableIdRegistry>() else {
            return;
        };

        let Some(equipment) = world.get::<EquipmentSlots>(self.entity) else {
            return;
        };

        let weapon_id = equipment.get_equipped_item(EquipmentSlot::MainHand);

        let Some(weapon_id) = weapon_id else {
            return;
        };

        let Some(weapon_entity) = registry.get_entity(weapon_id) else {
            return;
        };

        let (clip_size, current_ammo, reload_audio, reload_complete_audio, energy_cost) = {
            let Some(weapon) = world.get::<Weapon>(weapon_entity) else {
                return;
            };

            // Only ranged weapons can be reloaded
            if weapon.weapon_type != WeaponType::Ranged {
                return;
            }

            let Some(clip_size) = weapon.clip_size else {
                return;
            };

            let current_ammo = weapon.current_ammo.unwrap_or(0);

            // Can't reload if already at max capacity
            if current_ammo >= clip_size {
                return;
            }

            // Calculate energy cost using reload speed stat
            let base_cost = weapon.base_reload_cost.unwrap_or_else(|| 50); // New lower base cost

            let stats = world.get::<Stats>(self.entity);
            let energy_cost = if let Some(stats) = stats {
                let reload_speed = stats.get_stat(StatType::ReloadSpeed);
                (base_cost - (reload_speed * 2)).max(1)
            } else {
                base_cost
            };

            (
                clip_size,
                current_ammo,
                weapon.reload_audio,
                weapon.reload_complete_audio,
                energy_cost,
            )
        };

        // Reload one bullet at a time
        let new_ammo = current_ammo + 1;
        if let Some(mut weapon) = world.get_mut::<Weapon>(weapon_entity) {
            weapon.current_ammo = Some(new_ammo);
        }

        if let Some(mut audio) = world.get_resource_mut::<Audio>() {
            if let Some(reload_audio_key) = reload_audio {
                audio.play(reload_audio_key, 0.4);
            }

            if let Some(reload_complete_audio_key) = reload_complete_audio
                && new_ammo >= clip_size
            {
                audio.play_delayed(reload_complete_audio_key, 0.5, 0.4);
            }
        }

        if let Some(mut energy) = world.get_mut::<Energy>(self.entity) {
            energy.consume_energy(energy_cost);
        }
    }
}
