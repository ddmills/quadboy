use bevy_ecs::prelude::*;

use crate::{
    common::Rand,
    domain::{
        Destructible, Energy, EnergyActionType, EquipmentSlots, Health, HitBlink, MeleeWeapon, Zone,
        get_energy_cost,
        systems::destruction_system::{DestructionCause, EntityDestroyedEvent},
    },
    engine::{Audio, StableIdRegistry},
    rendering::{Glyph, Position},
};

pub struct AttackAction {
    pub attacker_entity: Entity,
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

impl Command for AttackAction {
    fn apply(self, world: &mut World) {
        // Get the equipped weapon (if any)
        let weapon_damage = {
            let Some(registry) = world.get_resource::<StableIdRegistry>() else {
                return;
            };

            let Some(equipment) = world.get::<EquipmentSlots>(self.attacker_entity) else {
                return; // No equipment slots, can't have weapon
            };

            // Check for equipped weapon in MainHand
            let weapon_id = equipment.get_equipped_item(crate::domain::EquipmentSlot::MainHand);

            if let Some(weapon_id) = weapon_id {
                if let Some(weapon_entity) = registry.get_entity(weapon_id) {
                    world
                        .get::<MeleeWeapon>(weapon_entity)
                        .map(|weapon| (weapon.damage, weapon.can_damage.clone()))
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
            let mut should_apply_hit_blink = false;
            
            if let Some(mut health) = world.get_mut::<Health>(target_entity) {
                if let Some((damage, can_damage)) = &weapon_damage
                    && can_damage.contains(&crate::domain::MaterialType::Flesh)
                {
                    health.take_damage(*damage);
                    should_apply_hit_blink = true;

                    if health.is_dead()
                        && let Some(position) = world.get::<Position>(target_entity)
                    {
                        let event = EntityDestroyedEvent::new(
                            target_entity,
                            position.world(),
                            DestructionCause::Attack,
                        );
                        world.send_event(event);
                    }
                }
            }
            // Check if target has Destructible (object)
            else if let Some(mut destructible) = world.get_mut::<Destructible>(target_entity)
                && let Some((damage, can_damage)) = &weapon_damage
            {
                // Check if weapon can damage this material type
                if can_damage.contains(&destructible.material_type) {
                    let material_type = destructible.material_type;
                    destructible.take_damage(*damage);
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
            
            if should_apply_hit_blink {
                apply_hit_blink(world, target_entity);
            }
        }

        // Consume energy
        if let Some(mut energy) = world.get_mut::<Energy>(self.attacker_entity) {
            let cost = get_energy_cost(EnergyActionType::Move); // Same cost as movement for now
            energy.consume_energy(cost);
        }
    }
}
