use bevy_ecs::prelude::*;

use crate::{
    common::Rand,
    domain::{
        Destructible, Energy, EnergyActionType, EquipmentSlots, Health, MeleeWeapon, Zone,
        get_energy_cost,
        systems::destruction_system::{DestructionCause, EntityDestroyedEvent},
    },
    engine::{AudioRegistry, StableIdRegistry},
    rendering::Position,
};

pub struct AttackAction {
    pub attacker_entity: Entity,
    pub target_pos: (usize, usize, usize),
}

impl Command for AttackAction {
    fn apply(self, world: &mut World) {
        // Get the equipped weapon (if any)
        let weapon_damage = {
            let Some(registry) = world.get_resource::<StableIdRegistry>() else {
                return;
            };

            let Some(attacker_id) = registry.get_id(self.attacker_entity) else {
                return;
            };

            let Some(equipment) = world.get::<EquipmentSlots>(self.attacker_entity) else {
                return; // No equipment slots, can't have weapon
            };

            // Check for equipped weapon in MainHand
            let weapon_id = equipment.get_equipped_item(crate::domain::EquipmentSlot::MainHand);

            if let Some(weapon_id) = weapon_id {
                if let Some(weapon_entity) = registry.get_entity(weapon_id) {
                    if let Some(weapon) = world.get::<MeleeWeapon>(weapon_entity) {
                        Some((weapon.damage, weapon.can_damage.clone()))
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        };

        // Find target at position
        let targets = {
            let zone_idx = crate::rendering::world_to_zone_idx(
                self.target_pos.0,
                self.target_pos.1,
                self.target_pos.2,
            );
            let local = crate::rendering::world_to_zone_local(self.target_pos.0, self.target_pos.1);

            let mut found_targets = Vec::new();
            let mut zones = world.query::<&Zone>();
            for zone in zones.iter(world) {
                if zone.idx == zone_idx {
                    if let Some(entities) = zone.entities.get(local.0, local.1) {
                        found_targets.extend(entities.iter().copied());
                    }
                    break;
                }
            }
            found_targets
        };

        if targets.is_empty() {
            return;
        }

        // Process attack on each target at position
        for &target_entity in targets.iter() {
            // Check if target has Health (living creature)
            if let Some(mut health) = world.get_mut::<Health>(target_entity) {
                if let Some((damage, can_damage)) = &weapon_damage {
                    // Check if weapon can damage flesh
                    if can_damage.contains(&crate::domain::MaterialType::Flesh) {
                        health.take_damage(*damage);

                        // Check if target died
                        if health.is_dead() {
                            // Fire destruction event instead of handling directly
                            if let Some(position) = world.get::<Position>(target_entity) {
                                let event = EntityDestroyedEvent::new(
                                    target_entity,
                                    position.world(),
                                    DestructionCause::Attack,
                                );
                                world.send_event(event);
                            }
                        }
                    }
                }
            }
            // Check if target has Destructible (object)
            else if let Some(mut destructible) = world.get_mut::<Destructible>(target_entity) {
                if let Some((damage, can_damage)) = &weapon_damage {
                    // Check if weapon can damage this material type
                    if can_damage.contains(&destructible.material_type) {
                        let material_type = destructible.material_type;
                        destructible.take_damage(*damage);
                        let is_destroyed = destructible.is_destroyed();

                        // Drop the borrow before accessing resources
                        drop(destructible);

                        // Play hit audio
                        if let Some(audio_collection) = material_type.hit_audio_collection() {
                            world.resource_scope(|world, audio_registry: Mut<AudioRegistry>| {
                                if let Some(mut rand) = world.get_resource_mut::<Rand>() {
                                    audio_registry.play_random_from_collection(
                                        audio_collection,
                                        &mut rand,
                                        0.5,
                                    );
                                }
                            });
                        }

                        // Check if target was destroyed
                        if is_destroyed {
                            // Fire destruction event instead of handling directly
                            if let Some(position) = world.get::<Position>(target_entity) {
                                let event = EntityDestroyedEvent::new(
                                    target_entity,
                                    position.world(),
                                    DestructionCause::Attack,
                                );
                                world.send_event(event);
                            }
                        }
                    }
                }
            }
        }

        // Consume energy
        if let Some(mut energy) = world.get_mut::<Energy>(self.attacker_entity) {
            let cost = get_energy_cost(EnergyActionType::Move); // Same cost as movement for now
            energy.consume_energy(cost);
        }
    }
}
