use bevy_ecs::prelude::*;
use macroquad::prelude::trace;

use crate::{
    common::Rand,
    domain::{
        BumpAttack, Condition, ConditionSource, ConditionType, DefaultMeleeAttack,
        DefaultRangedAttack, Destructible, Energy, EnergyActionType, EquipmentSlot, EquipmentSlots,
        Health, HitBlink, HitEffect, KnockbackAnimation, Label, MaterialType, PlayerPosition,
        StatType, Stats, Weapon, WeaponFamily, WeaponType, Zone, get_base_energy_cost,
        systems::{
            apply_condition_to_entity, condition_system::spawn_condition_particles,
            destruction_system::EntityDestroyedEvent,
        },
    },
    engine::{Audio, Clock, StableId, StableIdRegistry},
    rendering::{
        Glyph, Position, spawn_bullet_trail_in_world, spawn_delayed_material_hit,
        spawn_directional_blood_mist, spawn_material_hit_in_world, spawn_particle_effect,
        world_to_zone_idx, world_to_zone_local,
    },
};

pub struct AttackAction {
    pub attacker_stable_id: StableId,
    pub weapon_stable_id: Option<StableId>,
    pub target_stable_id: StableId,
    pub is_bump_attack: bool,
}

fn resolve_hit_miss(
    attacker_entity: Entity,
    target_entity: Entity,
    world: &mut World,
) -> (bool, bool) {
    if world.get::<Destructible>(target_entity).is_some()
        && world.get::<Health>(target_entity).is_none()
    {
        return (true, false); // hit=true, critical=false
    }

    let target_dodge = world
        .get::<Stats>(target_entity)
        .map(|stats| stats.get_stat(StatType::Dodge))
        .unwrap_or(0);

    let weapon_family = {
        if let Some(registry) = world.get_resource::<StableIdRegistry>()
            && let Some(equipment) = world.get::<EquipmentSlots>(attacker_entity)
            && let Some(weapon_id) = equipment.get_equipped_item(EquipmentSlot::MainHand)
            && let Some(weapon_entity) = registry.get_entity(StableId(weapon_id))
            && let Some(weapon) = world.get::<Weapon>(weapon_entity)
        {
            weapon.weapon_family
        } else if let Some(default_attack) = world.get::<DefaultMeleeAttack>(attacker_entity) {
            default_attack.weapon.weapon_family
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

fn calculate_direction(
    from_pos: (usize, usize, usize),
    to_pos: (usize, usize, usize),
) -> macroquad::math::Vec2 {
    let dx = to_pos.0 as f32 - from_pos.0 as f32;
    let dy = to_pos.1 as f32 - from_pos.1 as f32;
    macroquad::math::Vec2::new(dx, dy)
}

fn calculate_normalized_direction(
    from_pos: (usize, usize, usize),
    to_pos: (usize, usize, usize),
) -> Option<(f32, f32)> {
    let dx = to_pos.0 as f32 - from_pos.0 as f32;
    let dy = to_pos.1 as f32 - from_pos.1 as f32;

    let length = (dx * dx + dy * dy).sqrt();
    if length > 0.0 {
        Some((dx / length, dy / length))
    } else {
        None
    }
}

impl Command for AttackAction {
    fn apply(self, world: &mut World) {
        let Some(registry) = world.get_resource::<StableIdRegistry>() else {
            eprintln!("AttackAction: StableIdRegistry not found");
            return;
        };

        let Some(attacker_entity) = registry.get_entity(self.attacker_stable_id) else {
            eprintln!(
                "AttackAction: Attacker entity not found for stable ID {}",
                self.attacker_stable_id.0
            );
            return;
        };

        let attacker_pos = world.get::<Position>(attacker_entity).map(|p| p.world());

        let Some(target_entity) = registry.get_entity(self.target_stable_id) else {
            eprintln!(
                "AttackAction: Target entity not found for stable ID {}",
                self.target_stable_id.0
            );
            return;
        };

        let Some(target_pos) = world.get::<Position>(target_entity).map(|p| p.world()) else {
            eprintln!("AttackAction: Target entity has no Position component");
            return;
        };

        // Use the new unified weapon resolution
        let (weapon, weapon_entity) = {
            let Some((weapon, weapon_entity)) =
                self.resolve_weapon(world, attacker_entity, registry)
            else {
                eprintln!(
                    "AttackAction: No weapon found for attacker {}",
                    self.attacker_stable_id.0
                );
                return;
            };
            (weapon, weapon_entity)
        };

        // Use the unified attack method - only proceed if we have attacker position
        if let Some(attacker_pos) = attacker_pos {
            self.apply_attack(
                world,
                attacker_entity,
                target_entity,
                target_pos,
                &weapon,
                weapon_entity,
                attacker_pos,
            );
        } else {
            eprintln!("AttackAction: Attacker entity has no Position component");
        }
    }
}

impl AttackAction {
    fn resolve_weapon(
        &self,
        world: &World,
        attacker_entity: Entity,
        registry: &StableIdRegistry,
    ) -> Option<(Weapon, Option<Entity>)> {
        // Check for explicitly specified weapon first
        if let Some(weapon_stable_id) = self.weapon_stable_id {
            if let Some(weapon_entity) = registry.get_entity(weapon_stable_id) {
                if let Some(weapon) = world.get::<Weapon>(weapon_entity) {
                    return Some((weapon.clone(), Some(weapon_entity)));
                }
            }
        }

        // Check for equipped weapon in main hand
        if let Some(equipment) = world.get::<EquipmentSlots>(attacker_entity) {
            if let Some(weapon_id) = equipment.get_equipped_item(EquipmentSlot::MainHand) {
                if let Some(weapon_entity) = registry.get_entity(StableId(weapon_id)) {
                    if let Some(weapon) = world.get::<Weapon>(weapon_entity) {
                        return Some((weapon.clone(), Some(weapon_entity)));
                    }
                }
            }
        }

        // For non-bump attacks, check for default ranged attack first
        if !self.is_bump_attack {
            if let Some(default_ranged) = world.get::<DefaultRangedAttack>(attacker_entity) {
                if default_ranged.has_ammo() {
                    return Some((default_ranged.weapon.clone(), None));
                }
            }
        }

        // Fall back to default melee attack
        if let Some(default_melee) = world.get::<DefaultMeleeAttack>(attacker_entity) {
            return Some((default_melee.weapon.clone(), None));
        }

        None
    }

    fn apply_attack(
        self,
        world: &mut World,
        attacker_entity: Entity,
        target_entity: Entity,
        target_pos: (usize, usize, usize),
        weapon: &Weapon,
        weapon_entity: Option<Entity>,
        attacker_pos: (usize, usize, usize),
    ) {
        match weapon.weapon_type {
            WeaponType::Melee => self.apply_unified_melee_attack(
                world,
                attacker_entity,
                target_entity,
                target_pos,
                weapon,
                attacker_pos,
            ),
            WeaponType::Ranged => {
                // For bump attacks, should fall back to melee
                if self.is_bump_attack {
                    // This shouldn't happen due to weapon resolution logic, but handle gracefully
                    let default_weapon = world
                        .get::<DefaultMeleeAttack>(attacker_entity)
                        .map(|default_melee| default_melee.weapon.clone());
                    if let Some(default_weapon) = default_weapon {
                        return self.apply_unified_melee_attack(
                            world,
                            attacker_entity,
                            target_entity,
                            target_pos,
                            &default_weapon,
                            attacker_pos,
                        );
                    }
                    return;
                }
                self.apply_unified_ranged_attack(
                    world,
                    attacker_entity,
                    target_entity,
                    target_pos,
                    weapon,
                    weapon_entity,
                    attacker_pos,
                )
            }
        }
    }

    fn apply_unified_melee_attack(
        self,
        world: &mut World,
        attacker_entity: Entity,
        target_entity: Entity,
        target_pos: (usize, usize, usize),
        weapon: &Weapon,
        attacker_pos: (usize, usize, usize),
    ) {
        // Add bump attack animation for melee attacks
        if let Some(direction) = calculate_normalized_direction(attacker_pos, target_pos) {
            world
                .entity_mut(attacker_entity)
                .insert(BumpAttack::attacked(direction));
        }

        // Process the attack
        let targets = vec![target_entity];
        let current_tick = world.resource::<Clock>().current_tick();

        for &target_entity in targets.iter() {
            let mut should_apply_hit_blink = false;
            let (hit, _is_critical) = resolve_hit_miss(attacker_entity, target_entity, world);

            let rolled_damage = if hit {
                world.resource_scope(|_world, mut rand: Mut<Rand>| {
                    rand.roll(&weapon.damage_dice).unwrap_or(1)
                })
            } else {
                0
            };

            self.apply_damage_to_target(
                world,
                attacker_entity,
                target_entity,
                rolled_damage,
                hit,
                &weapon.can_damage,
                &weapon.hit_effects,
                current_tick,
                &mut should_apply_hit_blink,
                attacker_pos,
                target_pos,
            );

            if should_apply_hit_blink {
                apply_hit_blink(world, target_entity);
            }
        }

        // Consume energy
        if let Some(mut energy) = world.get_mut::<Energy>(attacker_entity) {
            let cost = get_base_energy_cost(EnergyActionType::Attack);
            energy.consume_energy(cost);
        }
    }

    fn apply_unified_ranged_attack(
        self,
        world: &mut World,
        attacker_entity: Entity,
        target_entity: Entity,
        target_pos: (usize, usize, usize),
        weapon: &Weapon,
        weapon_entity: Option<Entity>,
        attacker_pos: (usize, usize, usize),
    ) {
        // Check ammo for equipped weapons (default weapons have infinite ammo via None)
        if weapon_entity.is_some() && weapon.current_ammo == Some(0) {
            // Play empty sound and consume energy
            if let Some(empty_audio) = weapon.no_ammo_audio {
                if let Some(mut audio) = world.get_resource_mut::<Audio>() {
                    audio
                        .clip(empty_audio)
                        .volume(0.2)
                        .position(attacker_pos)
                        .play();
                }
            }
            if let Some(mut energy) = world.get_mut::<Energy>(attacker_entity) {
                let cost = get_base_energy_cost(EnergyActionType::Shoot);
                energy.consume_energy(cost);
            }
            return;
        }

        // Check range
        if let Some(range) = weapon.range {
            let distance = ((target_pos.0 as i32 - attacker_pos.0 as i32).abs()
                + (target_pos.1 as i32 - attacker_pos.1 as i32).abs())
                as usize;
            if distance > range {
                return;
            }
        }

        // Play shoot sound
        if let Some(shoot_audio) = weapon.shoot_audio {
            if let Some(mut audio) = world.get_resource_mut::<Audio>() {
                audio
                    .clip(shoot_audio)
                    .volume(0.1)
                    .position(attacker_pos)
                    .play();
            }
        }

        let targets = vec![target_entity];
        let current_tick = world.resource::<Clock>().current_tick();

        // Process shot on each target
        for &target_entity in targets.iter() {
            let mut should_apply_hit_blink = false;
            let (hit, _is_critical) = resolve_hit_miss(attacker_entity, target_entity, world);

            let rolled_damage = if hit {
                world.resource_scope(|_world, mut rand: Mut<Rand>| {
                    rand.roll(&weapon.damage_dice).unwrap_or(1)
                })
            } else {
                0
            };

            // Spawn particle effects
            world.resource_scope(|world, mut rand: Mut<Rand>| {
                if let Some(effect_id) = &weapon.particle_effect_id {
                    spawn_particle_effect(world, effect_id, attacker_pos, target_pos, &mut rand);
                } else {
                    spawn_bullet_trail_in_world(world, attacker_pos, target_pos, 60.0, &mut rand);
                }
            });

            // Spawn delayed hit effect
            self.spawn_ranged_hit_effect(
                world,
                target_entity,
                target_pos,
                attacker_pos,
                weapon.particle_effect_id.as_ref(),
                hit,
            );

            self.apply_damage_to_target(
                world,
                attacker_entity,
                target_entity,
                rolled_damage,
                hit,
                &weapon.can_damage,
                &weapon.hit_effects,
                current_tick,
                &mut should_apply_hit_blink,
                attacker_pos,
                target_pos,
            );

            if should_apply_hit_blink {
                apply_hit_blink(world, target_entity);
            }
        }

        // Consume ammo for equipped weapons
        if let Some(weapon_entity) = weapon_entity {
            if let Some(mut weapon_mut) = world.get_mut::<Weapon>(weapon_entity) {
                if let Some(current) = weapon_mut.current_ammo {
                    weapon_mut.current_ammo = Some(current.saturating_sub(1));
                }
            }
        } else {
            // For default ranged attacks, consume ammo in the component
            if let Some(mut default_ranged) = world.get_mut::<DefaultRangedAttack>(attacker_entity)
            {
                default_ranged.consume_ammo();
            }
        }

        // Consume energy
        if let Some(mut energy) = world.get_mut::<Energy>(attacker_entity) {
            let cost = get_base_energy_cost(EnergyActionType::Shoot);
            energy.consume_energy(cost);
        }
    }

    fn spawn_ranged_hit_effect(
        &self,
        world: &mut World,
        target_entity: Entity,
        target_pos: (usize, usize, usize),
        attacker_pos: (usize, usize, usize),
        weapon_effect_id: Option<&crate::rendering::ParticleEffectId>,
        hit: bool,
    ) {
        if !hit {
            return;
        }

        // Determine target material type
        let target_material = if let Some(destructible) = world.get::<Destructible>(target_entity) {
            destructible.material_type
        } else {
            MaterialType::Flesh // Default for Health entities
        };

        // Calculate bullet travel time based on weapon speed
        let distance = ((target_pos.0 as f32 - attacker_pos.0 as f32).powi(2)
            + (target_pos.1 as f32 - attacker_pos.1 as f32).powi(2))
        .sqrt();

        let bullet_speed = weapon_effect_id
            .map(|effect| effect.get_bullet_speed())
            .unwrap_or(60.0);

        let travel_time = distance / bullet_speed;

        let direction = calculate_direction(attacker_pos, target_pos);

        spawn_delayed_material_hit(world, target_pos, target_material, direction, travel_time);
    }

    fn apply_damage_to_target(
        &self,
        world: &mut World,
        attacker_entity: Entity,
        target_entity: Entity,
        rolled_damage: i32,
        hit: bool,
        can_damage: &[MaterialType],
        hit_effects: &[HitEffect],
        current_tick: u32,
        should_apply_hit_blink: &mut bool,
        attacker_pos: (usize, usize, usize),
        target_pos: (usize, usize, usize),
    ) {
        let attacker_stable_id = world.get::<StableId>(attacker_entity).copied();
        if let Some(mut health) = world.get_mut::<Health>(target_entity) {
            if can_damage.contains(&MaterialType::Flesh) && hit {
                health.take_damage_from_source(rolled_damage, current_tick, attacker_stable_id);
                *should_apply_hit_blink = true;

                // Apply hit effects to flesh targets
                self.apply_hit_effects(world, attacker_entity, target_entity, hit_effects);

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
                let direction = calculate_direction(attacker_pos, target_pos);
                spawn_directional_blood_mist(world, target_pos, direction, 0.8);
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

            let direction = calculate_direction(attacker_pos, target_pos);
            spawn_material_hit_in_world(world, target_pos, material_type, direction);

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

            if is_destroyed {
                let event = EntityDestroyedEvent::with_attacker(
                    target_entity,
                    target_pos,
                    attacker_entity,
                    material_type,
                );
                world.send_event(event);

                let direction = calculate_direction(attacker_pos, target_pos);
                spawn_material_hit_in_world(world, target_pos, material_type, direction);
            }
        }
    }

    fn apply_hit_effects(
        &self,
        world: &mut World,
        attacker_entity: Entity,
        target_entity: Entity,
        hit_effects: &[HitEffect],
    ) {
        for effect in hit_effects {
            // Roll for chance and apply effect if successful
            let mut should_apply = false;
            let mut roll_result = 0.0;
            let mut effect_chance = 0.0;

            world.resource_scope(|_world, mut rand: Mut<Rand>| {
                let roll = rand.random();
                roll_result = roll;

                should_apply = match effect {
                    HitEffect::Knockback { chance, .. } => {
                        effect_chance = *chance;
                        roll <= *chance
                    }
                    HitEffect::Poison { chance, .. } => {
                        effect_chance = *chance;
                        roll <= *chance
                    }
                    HitEffect::Bleeding { chance, .. } => {
                        effect_chance = *chance;
                        roll <= *chance
                    }
                    HitEffect::Burning { chance, .. } => {
                        effect_chance = *chance;
                        roll <= *chance
                    }
                };
            });

            if should_apply {
                match effect {
                    HitEffect::Knockback { strength, .. } => {
                        // We need to get positions for knockback
                        if let Some(attacker_pos_comp) = world.get::<Position>(attacker_entity)
                            && let Some(target_pos_comp) = world.get::<Position>(target_entity)
                        {
                            self.apply_knockback_effect(
                                world,
                                attacker_entity,
                                target_entity,
                                *strength,
                                attacker_pos_comp.world(),
                                target_pos_comp.world(),
                            );
                        }
                    }
                    HitEffect::Poison {
                        damage_per_tick,
                        duration_ticks,
                        ..
                    } => {
                        self.apply_poison_effect(
                            world,
                            attacker_entity,
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
                            attacker_entity,
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
                            attacker_entity,
                            target_entity,
                            *damage_per_tick,
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
        attacker_entity: Entity,
        target_entity: Entity,
        strength_multiplier: f32,
        attacker_pos: (usize, usize, usize),
        target_pos: (usize, usize, usize),
    ) {
        // Get attacker's knockback stat to calculate knockback distance
        let attacker_knockback = world
            .get::<Stats>(attacker_entity)
            .map(|stats| stats.get_stat(StatType::Knockback))
            .unwrap_or(10) as f32; // Default knockback of 10

        // Use Knockback / 2 as requested
        let knockback_distance = ((attacker_knockback / 2.0) * strength_multiplier) as usize;

        if knockback_distance == 0 {
            return;
        }

        // Calculate knockback direction (from attacker to target)
        let dx = target_pos.0 as i32 - attacker_pos.0 as i32;
        let dy = target_pos.1 as i32 - attacker_pos.1 as i32;

        // Normalize direction
        let (norm_dx, norm_dy) = if dx == 0 && dy == 0 {
            (0, 1) // Default to south if same position
        } else if dx.abs() > dy.abs() {
            (dx.signum(), 0) // Primary horizontal movement
        } else {
            (0, dy.signum()) // Primary vertical movement
        };

        // Calculate target position (check for colliders along the path)
        let mut final_x = target_pos.0 as i32;
        let mut final_y = target_pos.1 as i32;
        let final_z = target_pos.2;

        // Check each tile along the knockback path for colliders
        for step in 1..=knockback_distance {
            let test_x = target_pos.0 as i32 + (norm_dx * step as i32);
            let test_y = target_pos.1 as i32 + (norm_dy * step as i32);

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
        if final_x as usize != target_pos.0 || final_y as usize != target_pos.1 {
            // Apply knockback instantly - update position immediately
            let Some(mut target_position) = world.get_mut::<Position>(target_entity) else {
                return;
            };

            let old_world_pos = target_pos;
            let new_world_pos = (final_x as usize, final_y as usize, final_z);

            // Calculate knockback distance for animation
            let knockback_dx = final_x as f32 - target_pos.0 as f32;
            let knockback_dy = final_y as f32 - target_pos.1 as f32;

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
        attacker_entity: Entity,
        target_entity: Entity,
        damage_per_tick: i32,
        duration_ticks: u32,
    ) {
        // Get entity names for logging
        let attacker_name = world
            .get::<Label>(attacker_entity)
            .map(|label| label.get().to_string())
            .unwrap_or_else(|| format!("Entity({:?})", attacker_entity));

        let target_name = world
            .get::<Label>(target_entity)
            .map(|label| label.get().to_string())
            .unwrap_or_else(|| format!("Entity({:?})", target_entity));

        // Get the attacker's StableId to use as the condition source
        let condition_source =
            if let Some(attacker_stable_id) = world.get::<StableId>(attacker_entity) {
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
        match apply_condition_to_entity(target_entity, poison_condition, world) {
            Ok(()) => {
                // Immediately spawn particle effects for the new condition
                spawn_condition_particles(world);
            }
            Err(err) => {
                eprintln!("Failed to apply poison condition: {}", err);
            }
        }
    }

    fn apply_bleeding_effect(
        &self,
        world: &mut World,
        attacker_entity: Entity,
        target_entity: Entity,
        damage_per_tick: i32,
        duration_ticks: u32,
        can_stack: bool,
    ) {
        // Get the attacker's StableId to use as the condition source
        let condition_source =
            if let Some(attacker_stable_id) = world.get::<StableId>(attacker_entity) {
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
        match apply_condition_to_entity(target_entity, bleeding_condition, world) {
            Ok(()) => {
                // Immediately spawn particle effects for the new condition
                spawn_condition_particles(world);
            }
            Err(err) => {
                eprintln!("Failed to apply bleeding condition: {}", err);
            }
        }
    }

    fn apply_burning_effect(
        &self,
        world: &mut World,
        attacker_entity: Entity,
        target_entity: Entity,
        damage_per_tick: i32,
        duration_ticks: u32,
    ) {
        // Get the attacker's StableId to use as the condition source
        let condition_source =
            if let Some(attacker_stable_id) = world.get::<StableId>(attacker_entity) {
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
        match apply_condition_to_entity(target_entity, burning_condition, world) {
            Ok(()) => {
                // Immediately spawn particle effects for the new condition
                spawn_condition_particles(world);
            }
            Err(err) => {
                eprintln!("Failed to apply burning condition: {}", err);
            }
        }
    }
}
