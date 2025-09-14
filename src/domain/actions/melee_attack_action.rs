use bevy_ecs::prelude::*;

use crate::{
    common::Rand,
    domain::{
        BumpAttack, DefaultMeleeAttack, Destructible, Energy, EnergyActionType, EquipmentSlot,
        EquipmentSlots, Health, HitBlink, MaterialType, MeleeWeapon, Player, StatType, Stats,
        WeaponFamily, Zone, get_base_energy_cost,
        systems::destruction_system::EntityDestroyedEvent,
    },
    engine::{Audio, Clock, StableIdRegistry},
    rendering::{Glyph, Position, spawn_directional_blood_mist, spawn_material_hit_in_world},
};

pub struct MeleeAttackAction {
    pub attacker_entity: Entity,
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
        // First try to get equipped melee weapon
        if let Some(registry) = world.get_resource::<StableIdRegistry>()
            && let Some(equipment) = world.get::<EquipmentSlots>(attacker_entity)
            && let Some(weapon_id) = equipment.get_equipped_item(EquipmentSlot::MainHand)
            && let Some(weapon_entity) = registry.get_entity(weapon_id)
            && let Some(melee_weapon) = world.get::<MeleeWeapon>(weapon_entity)
        {
            melee_weapon.weapon_family
        }
        // Fall back to default melee attack
        else if let Some(default_attack) = world.get::<DefaultMeleeAttack>(attacker_entity) {
            default_attack.weapon_family
        }
        // Default to unarmed if no weapon or default attack
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

impl Command for MeleeAttackAction {
    fn apply(self, world: &mut World) {
        // Get the weapon damage dice (equipped weapon takes priority, then default attack)
        let weapon_damage = {
            // First check for equipped weapon
            if let Some(registry) = world.get_resource::<StableIdRegistry>()
                && let Some(equipment) = world.get::<EquipmentSlots>(self.attacker_entity)
                && let Some(weapon_id) =
                    equipment.get_equipped_item(crate::domain::EquipmentSlot::MainHand)
                && let Some(weapon_entity) = registry.get_entity(weapon_id)
                && let Some(weapon) = world.get::<MeleeWeapon>(weapon_entity)
            {
                Some((weapon.damage_dice.clone(), weapon.can_damage.clone()))
            }
            // Fall back to default melee attack if no equipped weapon
            else if let Some(default_attack) =
                world.get::<DefaultMeleeAttack>(self.attacker_entity)
            {
                Some((
                    default_attack.damage.to_string(),
                    default_attack.can_damage.clone(),
                ))
            } else {
                None
            }
        };

        // Apply bump attack effect if attacker is a player
        if world.get::<Player>(self.attacker_entity).is_some()
            && let Some(attacker_pos) = world.get::<Position>(self.attacker_entity)
        {
            // Calculate direction from attacker to target
            let dx = self.target_pos.0 as f32 - attacker_pos.x;
            let dy = self.target_pos.1 as f32 - attacker_pos.y;

            // Normalize direction
            let length = (dx * dx + dy * dy).sqrt();
            if length > 0.0 {
                let direction = (dx / length, dy / length);
                world
                    .entity_mut(self.attacker_entity)
                    .insert(BumpAttack::attacked(direction));
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

        if targets.is_empty() {
            return;
        }

        // Get current tick once for this attack
        let current_tick = world.resource::<Clock>().current_tick();

        // Process attack on each target at position
        for &target_entity in targets.iter() {
            let mut should_apply_hit_blink = false;

            // Resolve hit/miss for this target
            let (hit, _is_critical) = resolve_hit_miss(self.attacker_entity, target_entity, world);

            // Roll damage if we have a weapon and hit
            let rolled_damage = if let Some((damage_dice, _)) = &weapon_damage
                && hit
            {
                world.resource_scope(|_world, mut rand: Mut<Rand>| {
                    rand.roll(damage_dice).unwrap_or(1)
                })
            } else {
                0
            };

            if let Some(mut health) = world.get_mut::<Health>(target_entity) {
                if let Some((_, can_damage)) = &weapon_damage
                    && can_damage.contains(&MaterialType::Flesh)
                    && hit
                // Only apply damage if attack hits
                {
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

                    // Add directional blood spray for flesh targets
                    if let Some(attacker_pos) = world.get::<Position>(self.attacker_entity) {
                        let dx = self.target_pos.0 as f32 - attacker_pos.x;
                        let dy = self.target_pos.1 as f32 - attacker_pos.y;
                        let direction = macroquad::math::Vec2::new(dx, dy);

                        // Use moderate intensity for melee attacks
                        spawn_directional_blood_mist(world, self.target_pos, direction, 0.8);
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
                                self.attacker_entity,
                                MaterialType::Flesh,
                            );
                            world.send_event(event);

                            // Calculate direction from attacker to target for particles
                            if let Some(attacker_pos) = world.get::<Position>(self.attacker_entity)
                            {
                                let dx = position_coords.0 as f32 - attacker_pos.x;
                                let dy = position_coords.1 as f32 - attacker_pos.y;
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
                && let Some((_, can_damage)) = &weapon_damage
                && hit
            // Only apply damage if attack hits
            {
                // Check if weapon can damage this material type
                if can_damage.contains(&destructible.material_type) {
                    let material_type = destructible.material_type;
                    destructible.take_damage(rolled_damage);
                    should_apply_hit_blink = true;
                    let is_destroyed = destructible.is_destroyed();

                    // Calculate direction from attacker to target for particles
                    if let Some(attacker_pos) = world.get::<Position>(self.attacker_entity) {
                        let dx = self.target_pos.0 as f32 - attacker_pos.x;
                        let dy = self.target_pos.1 as f32 - attacker_pos.y;
                        let direction = macroquad::math::Vec2::new(dx, dy);
                        spawn_material_hit_in_world(
                            world,
                            self.target_pos,
                            material_type,
                            direction,
                        );
                    }

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
                            let position_coords = position.world();
                            let event = EntityDestroyedEvent::with_attacker(
                                target_entity,
                                position_coords,
                                self.attacker_entity,
                                material_type,
                            );
                            world.send_event(event);

                            // Calculate direction from attacker to target for particles
                            if let Some(attacker_pos) = world.get::<Position>(self.attacker_entity)
                            {
                                let dx = position_coords.0 as f32 - attacker_pos.x;
                                let dy = position_coords.1 as f32 - attacker_pos.y;
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
            }

            if should_apply_hit_blink {
                apply_hit_blink(world, target_entity);
            }
        }

        // Consume energy
        if let Some(mut energy) = world.get_mut::<Energy>(self.attacker_entity) {
            let cost = get_base_energy_cost(EnergyActionType::Attack);
            energy.consume_energy(cost);
        }
    }
}
