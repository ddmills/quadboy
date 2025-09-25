use std::collections::HashMap;

use bevy_ecs::prelude::*;
use macroquad::prelude::trace;

use crate::{
    cfg::WORLD_SIZE,
    common::algorithm::{
        astar::{AStarSettings, astar},
        distance::Distance,
    },
    domain::{
        AiContext, AiController, AttackAction, DefaultRangedAttack, Energy, EnergyActionType,
        EquipmentSlot, EquipmentSlots, MoveAction, MovementCapabilities, StairDown, StairUp,
        Weapon, WeaponType, Zone, get_base_energy_cost,
    },
    engine::{StableId, StableIdRegistry},
    rendering::{Position, world_to_zone_idx, world_to_zone_local, zone_local_to_world, zone_xyz},
};
use macroquad::rand;

pub fn ai_try_use_stair(world: &mut World, entity: Entity, going_down: bool) -> bool {
    let Some(position) = world.get::<Position>(entity) else {
        return false;
    };

    let pos_world = position.world();
    let entity_zone_idx = world_to_zone_idx(pos_world.0, pos_world.1, pos_world.2);
    let entity_local = world_to_zone_local(pos_world.0, pos_world.1);

    let mut zone_cache: HashMap<usize, &Zone> = HashMap::new();
    for zone in world.query::<&Zone>().iter(world) {
        zone_cache.insert(zone.idx, zone);
    }

    let Some(current_zone) = zone_cache.get(&entity_zone_idx) else {
        return false;
    };

    let Some(entities_at_pos) = current_zone.entities.get(entity_local.0, entity_local.1) else {
        return false;
    };

    for &stair_entity in entities_at_pos.iter() {
        let has_correct_stair = if going_down {
            world.get::<StairDown>(stair_entity).is_some()
        } else {
            world.get::<StairUp>(stair_entity).is_some()
        };

        if has_correct_stair {
            let new_z = if going_down {
                pos_world.2 + 1
            } else {
                pos_world.2.saturating_sub(1)
            };

            let action = MoveAction {
                entity,
                new_position: (pos_world.0, pos_world.1, new_z),
            };
            action.apply(world);
            return true;
        }
    }

    false
}

pub fn ai_try_navigate_to_stair(
    world: &mut World,
    entity: Entity,
    current_z: i32,
    target_z: i32,
) -> bool {
    let Some(position) = world.get::<Position>(entity) else {
        return false;
    };

    let pos_world = position.world();
    let going_down = current_z < target_z;
    let mut nearest_stair_pos: Option<(i32, i32)> = None;
    let mut nearest_distance = f32::INFINITY;

    for zone in world.query::<&Zone>().iter(world) {
        let (_, _, zone_z) = zone_xyz(zone.idx);
        if zone_z != current_z as usize {
            continue;
        }

        for y in 0..zone.entities.height() {
            for x in 0..zone.entities.width() {
                let Some(entities) = zone.entities.get(x, y) else {
                    continue;
                };

                for &stair_entity in entities.iter() {
                    let should_target = if going_down {
                        world.get::<StairDown>(stair_entity).is_some()
                    } else {
                        world.get::<StairUp>(stair_entity).is_some()
                    };

                    if should_target {
                        let (world_x, world_y, world_z) = zone_local_to_world(zone.idx, x, y);

                        let distance = Distance::diagonal(
                            [pos_world.0 as i32, pos_world.1 as i32, current_z],
                            [world_x as i32, world_y as i32, world_z as i32],
                        );

                        if distance < nearest_distance {
                            nearest_distance = distance;
                            nearest_stair_pos = Some((world_x as i32, world_y as i32));
                        }
                    }
                }
            }
        }
    }

    if let Some((stair_x, stair_y)) = nearest_stair_pos {
        return ai_try_move_toward(
            world,
            entity,
            (stair_x as usize, stair_y as usize, current_z as usize),
        );
    }

    false
}

pub fn ai_try_attacking_nearby(world: &mut World, entity: Entity, context: &mut AiContext) -> bool {
    let Some(nearest) = context.nearest_hostile() else {
        return false;
    };

    // what is 'entity' attack range?
    if nearest.distance > 1.75 {
        return false;
    }

    let Some(stable_id) = world.get::<StableId>(entity) else {
        return false;
    };

    let attack = AttackAction {
        attacker_stable_id: *stable_id,
        weapon_stable_id: None, // Use equipped weapon or default attack
        target_stable_id: nearest.stable_id,
        is_bump_attack: true,
    };

    attack.apply(world);
    true
}

pub fn ai_try_ranged_attack(world: &mut World, entity: Entity, context: &mut AiContext) -> bool {
    // Check if AI has a target
    let Some(target) = context.target else {
        return false;
    };

    // Skip ranged attack if target is adjacent (prefer melee at close range)
    if target.distance <= 1.5 {
        return false;
    }

    // Get AI's stable ID
    let Some(stable_id) = world.get::<StableId>(entity) else {
        return false;
    };

    // Check if AI has a ranged weapon equipped
    if let Some(equipment) = world.get::<EquipmentSlots>(entity) {
        if let Some(weapon_id) = equipment.get_equipped_item(EquipmentSlot::MainHand) {
            let registry = world.resource::<StableIdRegistry>();
            if let Some(weapon_entity) = registry.get_entity(StableId(weapon_id)) {
                if let Some(weapon) = world.get::<Weapon>(weapon_entity) {
                    // Only proceed if weapon is ranged
                    if weapon.weapon_type == WeaponType::Ranged {
                        // Check if weapon has ammo
                        if let Some(ammo) = weapon.current_ammo {
                            if ammo == 0 {
                                return false; // No ammo
                            }
                        }

                        // Check if target is within range
                        if let Some(range) = weapon.range {
                            let Some(ai_pos) = world.get::<Position>(entity) else {
                                return false;
                            };

                            let ai_world_pos = ai_pos.world();
                            let distance = Distance::diagonal(
                                [
                                    ai_world_pos.0 as i32,
                                    ai_world_pos.1 as i32,
                                    ai_world_pos.2 as i32,
                                ],
                                [
                                    target.pos.0 as i32,
                                    target.pos.1 as i32,
                                    target.pos.2 as i32,
                                ],
                            ) as usize;

                            if distance <= range {
                                // Perform ranged attack with equipped weapon
                                let attack = AttackAction {
                                    attacker_stable_id: *stable_id,
                                    weapon_stable_id: Some(StableId(weapon_id)),
                                    target_stable_id: target.stable_id,
                                    is_bump_attack: false,
                                };

                                attack.apply(world);
                                return true;
                            }
                        }
                    }
                }
            }
        }
    }

    // Check if AI has a default ranged attack
    if let Some(default_ranged) = world.get::<DefaultRangedAttack>(entity) {
        // Check if default ranged attack has ammo
        if !default_ranged.has_ammo() {
            return false; // No ammo
        }

        // Check if target is within range
        let Some(ai_pos) = world.get::<Position>(entity) else {
            return false;
        };

        let ai_world_pos = ai_pos.world();
        let distance = Distance::diagonal(
            [
                ai_world_pos.0 as i32,
                ai_world_pos.1 as i32,
                ai_world_pos.2 as i32,
            ],
            [
                target.pos.0 as i32,
                target.pos.1 as i32,
                target.pos.2 as i32,
            ],
        ) as usize;

        if let Some(range) = default_ranged.weapon.range {
            if distance <= range {
                // Perform ranged attack with default ranged attack
                let attack = AttackAction {
                    attacker_stable_id: *stable_id,
                    weapon_stable_id: None, // No specific weapon, use default
                    target_stable_id: target.stable_id,
                    is_bump_attack: false,
                };

                attack.apply(world);
                return true;
            }
        }
    }

    false
}

pub fn ai_try_select_target(_world: &mut World, _entity: Entity, context: &mut AiContext) -> bool {
    if let Some(t) = context.target {
        context.target = Some(t);
        return true;
    }

    let Some(nearest) = context.nearest_hostile() else {
        return false;
    };

    context.target = Some(*nearest);
    true
}

pub fn ai_try_move_toward_target(
    world: &mut World,
    entity: Entity,
    context: &mut AiContext,
) -> bool {
    let Some(target) = context.target else {
        return false;
    };

    ai_try_move_toward(world, entity, target.pos)
}

pub fn ai_try_move_toward(
    world: &mut World,
    entity: Entity,
    target_pos: (usize, usize, usize),
) -> bool {
    let mut zone_cache: HashMap<usize, &Zone> = HashMap::new();

    let (start_x, start_y, start_z) = {
        let Some(position) = world.get::<Position>(entity) else {
            return false;
        };

        let pos_world = position.world();

        (pos_world.0 as i32, pos_world.1 as i32, pos_world.2 as i32)
    };

    let (target_x, target_y, target_z) = (
        target_pos.0 as i32,
        target_pos.1 as i32,
        target_pos.2 as i32,
    );

    // check if on different Z levels!
    if start_z != target_z {
        let going_down = start_z < target_z;

        // First try to use a stair we're already on
        if ai_try_use_stair(world, entity, going_down) {
            return true;
        }

        // If that failed, try to navigate to the nearest appropriate stair
        return ai_try_navigate_to_stair(world, entity, start_z, target_z);
    }

    let movement_flags = world
        .get::<MovementCapabilities>(entity)
        .unwrap_or(&MovementCapabilities::terrestrial())
        .flags;

    for zone in world.query::<&Zone>().iter(world) {
        zone_cache.insert(zone.idx, zone);
    }

    let deltas = [
        (-1, -1),
        (0, -1),
        (1, -1),
        (-1, 0),
        (1, 0),
        (-1, 1),
        (0, 1),
        (1, 1),
    ];

    let mut result = astar(AStarSettings {
        start: (start_x, start_y),
        is_goal: |(x, y)| x == target_x && y == target_y,
        cost: |(from_x, from_y), (to_x, to_y)| {
            if to_x < 0
                || to_y < 0
                || to_x as usize >= WORLD_SIZE.0
                || to_y as usize >= WORLD_SIZE.1
            {
                return f32::INFINITY;
            }

            let to_zone_idx = world_to_zone_idx(to_x as usize, to_y as usize, target_z as usize);
            let Some(zone) = zone_cache.get(&to_zone_idx) else {
                trace!(
                    "zone not in cache {},{},{} = {}",
                    to_x, to_y, target_z, to_zone_idx
                );
                return f32::INFINITY;
            };

            // Convert to local zone coordinates
            let local = world_to_zone_local(to_x as usize, to_y as usize);

            // Get cached collider flags at target position
            let collider_flags = zone.colliders.get_flags(local.0, local.1);

            // Check if movement is blocked
            if movement_flags.is_blocked_by(collider_flags) {
                return f32::INFINITY;
            }

            Distance::diagonal([from_x, from_y, target_z], [to_x, to_y, target_z])
        },
        heuristic: |(to_x, to_y)| {
            Distance::diagonal([start_x, start_y, start_z], [to_x, to_y, target_z])
        },
        neighbors: |(x, y)| deltas.iter().map(|(dx, dy)| (x + dx, y + dy)).collect(),
        max_depth: 1000,
        max_cost: Some(100.),
    });

    // last element is the starting position
    result.path.pop();

    if !result.is_success || result.path.is_empty() {
        trace!("A* FAILED");
        return false;
    }

    let Some(move_to_target) = result.path.last() else {
        return false;
    };

    let action = MoveAction {
        entity,
        new_position: (
            move_to_target.0 as usize,
            move_to_target.1 as usize,
            start_z as usize,
        ),
    };

    action.apply(world);
    true
}

pub fn ai_try_flee_from(
    world: &mut World,
    entity: Entity,
    flee_from_pos: (usize, usize, usize),
) -> bool {
    let Some(position) = world.get::<Position>(entity) else {
        return false;
    };

    let current_pos = position.world();
    let movement_flags = world
        .get::<MovementCapabilities>(entity)
        .unwrap_or(&MovementCapabilities::terrestrial())
        .flags;

    // Calculate all possible moves and pick the one that maximizes distance from the flee target
    let deltas = [
        (-1, -1),
        (0, -1),
        (1, -1),
        (-1, 0),
        (1, 0),
        (-1, 1),
        (0, 1),
        (1, 1),
    ];

    let mut best_position: Option<(usize, usize, usize)> = None;
    let mut best_distance = 0.0;

    for (dx, dy) in deltas.iter() {
        let new_x = current_pos.0 as i32 + dx;
        let new_y = current_pos.1 as i32 + dy;

        if new_x < 0
            || new_y < 0
            || new_x as usize >= WORLD_SIZE.0
            || new_y as usize >= WORLD_SIZE.1
        {
            continue;
        }

        let new_pos = (new_x as usize, new_y as usize, current_pos.2);

        // Check if position is blocked
        let zone_idx = world_to_zone_idx(new_pos.0, new_pos.1, new_pos.2);
        let mut zone_query = world.query::<&Zone>();
        if let Some(zone) = zone_query.iter(world).find(|z| z.idx == zone_idx) {
            let local = world_to_zone_local(new_pos.0, new_pos.1);
            let collider_flags = zone.colliders.get_flags(local.0, local.1);

            if movement_flags.is_blocked_by(collider_flags) {
                continue;
            }
        } else {
            continue;
        }

        // Calculate distance from flee target
        let distance = Distance::diagonal(
            [new_pos.0 as i32, new_pos.1 as i32, new_pos.2 as i32],
            [
                flee_from_pos.0 as i32,
                flee_from_pos.1 as i32,
                flee_from_pos.2 as i32,
            ],
        );

        if distance > best_distance {
            best_distance = distance;
            best_position = Some(new_pos);
        }
    }

    if let Some(flee_to) = best_position {
        let action = MoveAction {
            entity,
            new_position: flee_to,
        };
        action.apply(world);
        return true;
    }

    false
}

pub fn ai_try_random_move(world: &mut World, entity: Entity) -> bool {
    let Some(position) = world.get::<Position>(entity) else {
        return false;
    };

    let current_pos = position.world();
    let current_zone_idx = world_to_zone_idx(current_pos.0, current_pos.1, current_pos.2);
    let movement_flags = world
        .get::<MovementCapabilities>(entity)
        .unwrap_or(&MovementCapabilities::terrestrial())
        .flags;

    // Get all valid adjacent positions
    let deltas = [
        (-1, -1),
        (0, -1),
        (1, -1),
        (-1, 0),
        (1, 0),
        (-1, 1),
        (0, 1),
        (1, 1),
    ];

    let mut valid_positions = Vec::new();

    for (dx, dy) in deltas.iter() {
        let new_x = current_pos.0 as i32 + dx;
        let new_y = current_pos.1 as i32 + dy;

        if new_x < 0
            || new_y < 0
            || new_x as usize >= WORLD_SIZE.0
            || new_y as usize >= WORLD_SIZE.1
        {
            continue;
        }

        let new_pos = (new_x as usize, new_y as usize, current_pos.2);

        // Check if new position would cross zone boundary
        let zone_idx = world_to_zone_idx(new_pos.0, new_pos.1, new_pos.2);
        if zone_idx != current_zone_idx {
            continue;
        }

        // Check if position is blocked
        let mut zone_query = world.query::<&Zone>();
        if let Some(zone) = zone_query.iter(world).find(|z| z.idx == zone_idx) {
            let local = world_to_zone_local(new_pos.0, new_pos.1);
            let collider_flags = zone.colliders.get_flags(local.0, local.1);

            if !movement_flags.is_blocked_by(collider_flags) {
                valid_positions.push(new_pos);
            }
        }
    }

    if valid_positions.is_empty() {
        return false;
    }

    // Pick a random valid position
    let random_index =
        (rand::gen_range(0.0, 1.0) * valid_positions.len() as f32) as usize % valid_positions.len();
    let chosen_pos = valid_positions[random_index];

    let action = MoveAction {
        entity,
        new_position: chosen_pos,
    };
    action.apply(world);
    true
}

pub fn ai_try_wander(world: &mut World, entity: Entity) -> bool {
    let Some(position) = world.get::<Position>(entity) else {
        return false;
    };

    let Some(ai_controller) = world.get::<AiController>(entity) else {
        return false;
    };

    let current_pos = position.world();
    let home_pos = ai_controller.home_position;
    let wander_range = ai_controller.wander_range;
    let current_zone_idx = world_to_zone_idx(current_pos.0, current_pos.1, current_pos.2);

    let movement_flags = world
        .get::<MovementCapabilities>(entity)
        .unwrap_or(&MovementCapabilities::terrestrial())
        .flags;

    // Get all valid adjacent positions within wander range from home
    let deltas = [
        (-1, -1),
        (0, -1),
        (1, -1),
        (-1, 0),
        (1, 0),
        (-1, 1),
        (0, 1),
        (1, 1),
    ];

    let mut valid_positions = Vec::new();

    for (dx, dy) in deltas.iter() {
        let new_x = current_pos.0 as i32 + dx;
        let new_y = current_pos.1 as i32 + dy;

        if new_x < 0
            || new_y < 0
            || new_x as usize >= WORLD_SIZE.0
            || new_y as usize >= WORLD_SIZE.1
        {
            continue;
        }

        let new_pos = (new_x as usize, new_y as usize, current_pos.2);

        // Check if new position is within wander range from home
        let distance_from_home = Distance::diagonal(
            [home_pos.0 as i32, home_pos.1 as i32, home_pos.2 as i32],
            [new_pos.0 as i32, new_pos.1 as i32, new_pos.2 as i32],
        );

        if distance_from_home > wander_range as f32 {
            continue;
        }

        // Check if new position would cross zone boundary
        let zone_idx = world_to_zone_idx(new_pos.0, new_pos.1, new_pos.2);
        if zone_idx != current_zone_idx {
            continue;
        }

        // Check if position is blocked
        let mut zone_query = world.query::<&Zone>();
        if let Some(zone) = zone_query.iter(world).find(|z| z.idx == zone_idx) {
            let local = world_to_zone_local(new_pos.0, new_pos.1);
            let collider_flags = zone.colliders.get_flags(local.0, local.1);

            if !movement_flags.is_blocked_by(collider_flags) {
                valid_positions.push(new_pos);
            }
        }
    }

    if valid_positions.is_empty() {
        return false;
    }

    // Pick a random valid position
    let random_index =
        (rand::gen_range(0.0, 1.0) * valid_positions.len() as f32) as usize % valid_positions.len();
    let chosen_pos = valid_positions[random_index];

    let action = MoveAction {
        entity,
        new_position: chosen_pos,
    };
    action.apply(world);

    // Consume energy for wandering (same as movement)
    if let Some(mut energy) = world.get_mut::<Energy>(entity) {
        let cost = get_base_energy_cost(EnergyActionType::Move);
        energy.consume_energy(cost);
    }

    true
}
