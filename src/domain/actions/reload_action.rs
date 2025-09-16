use bevy_ecs::prelude::*;

use crate::{
    domain::{
        Energy, EnergyActionType, EquipmentSlot, EquipmentSlots, Weapon, WeaponType,
        get_base_energy_cost,
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

        let (clip_size, reload_audio, energy_cost) = {
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

            if weapon.current_ammo == Some(clip_size) {
                return;
            }

            let energy_cost = weapon
                .base_reload_cost
                .unwrap_or_else(|| get_base_energy_cost(EnergyActionType::Reload));

            (clip_size, weapon.reload_audio, energy_cost)
        };

        if let Some(mut weapon) = world.get_mut::<Weapon>(weapon_entity) {
            weapon.current_ammo = Some(clip_size);
        }

        if let Some(reload_audio) = reload_audio
            && let Some(audio) = world.get_resource::<Audio>() {
                audio.play(reload_audio, 0.3);
            }

        if let Some(mut energy) = world.get_mut::<Energy>(self.entity) {
            energy.consume_energy(energy_cost);
        }
    }
}
