use bevy_ecs::prelude::*;

use crate::{
    common::Rand,
    domain::{
        ActiveConditions, BumpAttack, Condition, ConditionSource, ConditionType, CreatureType,
        DefaultMeleeAttack, Destructible, Energy, EnergyActionType, EquipmentSlot, EquipmentSlots,
        Health, HitBlink, HitEffect, KnockbackAnimation, MaterialType, Player, StatType, Stats,
        Weapon, WeaponFamily, WeaponType, Zone, get_base_energy_cost,
        systems::{apply_condition_to_entity, destruction_system::EntityDestroyedEvent},
    },
    engine::{Audio, Clock, StableId, StableIdRegistry},
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
        if let Some(attacker_pos) = world.get::<Position>(self.attacker_entity) {
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

        let (weapon_damage, hit_effects) = {
            world
                .get::<DefaultMeleeAttack>(self.attacker_entity)
                .map(|default_attack| {
                    (
                        Some((
                            default_attack.damage.to_string(),
                            default_attack.can_damage.clone(),
                        )),
                        default_attack.hit_effects.clone(),
                    )
                })
                .unwrap_or((None, Vec::new()))
        };

        self.process_melee_attack(world, weapon_damage, hit_effects);
    }

    fn apply_melee_attack(self, world: &mut World, weapon: &Weapon) {
        if weapon.weapon_type == WeaponType::Melee
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
        let hit_effects = weapon.hit_effects.clone();
        self.process_melee_attack(world, weapon_damage, hit_effects);
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
            && ammo == 0
        {
            if let Some(empty_audio) = weapon.no_ammo_audio
                && let Some(audio) = world.get_resource::<Audio>()
            {
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
            && let Some((sx, sy, _sz)) = attacker_pos
        {
            let distance = ((self.target_pos.0 as i32 - sx as i32).abs()
                + (self.target_pos.1 as i32 - sy as i32).abs()) as usize;

            if distance > range {
                return;
            }
        }

        // Find targets (ranged only targets entities with Health)
        let targets = self.find_ranged_targets(world);

        if let Some(shoot_audio) = weapon.shoot_audio
            && let Some(audio) = world.get_resource::<Audio>()
        {
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
                &weapon.hit_effects,
                current_tick,
                &mut should_apply_hit_blink,
            );

            if should_apply_hit_blink {
                apply_hit_blink(world, target_entity);
            }
        }

        // Decrement ammo
        if let Some(mut weapon_mut) = world.get_mut::<Weapon>(weapon_entity)
            && let Some(current) = weapon_mut.current_ammo
        {
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
        hit_effects: Vec<HitEffect>,
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
                    &hit_effects,
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
        hit_effects: &[HitEffect],
        current_tick: u32,
        should_apply_hit_blink: &mut bool,
    ) {
        let attacker_stable_id = world.get::<StableId>(self.attacker_entity).copied();
        if let Some(mut health) = world.get_mut::<Health>(target_entity) {
            if can_damage.contains(&MaterialType::Flesh) && hit {
                health.take_damage_from_source(rolled_damage, current_tick, attacker_stable_id);
                *should_apply_hit_blink = true;

                // Apply hit effects to flesh targets
                self.apply_hit_effects(world, target_entity, hit_effects);

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
            }
        }
        // Check if target has Destructible (object)
        else if let Some(mut destructible) = world.get_mut::<Destructible>(target_entity)
            && hit
            && can_damage.contains(&destructible.material_type)
        {
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

            if is_destroyed && let Some(position) = world.get::<Position>(target_entity) {
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
                    spawn_material_hit_in_world(world, position_coords, material_type, direction);
                }
            }
        }
    }

    fn apply_hit_effects(
        &self,
        world: &mut World,
        target_entity: Entity,
        hit_effects: &[HitEffect],
    ) {
        for effect in hit_effects {
            // Roll for chance and apply effect if successful
            let mut should_apply = false;
            world.resource_scope(|_world, mut rand: Mut<Rand>| {
                let roll = rand.random();
                should_apply = match effect {
                    HitEffect::Knockback { chance, .. } => roll <= *chance,
                    HitEffect::Poison { chance, .. } => roll <= *chance,
                    HitEffect::Bleeding { chance, .. } => roll <= *chance,
                    HitEffect::Burning { chance, .. } => roll <= *chance,
                    HitEffect::Stun { chance, .. } => roll <= *chance,
                    HitEffect::Slow { chance, .. } => roll <= *chance,
                };
            });

            if should_apply {
                match effect {
                    HitEffect::Knockback { strength, .. } => {
                        self.apply_knockback_effect(world, target_entity, *strength);
                    }
                    HitEffect::Poison {
                        damage_per_tick,
                        duration_ticks,
                        ..
                    } => {
                        self.apply_poison_effect(
                            world,
                            target_entity,
                            *damage_per_tick,
                            *duration_ticks,
                        );
                    }
                    HitEffect::Bleeding {
                        damage_per_tick,
                        duration_ticks,
                        can_stack,
                        ..
                    } => {
                        self.apply_bleeding_effect(
                            world,
                            target_entity,
                            *damage_per_tick,
                            *duration_ticks,
                            *can_stack,
                        );
                    }
                    HitEffect::Burning {
                        damage_per_tick,
                        duration_ticks,
                        ..
                    } => {
                        self.apply_burning_effect(
                            world,
                            target_entity,
                            *damage_per_tick,
                            *duration_ticks,
                        );
                    }
                    HitEffect::Stun { duration_ticks, .. } => {
                        self.apply_stun_effect(world, target_entity, *duration_ticks);
                    }
                    HitEffect::Slow {
                        speed_reduction,
                        duration_ticks,
                        ..
                    } => {
                        self.apply_slow_effect(
                            world,
                            target_entity,
                            *speed_reduction,
                            *duration_ticks,
                        );
                    }
                }
            }
        }
    }

    fn apply_knockback_effect(
        &self,
        world: &mut World,
        target_entity: Entity,
        strength_multiplier: f32,
    ) {
        // Get attacker's knockback stat to calculate knockback distance
        let attacker_knockback = world
            .get::<Stats>(self.attacker_entity)
            .map(|stats| stats.get_stat(StatType::Knockback))
            .unwrap_or(10) as f32; // Default knockback of 10

        // Use Knockback / 2 as requested
        let knockback_distance = ((attacker_knockback / 2.0) * strength_multiplier) as usize;

        if knockback_distance == 0 {
            return;
        }

        // Get target's current position
        let Some(target_pos) = world.get::<Position>(target_entity) else {
            return;
        };
        let target_world_pos = target_pos.world();

        // Get attacker's position to calculate knockback direction
        let Some(attacker_pos) = world.get::<Position>(self.attacker_entity) else {
            return;
        };
        let attacker_world_pos = attacker_pos.world();

        // Calculate knockback direction (from attacker to target)
        let dx = target_world_pos.0 as i32 - attacker_world_pos.0 as i32;
        let dy = target_world_pos.1 as i32 - attacker_world_pos.1 as i32;

        // Normalize direction
        let (norm_dx, norm_dy) = if dx == 0 && dy == 0 {
            (0, 1) // Default to south if same position
        } else if dx.abs() > dy.abs() {
            (dx.signum(), 0) // Primary horizontal movement
        } else {
            (0, dy.signum()) // Primary vertical movement
        };

        // Calculate target position (check for colliders along the path)
        let mut final_x = target_world_pos.0 as i32;
        let mut final_y = target_world_pos.1 as i32;
        let final_z = target_world_pos.2;

        // Check each tile along the knockback path for colliders
        for step in 1..=knockback_distance {
            let test_x = target_world_pos.0 as i32 + (norm_dx * step as i32);
            let test_y = target_world_pos.1 as i32 + (norm_dy * step as i32);

            // Ensure coordinates are valid
            if test_x < 0 || test_y < 0 {
                break;
            }

            let test_pos = (test_x as usize, test_y as usize, final_z);

            // Check for colliders at this position
            if self.has_collider_at_position(world, test_pos) {
                break; // Stop before the collider
            }

            // This position is clear, update final position
            final_x = test_x;
            final_y = test_y;
        }

        // Only apply knockback if there's actual movement
        if final_x as usize != target_world_pos.0 || final_y as usize != target_world_pos.1 {
            // Apply knockback instantly - update position immediately
            let Some(mut target_position) = world.get_mut::<Position>(target_entity) else {
                return;
            };

            let old_world_pos = target_position.world();
            let new_world_pos = (final_x as usize, final_y as usize, final_z);

            // Calculate knockback distance for animation
            let knockback_dx = final_x as f32 - target_world_pos.0 as f32;
            let knockback_dy = final_y as f32 - target_world_pos.1 as f32;

            // Update position immediately
            target_position.x = final_x as f32;
            target_position.y = final_y as f32;
            target_position.z = final_z as f32;

            // Update zone spatial indexing immediately
            self.update_zone_position(world, target_entity, old_world_pos, new_world_pos);

            // Add visual animation component if target has a Glyph
            if world
                .get::<crate::rendering::Glyph>(target_entity)
                .is_some()
            {
                let animation = KnockbackAnimation::new((knockback_dx, knockback_dy), 0.1); // 100ms animation
                world.entity_mut(target_entity).insert(animation);
            }
        }
    }

    fn has_collider_at_position(&self, world: &mut World, pos: (usize, usize, usize)) -> bool {
        let zone_idx = world_to_zone_idx(pos.0, pos.1, pos.2);
        let local = world_to_zone_local(pos.0, pos.1);

        let mut zones = world.query::<&Zone>();
        for zone in zones.iter(world) {
            if zone.idx == zone_idx {
                if let Some(entities) = zone.entities.get(local.0, local.1) {
                    for &entity in entities {
                        if world.get::<crate::domain::Collider>(entity).is_some() {
                            return true;
                        }
                    }
                }
                break;
            }
        }
        false
    }

    fn update_zone_position(
        &self,
        world: &mut World,
        entity: Entity,
        old_pos: (usize, usize, usize),
        new_pos: (usize, usize, usize),
    ) {
        let old_zone_idx = world_to_zone_idx(old_pos.0, old_pos.1, old_pos.2);
        let new_zone_idx = world_to_zone_idx(new_pos.0, new_pos.1, new_pos.2);

        let mut q_zones = world.query::<&mut Zone>();

        // Remove from old zone
        for mut zone in q_zones.iter_mut(world) {
            if zone.idx == old_zone_idx {
                let _ = zone.entities.remove(&entity);
                break;
            }
        }

        // Add to new zone (only if it's a different zone)
        if old_zone_idx != new_zone_idx {
            for mut zone in q_zones.iter_mut(world) {
                if zone.idx == new_zone_idx {
                    let new_local = world_to_zone_local(new_pos.0, new_pos.1);
                    zone.entities.insert(new_local.0, new_local.1, entity);
                    break;
                }
            }
        } else {
            // Same zone, just update position within zone
            for mut zone in q_zones.iter_mut(world) {
                if zone.idx == new_zone_idx {
                    let old_local = world_to_zone_local(old_pos.0, old_pos.1);
                    let new_local = world_to_zone_local(new_pos.0, new_pos.1);

                    if old_local != new_local {
                        let _ = zone.entities.remove(&entity);
                        zone.entities.insert(new_local.0, new_local.1, entity);
                    }
                    break;
                }
            }
        }
    }

    fn apply_poison_effect(
        &self,
        world: &mut World,
        target_entity: Entity,
        damage_per_tick: i32,
        duration_ticks: u32,
    ) {
        // Get the attacker's StableId to use as the condition source
        let condition_source =
            if let Some(attacker_stable_id) = world.get::<StableId>(self.attacker_entity) {
                ConditionSource::entity(*attacker_stable_id)
            } else {
                ConditionSource::Unknown
            };

        // Create the poison condition
        let poison_condition = Condition::new(
            ConditionType::Poisoned {
                damage_per_tick,
                tick_interval: 100, // Damage every 100 ticks
            },
            duration_ticks,
            1.0, // Intensity (not used for poison currently)
            condition_source,
        );

        // Apply the poison condition to the target
        if let Err(err) = apply_condition_to_entity(target_entity, poison_condition, world) {
            // Log error if needed, but don't fail the attack
            eprintln!("Failed to apply poison condition: {}", err);
        }
    }

    fn apply_bleeding_effect(
        &self,
        world: &mut World,
        target_entity: Entity,
        damage_per_tick: i32,
        duration_ticks: u32,
        can_stack: bool,
    ) {
        // Get the attacker's StableId to use as the condition source
        let condition_source =
            if let Some(attacker_stable_id) = world.get::<StableId>(self.attacker_entity) {
                ConditionSource::entity(*attacker_stable_id)
            } else {
                ConditionSource::Unknown
            };

        // Create the bleeding condition
        let bleeding_condition = Condition::new(
            ConditionType::Bleeding {
                damage_per_tick,
                can_stack,
            },
            duration_ticks,
            1.0, // intensity
            condition_source,
        );

        // Apply the bleeding condition to the target
        let _ = apply_condition_to_entity(target_entity, bleeding_condition, world);
    }

    fn apply_burning_effect(
        &self,
        world: &mut World,
        target_entity: Entity,
        damage_per_tick: i32,
        duration_ticks: u32,
    ) {
        // Get the attacker's StableId to use as the condition source
        let condition_source =
            if let Some(attacker_stable_id) = world.get::<StableId>(self.attacker_entity) {
                ConditionSource::entity(*attacker_stable_id)
            } else {
                ConditionSource::Unknown
            };

        // Create the burning condition
        let burning_condition = Condition::new(
            ConditionType::Burning {
                damage_per_tick,
                spread_chance: 0.0, // No spreading for weapon-applied burning
            },
            duration_ticks,
            1.0, // intensity
            condition_source,
        );

        // Apply the burning condition to the target
        let _ = apply_condition_to_entity(target_entity, burning_condition, world);
    }

    fn apply_stun_effect(&self, world: &mut World, target_entity: Entity, duration_ticks: u32) {
        // Get the attacker's StableId to use as the condition source
        let condition_source =
            if let Some(attacker_stable_id) = world.get::<StableId>(self.attacker_entity) {
                ConditionSource::entity(*attacker_stable_id)
            } else {
                ConditionSource::Unknown
            };

        // Create the stun condition
        let stun_condition = Condition::new(
            ConditionType::Stunned,
            duration_ticks,
            1.0, // intensity
            condition_source,
        );

        // Apply the stun condition to the target
        let _ = apply_condition_to_entity(target_entity, stun_condition, world);
    }

    fn apply_slow_effect(
        &self,
        world: &mut World,
        target_entity: Entity,
        speed_reduction: f32,
        duration_ticks: u32,
    ) {
        // Get the attacker's StableId to use as the condition source
        let condition_source =
            if let Some(attacker_stable_id) = world.get::<StableId>(self.attacker_entity) {
                ConditionSource::entity(*attacker_stable_id)
            } else {
                ConditionSource::Unknown
            };

        // Create the slow condition
        let slow_condition = Condition::new(
            ConditionType::Slowed {
                energy_multiplier: 1.0 + speed_reduction, // Convert reduction to multiplier
            },
            duration_ticks,
            1.0, // intensity
            condition_source,
        );

        // Apply the slow condition to the target
        let _ = apply_condition_to_entity(target_entity, slow_condition, world);
    }
}
