use bevy_ecs::prelude::*;

use crate::{
    common::Rand,
    domain::{
        BumpAttack, CreatureType, DefaultMeleeAttack, Destructible, Energy, EnergyActionType,
        EquipmentSlot, EquipmentSlots, Health, HitBlink, MaterialType, Player, StatType, Stats,
        Weapon, WeaponFamily, WeaponType, Zone, get_base_energy_cost,
        systems::destruction_system::EntityDestroyedEvent,
    },
    engine::{Audio, Clock, StableIdRegistry},
    rendering::{
        Glyph, Position, spawn_bullet_trail_in_world, spawn_directional_blood_mist,
        spawn_material_hit_in_world, world_to_zone_idx, world_to_zone_local,
    },
};

pub struct AttackAction {
    pub attacker_entity: Entity,
    pub target_pos: (usize, usize, usize),
    pub is_bump_attack: bool,
}

fn resolve_hit_miss(
    attacker_entity: Entity,
    target_entity: Entity,
    world: &mut World,
) -> (bool, bool) {
    let target_dodge = world
        .get::<Stats>(target_entity)
        .map(|stats| stats.get_stat(StatType::Dodge))
        .unwrap_or(0);

    let weapon_family = {
        if let Some(registry) = world.get_resource::<StableIdRegistry>()
            && let Some(equipment) = world.get::<EquipmentSlots>(attacker_entity)
            && let Some(weapon_id) = equipment.get_equipped_item(EquipmentSlot::MainHand)
            && let Some(weapon_entity) = registry.get_entity(weapon_id)
            && let Some(weapon) = world.get::<Weapon>(weapon_entity)
        {
            weapon.weapon_family
        } else if let Some(default_attack) = world.get::<DefaultMeleeAttack>(attacker_entity) {
            default_attack.weapon_family
        } else {
            WeaponFamily::Unarmed
        }
    };

    let weapon_proficiency = world
        .get::<Stats>(attacker_entity)
        .map(|stats| stats.get_stat(weapon_family.to_stat_type()))
        .unwrap_or(0);

    let Some(mut rand) = world.get_resource_mut::<Rand>() else {
        return (true, false);
    };

    let raw_roll = rand.d12();
    let is_critical = raw_roll == 12;

    let attacker_roll = raw_roll + weapon_proficiency;
    let defender_roll = rand.d12();
    let defense_value = defender_roll + target_dodge;

    let hit = is_critical || attacker_roll >= defense_value;

    (hit, is_critical)
}

fn apply_hit_blink(world: &mut World, target_entity: Entity) {
    if let Some(mut existing_hit_blink) = world.get_mut::<HitBlink>(target_entity) {
        existing_hit_blink.duration_remaining = 0.05;
    } else if world.get::<Glyph>(target_entity).is_some() {
        let hit_blink = HitBlink::attacked();
        world.entity_mut(target_entity).insert(hit_blink);
    }
}

impl Command for AttackAction {
    fn apply(self, world: &mut World) {
        let attacker_pos = world
            .get::<Position>(self.attacker_entity)
            .map(|p| p.world());

        // Get the equipped weapon (if any)
        let weapon_info = {
            let Some(registry) = world.get_resource::<StableIdRegistry>() else {
                // Try default melee attack if no registry
                return self.apply_default_melee_attack(world);
            };

            let Some(equipment) = world.get::<EquipmentSlots>(self.attacker_entity) else {
                return self.apply_default_melee_attack(world);
            };

            let weapon_id = equipment.get_equipped_item(EquipmentSlot::MainHand);

            if let Some(weapon_id) = weapon_id {
                if let Some(weapon_entity) = registry.get_entity(weapon_id) {
                    world
                        .get::<Weapon>(weapon_entity)
                        .map(|weapon| (weapon_entity, weapon.clone()))
                } else {
                    None
                }
            } else {
                None
            }
        };

        let Some((weapon_entity, weapon)) = weapon_info else {
            return self.apply_default_melee_attack(world);
        };

        // For bump attacks, only use melee weapons - ignore ranged weapons
        if self.is_bump_attack && weapon.weapon_type == WeaponType::Ranged {
            return self.apply_default_melee_attack(world);
        }

        match weapon.weapon_type {
            WeaponType::Melee => self.apply_melee_attack(world, &weapon),
            WeaponType::Ranged => {
                // Ranged attacks should not be bump attacks
                if self.is_bump_attack {
                    return self.apply_default_melee_attack(world);
                }
                self.apply_ranged_attack(world, weapon_entity, &weapon, attacker_pos)
            }
        }
    }
}

impl AttackAction {
    fn apply_default_melee_attack(self, world: &mut World) {
        // Apply bump attack effect if attacker is a player (for unarmed/default melee)
        if world.get::<Player>(self.attacker_entity).is_some()
            && let Some(attacker_pos) = world.get::<Position>(self.attacker_entity)
        {
            let dx = self.target_pos.0 as f32 - attacker_pos.x;
            let dy = self.target_pos.1 as f32 - attacker_pos.y;

            let length = (dx * dx + dy * dy).sqrt();
            if length > 0.0 {
                let direction = (dx / length, dy / length);
                world
                    .entity_mut(self.attacker_entity)
                    .insert(BumpAttack::attacked(direction));
            }
        }

        let weapon_damage = {
            world.get::<DefaultMeleeAttack>(self.attacker_entity).map(|default_attack| (
                    default_attack.damage.to_string(),
                    default_attack.can_damage.clone(),
                ))
        };

        self.process_melee_attack(world, weapon_damage);
    }

    fn apply_melee_attack(self, world: &mut World, weapon: &Weapon) {
        // Apply bump attack effect if attacker is a player using a melee weapon
        if weapon.weapon_type == WeaponType::Melee
            && world.get::<Player>(self.attacker_entity).is_some()
            && let Some(attacker_pos) = world.get::<Position>(self.attacker_entity)
        {
            let dx = self.target_pos.0 as f32 - attacker_pos.x;
            let dy = self.target_pos.1 as f32 - attacker_pos.y;

            let length = (dx * dx + dy * dy).sqrt();
            if length > 0.0 {
                let direction = (dx / length, dy / length);
                world
                    .entity_mut(self.attacker_entity)
                    .insert(BumpAttack::attacked(direction));
            }
        }

        let weapon_damage = Some((weapon.damage_dice.clone(), weapon.can_damage.clone()));
        self.process_melee_attack(world, weapon_damage);
    }

    fn apply_ranged_attack(
        self,
        world: &mut World,
        weapon_entity: Entity,
        weapon: &Weapon,
        attacker_pos: Option<(usize, usize, usize)>,
    ) {
        // Check if weapon has ammo
        if let Some(ammo) = weapon.current_ammo
            && ammo == 0 {
                if let Some(empty_audio) = weapon.no_ammo_audio
                    && let Some(audio) = world.get_resource::<Audio>() {
                        audio.play(empty_audio, 0.2);
                    }

                if let Some(mut energy) = world.get_mut::<Energy>(self.attacker_entity) {
                    let cost = get_base_energy_cost(EnergyActionType::Shoot);
                    energy.consume_energy(cost);
                }
                return;
            }

        // Check if target is within range
        if let Some(range) = weapon.range
            && let Some((sx, sy, _sz)) = attacker_pos {
                let distance = ((self.target_pos.0 as i32 - sx as i32).abs()
                    + (self.target_pos.1 as i32 - sy as i32).abs())
                    as usize;

                if distance > range {
                    return;
                }
            }

        // Find targets (ranged only targets entities with Health)
        let targets = self.find_ranged_targets(world);

        if let Some(shoot_audio) = weapon.shoot_audio
            && let Some(audio) = world.get_resource::<Audio>() {
                audio.play(shoot_audio, 0.1);
            }

        if targets.is_empty() {
            // Consume energy even if no target hit (shot fired)
            if let Some(mut energy) = world.get_mut::<Energy>(self.attacker_entity) {
                let cost = get_base_energy_cost(EnergyActionType::Shoot);
                energy.consume_energy(cost);
            }
            return;
        }

        let current_tick = world.resource::<Clock>().current_tick();

        // Process shot on each target
        for &target_entity in targets.iter() {
            let mut should_apply_hit_blink = false;

            let (hit, _is_critical) = resolve_hit_miss(self.attacker_entity, target_entity, world);

            let rolled_damage = if hit {
                world.resource_scope(|_world, mut rand: Mut<Rand>| {
                    rand.roll(&weapon.damage_dice).unwrap_or(1)
                })
            } else {
                0
            };

            // Spawn bullet trail
            if let Some(attacker_pos) = attacker_pos {
                world.resource_scope(|world, mut rand: Mut<Rand>| {
                    spawn_bullet_trail_in_world(
                        world,
                        attacker_pos,
                        self.target_pos,
                        60.0,
                        &mut rand,
                        hit,
                    );
                });
            }

            self.apply_damage_to_target(
                world,
                target_entity,
                rolled_damage,
                hit,
                &weapon.can_damage,
                current_tick,
                &mut should_apply_hit_blink,
            );

            if should_apply_hit_blink {
                apply_hit_blink(world, target_entity);
            }
        }

        // Decrement ammo
        if let Some(mut weapon_mut) = world.get_mut::<Weapon>(weapon_entity)
            && let Some(current) = weapon_mut.current_ammo {
                weapon_mut.current_ammo = Some(current.saturating_sub(1));
            }

        // Consume energy
        if let Some(mut energy) = world.get_mut::<Energy>(self.attacker_entity) {
            let cost = get_base_energy_cost(EnergyActionType::Shoot);
            energy.consume_energy(cost);
        }
    }

    fn process_melee_attack(
        self,
        world: &mut World,
        weapon_damage: Option<(String, Vec<MaterialType>)>,
    ) {
        let targets = self.find_melee_targets(world);

        if targets.is_empty() {
            return;
        }

        let current_tick = world.resource::<Clock>().current_tick();

        for &target_entity in targets.iter() {
            let mut should_apply_hit_blink = false;

            let (hit, _is_critical) = resolve_hit_miss(self.attacker_entity, target_entity, world);

            let rolled_damage = if let Some((damage_dice, _)) = &weapon_damage
                && hit
            {
                world.resource_scope(|_world, mut rand: Mut<Rand>| {
                    rand.roll(damage_dice).unwrap_or(1)
                })
            } else {
                0
            };

            if let Some((_, can_damage)) = &weapon_damage {
                self.apply_damage_to_target(
                    world,
                    target_entity,
                    rolled_damage,
                    hit,
                    can_damage,
                    current_tick,
                    &mut should_apply_hit_blink,
                );
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

    fn find_melee_targets(&self, world: &mut World) -> Vec<Entity> {
        let zone_idx = world_to_zone_idx(self.target_pos.0, self.target_pos.1, self.target_pos.2);
        let local = world_to_zone_local(self.target_pos.0, self.target_pos.1);

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
    }

    fn find_ranged_targets(&self, world: &mut World) -> Vec<Entity> {
        let zone_idx = world_to_zone_idx(self.target_pos.0, self.target_pos.1, self.target_pos.2);
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
    }

    fn apply_damage_to_target(
        &self,
        world: &mut World,
        target_entity: Entity,
        rolled_damage: i32,
        hit: bool,
        can_damage: &[MaterialType],
        current_tick: u32,
        should_apply_hit_blink: &mut bool,
    ) {
        if let Some(mut health) = world.get_mut::<Health>(target_entity) {
            if can_damage.contains(&MaterialType::Flesh) && hit {
                health.take_damage(rolled_damage, current_tick);
                *should_apply_hit_blink = true;
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

                    spawn_directional_blood_mist(world, self.target_pos, direction, 0.8);
                }

                if is_dead {
                    let position_data = world.get::<Position>(target_entity).map(|p| p.world());

                    if let Some(position_coords) = position_data {
                        // Play creature-specific death audio
                        if let Some(creature_type) = world.get::<CreatureType>(target_entity)
                            && let Some(audio) = world.get_resource::<Audio>() {
                                audio.play(creature_type.death_audio_key(), 0.5);
                            }

                        let event = EntityDestroyedEvent::with_attacker(
                            target_entity,
                            position_coords,
                            self.attacker_entity,
                            MaterialType::Flesh,
                        );
                        world.send_event(event);

                        if let Some(attacker_pos) = world.get::<Position>(self.attacker_entity) {
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
            && hit
            && can_damage.contains(&destructible.material_type) {
                let material_type = destructible.material_type;
                destructible.take_damage(rolled_damage);
                *should_apply_hit_blink = true;
                let is_destroyed = destructible.is_destroyed();

                if let Some(attacker_pos) = world.get::<Position>(self.attacker_entity) {
                    let dx = self.target_pos.0 as f32 - attacker_pos.x;
                    let dy = self.target_pos.1 as f32 - attacker_pos.y;
                    let direction = macroquad::math::Vec2::new(dx, dy);
                    spawn_material_hit_in_world(world, self.target_pos, material_type, direction);
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

                if is_destroyed
                    && let Some(position) = world.get::<Position>(target_entity) {
                        let position_coords = position.world();
                        let event = EntityDestroyedEvent::with_attacker(
                            target_entity,
                            position_coords,
                            self.attacker_entity,
                            material_type,
                        );
                        world.send_event(event);

                        if let Some(attacker_pos) = world.get::<Position>(self.attacker_entity) {
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
