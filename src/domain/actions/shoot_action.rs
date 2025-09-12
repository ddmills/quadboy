use bevy_ecs::prelude::*;

use crate::{
    common::Rand,
    domain::{
        Destructible, Energy, EnergyActionType, EquipmentSlots, Health, HitBlink, MaterialType,
        RangedWeapon, Zone, get_energy_cost,
        systems::destruction_system::{DestructionCause, EntityDestroyedEvent},
    },
    engine::{Audio, StableIdRegistry},
    rendering::{
        Glyph, Position, spawn_bullet_trail_in_world, spawn_destruction_particles_in_world,
    },
};

pub struct ShootAction {
    pub shooter_entity: Entity,
    pub target_pos: (usize, usize, usize),
}

fn apply_hit_blink(world: &mut World, target_entity: Entity) {
    if let Some(mut existing_hit_blink) = world.get_mut::<HitBlink>(target_entity) {
        // Reset duration on existing hit blink
        existing_hit_blink.duration_remaining = 0.05;
    } else if world.get::<Glyph>(target_entity).is_some() {
        // Create new hit blink
        let hit_blink = HitBlink::attacked();
        world.entity_mut(target_entity).insert(hit_blink);
    }
}

impl Command for ShootAction {
    fn apply(self, world: &mut World) {
        let shooter_pos = world
            .get::<Position>(self.shooter_entity)
            .map(|p| p.world());

        // Get the equipped ranged weapon (if any)
        let weapon_data = {
            let Some(registry) = world.get_resource::<StableIdRegistry>() else {
                return;
            };

            let Some(equipment) = world.get::<EquipmentSlots>(self.shooter_entity) else {
                return; // No equipment slots, can't have weapon
            };

            // Check for equipped weapon in MainHand
            let weapon_id = equipment.get_equipped_item(crate::domain::EquipmentSlot::MainHand);

            if let Some(weapon_id) = weapon_id {
                if let Some(weapon_entity) = registry.get_entity(weapon_id) {
                    world.get::<RangedWeapon>(weapon_entity).map(|weapon| {
                        (
                            weapon.damage,
                            weapon.can_damage.clone(),
                            weapon.range,
                            weapon.shoot_audio,
                        )
                    })
                } else {
                    None
                }
            } else {
                None
            }
        };

        let Some((damage, can_damage, range, shoot_audio)) = weapon_data else {
            // No ranged weapon equipped
            return;
        };

        // Check if target is within range
        if let Some((sx, sy, _sz)) = shooter_pos {
            let distance = ((self.target_pos.0 as i32 - sx as i32).abs()
                + (self.target_pos.1 as i32 - sy as i32).abs()) as usize;

            if distance > range {
                // Target out of range
                return;
            }
        }

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

        let Some(audio) = world.get_resource::<Audio>() else {
            return;
        };

        audio.play(shoot_audio, 0.1);

        if let Some(shooter_pos) = shooter_pos {
            world.resource_scope(|world, mut rand: Mut<crate::common::Rand>| {
                spawn_bullet_trail_in_world(world, shooter_pos, self.target_pos, 60.0, &mut rand);
            });
        }

        if targets.is_empty() {
            // Consume energy even if no target hit (shot fired)
            if let Some(mut energy) = world.get_mut::<Energy>(self.shooter_entity) {
                let cost = get_energy_cost(EnergyActionType::Shoot);
                energy.consume_energy(cost);
            }
            return;
        }

        // Process shot on each target at position
        for &target_entity in targets.iter() {
            let mut should_apply_hit_blink = false;

            if let Some(mut health) = world.get_mut::<Health>(target_entity) {
                if can_damage.contains(&MaterialType::Flesh) {
                    health.take_damage(damage);
                    should_apply_hit_blink = true;
                    let is_dead = health.is_dead();

                    // Play hit audio for flesh target
                    if let Some(audio_collection) = MaterialType::Flesh.hit_audio_collection() {
                        world.resource_scope(|world, audio_registry: Mut<Audio>| {
                            if let Some(mut rand) = world.get_resource_mut::<Rand>() {
                                audio_registry.play_random_from_collection(
                                    audio_collection,
                                    &mut rand,
                                    0.5,
                                );
                            }
                        });
                    }

                    if is_dead {
                        let position_data = world.get::<Position>(target_entity).map(|p| p.world());

                        if let Some(position_coords) = position_data {
                            // Play destroy audio for flesh target
                            if let Some(audio_collection) =
                                MaterialType::Flesh.destroy_audio_collection()
                            {
                                world.resource_scope(|world, audio_registry: Mut<Audio>| {
                                    if let Some(mut rand) = world.get_resource_mut::<Rand>() {
                                        audio_registry.play_random_from_collection(
                                            audio_collection,
                                            &mut rand,
                                            0.5,
                                        );
                                    }
                                });
                            }

                            let event = EntityDestroyedEvent::with_material_type(
                                target_entity,
                                position_coords,
                                DestructionCause::Attack,
                                crate::domain::MaterialType::Flesh,
                            );
                            world.send_event(event);
                            spawn_destruction_particles_in_world(
                                world,
                                position_coords,
                                crate::domain::MaterialType::Flesh,
                            );
                        }
                    }
                }
            }
            // Check if target has Destructible (object)
            else if let Some(mut destructible) = world.get_mut::<Destructible>(target_entity) {
                // Check if weapon can damage this material type
                if can_damage.contains(&destructible.material_type) {
                    let material_type = destructible.material_type;
                    destructible.take_damage(damage);
                    should_apply_hit_blink = true;
                    let is_destroyed = destructible.is_destroyed();

                    // Play hit audio
                    if let Some(audio_collection) = material_type.hit_audio_collection() {
                        world.resource_scope(|world, audio_registry: Mut<Audio>| {
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
                    if is_destroyed && let Some(position) = world.get::<Position>(target_entity) {
                        let position_coords = position.world();
                        let event = EntityDestroyedEvent::with_material_type(
                            target_entity,
                            position_coords,
                            DestructionCause::Attack,
                            material_type,
                        );
                        world.send_event(event);
                        spawn_destruction_particles_in_world(world, position_coords, material_type);
                    }
                }
            }

            if should_apply_hit_blink {
                apply_hit_blink(world, target_entity);
            }
        }

        // Consume energy
        if let Some(mut energy) = world.get_mut::<Energy>(self.shooter_entity) {
            let cost = get_energy_cost(EnergyActionType::Shoot);
            energy.consume_energy(cost);
        }
    }
}
