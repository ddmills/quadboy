use bevy_ecs::prelude::*;
use macroquad::prelude::trace;

use crate::{
    common::Rand,
    domain::{
        Destructible, Energy, EnergyActionType, EquipmentSlot, EquipmentSlots, Health, HitBlink,
        MaterialType, Player, RangedWeapon, StatType, Stats, WeaponFamily, Zone,
        get_base_energy_cost, systems::destruction_system::EntityDestroyedEvent,
    },
    engine::{Audio, Clock, StableIdRegistry},
    rendering::{
        Glyph, Position, spawn_bullet_trail_in_world, spawn_material_hit_in_world,
        world_to_zone_idx, world_to_zone_local,
    },
};

pub struct ShootAction {
    pub shooter_entity: Entity,
    pub target_pos: (usize, usize, usize),
}

fn resolve_hit_miss(
    attacker_entity: Entity,
    target_entity: Entity,
    world: &mut World,
) -> (bool, bool) {
    // Get target's dodge stat first (immutable borrow)
    let target_dodge = world
        .get::<Stats>(target_entity)
        .map(|stats| stats.get_stat(StatType::Dodge))
        .unwrap_or(0);

    // Determine weapon family for attacker
    let weapon_family = {
        // Try to get equipped ranged weapon
        if let Some(registry) = world.get_resource::<StableIdRegistry>()
            && let Some(equipment) = world.get::<EquipmentSlots>(attacker_entity)
            && let Some(weapon_id) = equipment.get_equipped_item(EquipmentSlot::MainHand)
            && let Some(weapon_entity) = registry.get_entity(weapon_id)
            && let Some(ranged_weapon) = world.get::<RangedWeapon>(weapon_entity)
        {
            ranged_weapon.weapon_family
        }
        // Default to unarmed if no ranged weapon equipped
        else {
            WeaponFamily::Unarmed
        }
    };

    // Get attacker's weapon proficiency stat
    let weapon_proficiency = world
        .get::<Stats>(attacker_entity)
        .map(|stats| stats.get_stat(weapon_family.to_stat_type()))
        .unwrap_or(0);

    let Some(mut rand) = world.get_resource_mut::<Rand>() else {
        return (true, false); // Default to hit if no RNG
    };

    // Roll raw d12 and check for critical BEFORE adding modifiers
    let raw_roll = rand.d12();
    let is_critical = raw_roll == 12; // Critical only on natural 12

    // Calculate final attack roll with weapon proficiency
    let attacker_roll = raw_roll + weapon_proficiency;

    // Roll defender's Defense Value (d12 + Dodge)
    let defender_roll = rand.d12();
    let defense_value = defender_roll + target_dodge;

    // Critical hits always hit, otherwise compare rolls
    let hit = is_critical || attacker_roll >= defense_value;

    (hit, is_critical)
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
        let weapon_info = {
            let Some(registry) = world.get_resource::<StableIdRegistry>() else {
                return;
            };

            let Some(equipment) = world.get::<EquipmentSlots>(self.shooter_entity) else {
                return; // No equipment slots, can't have weapon
            };

            // Check for equipped weapon in MainHand
            let weapon_id = equipment.get_equipped_item(EquipmentSlot::MainHand);

            if let Some(weapon_id) = weapon_id {
                if let Some(weapon_entity) = registry.get_entity(weapon_id) {
                    world.get::<RangedWeapon>(weapon_entity).map(|weapon| {
                        (
                            weapon_entity,
                            weapon.damage_dice.clone(),
                            weapon.can_damage.clone(),
                            weapon.range,
                            weapon.shoot_audio,
                            weapon.current_ammo,
                            weapon.no_ammo_audio,
                        )
                    })
                } else {
                    None
                }
            } else {
                None
            }
        };

        let Some((
            weapon_entity,
            damage_dice,
            can_damage,
            range,
            shoot_audio,
            current_ammo,
            no_ammo_audio,
        )) = weapon_info
        else {
            // No ranged weapon equipped
            return;
        };

        // Check if weapon has ammo
        if let Some(ammo) = current_ammo {
            if ammo == 0 {
                // No ammo - play empty sound and consume energy but don't shoot
                if let Some(empty_audio) = no_ammo_audio {
                    if let Some(audio) = world.get_resource::<Audio>() {
                        audio.play(empty_audio, 0.2);
                    }
                }

                if let Some(mut energy) = world.get_mut::<Energy>(self.shooter_entity) {
                    let cost = get_base_energy_cost(EnergyActionType::Shoot);
                    energy.consume_energy(cost);
                }
                return;
            }
        }

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
            let zone_idx =
                world_to_zone_idx(self.target_pos.0, self.target_pos.1, self.target_pos.2);
            let local = world_to_zone_local(self.target_pos.0, self.target_pos.1);

            let mut found_targets = vec![];
            let mut zones = world.query::<&Zone>();
            for zone in zones.iter(world) {
                if zone.idx == zone_idx {
                    if let Some(entities) = zone.entities.get(local.0, local.1) {
                        found_targets.extend(
                            entities
                                .iter()
                                .filter(|e| world.get::<Health>(**e).is_some())
                                .copied(),
                        );
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

        if targets.is_empty() {
            // Consume energy even if no target hit (shot fired)
            if let Some(mut energy) = world.get_mut::<Energy>(self.shooter_entity) {
                let cost = get_base_energy_cost(EnergyActionType::Shoot);
                energy.consume_energy(cost);
            }
            return;
        }

        // Get current tick once for this shot
        let current_tick = world.resource::<Clock>().current_tick();

        // Process shot on each target at position
        for &target_entity in targets.iter() {
            let mut should_apply_hit_blink = false;

            // Resolve hit/miss for this target
            let (hit, _is_critical) = resolve_hit_miss(self.shooter_entity, target_entity, world);

            // Roll damage if hit
            let rolled_damage = if hit {
                world.resource_scope(|_world, mut rand: Mut<Rand>| {
                    rand.roll(&damage_dice).unwrap_or(1)
                })
            } else {
                0
            };

            if let Some(shooter_pos) = shooter_pos {
                world.resource_scope(|world, mut rand: Mut<Rand>| {
                    spawn_bullet_trail_in_world(
                        world,
                        shooter_pos,
                        self.target_pos,
                        60.0,
                        &mut rand,
                        hit,
                    );
                });
            }

            if let Some(mut health) = world.get_mut::<Health>(target_entity) {
                if can_damage.contains(&MaterialType::Flesh) && hit {
                    health.take_damage(rolled_damage, current_tick);
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

                            let event = EntityDestroyedEvent::with_attacker(
                                target_entity,
                                position_coords,
                                self.shooter_entity,
                                MaterialType::Flesh,
                            );
                            world.send_event(event);

                            // Calculate direction from shooter to target for particles
                            if let Some(shooter_pos) = world.get::<Position>(self.shooter_entity) {
                                let dx = position_coords.0 as f32 - shooter_pos.x;
                                let dy = position_coords.1 as f32 - shooter_pos.y;
                                let direction = macroquad::math::Vec2::new(dx, dy);
                                spawn_material_hit_in_world(
                                    world,
                                    position_coords,
                                    MaterialType::Flesh,
                                    direction,
                                );
                            }
                        }
                    }
                }
            }
            // Check if target has Destructible (object)
            else if let Some(mut destructible) = world.get_mut::<Destructible>(target_entity)
                && hit
            // Only apply damage if attack hits
            {
                // Check if weapon can damage this material type
                if can_damage.contains(&destructible.material_type) {
                    let material_type = destructible.material_type;
                    destructible.take_damage(rolled_damage);
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
                        let event = EntityDestroyedEvent::with_attacker(
                            target_entity,
                            position_coords,
                            self.shooter_entity,
                            material_type,
                        );
                        world.send_event(event);

                        // Calculate direction from shooter to target for particles
                        if let Some(shooter_pos) = world.get::<Position>(self.shooter_entity) {
                            let dx = position_coords.0 as f32 - shooter_pos.x;
                            let dy = position_coords.1 as f32 - shooter_pos.y;
                            let direction = macroquad::math::Vec2::new(dx, dy);
                            spawn_material_hit_in_world(
                                world,
                                position_coords,
                                material_type,
                                direction,
                            );
                        }
                    }
                }
            }

            if should_apply_hit_blink {
                apply_hit_blink(world, target_entity);
            }
        }

        // Decrement ammo for weapons with clips
        if let Some(mut weapon) = world.get_mut::<RangedWeapon>(weapon_entity) {
            if let Some(current) = weapon.current_ammo {
                weapon.current_ammo = Some(current.saturating_sub(1));
            }
        }

        // Consume energy
        if let Some(mut energy) = world.get_mut::<Energy>(self.shooter_entity) {
            let cost = get_base_energy_cost(EnergyActionType::Shoot);
            energy.consume_energy(cost);
        }
    }
}
