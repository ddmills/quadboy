use bevy_ecs::prelude::*;
use macroquad::input::KeyCode;

use crate::{
    common::Palette,
    domain::{
        DefaultMeleeAttack, EquipmentSlot, EquipmentSlots, Health, IgnoreLighting, Label, Level,
        Player, StatType, Stats, Weapon, WeaponFamily, WeaponType, Zone,
    },
    engine::{KeyInput, Mouse, StableId, StableIdRegistry},
    rendering::{
        AnimatedGlyph, Glyph, Layer, Position, Text, Visibility, world_to_zone_idx,
        world_to_zone_local,
    },
};

const DEFAULT_WEAPON_RANGE: usize = 12;

#[derive(Resource)]
pub struct TargetCycling {
    pub targets: Vec<(Entity, (f32, f32, f32), f32)>, // Entity, position, distance
    pub current_index: Option<usize>,
    pub current_selected_entity: Option<Entity>,
}

#[derive(Component)]
pub struct TargetCrosshair;

#[derive(Component)]
pub struct TargetInfo;

#[derive(Component)]
pub struct TargetIndicator;

pub fn init_targeting_resource(cmds: &mut Commands) {
    cmds.insert_resource(TargetCycling {
        targets: Vec::new(),
        current_index: None,
        current_selected_entity: None,
    });
}

pub fn spawn_targeting_ui(cmds: &mut Commands, cleanup_marker: impl Component + Clone) {
    // Spawn target crosshair (hidden until target selected)
    cmds.spawn((
        AnimatedGlyph::new(vec![116, 117, 118, 119, 120], 6.0),
        Glyph::new(88, Palette::Yellow, Palette::Yellow).layer(Layer::Overlay),
        Position::new_f32(0., 0., 0.),
        Visibility::Hidden,
        IgnoreLighting,
        TargetCrosshair,
        cleanup_marker.clone(),
    ));

    // Spawn target info display
    cmds.spawn((
        Text::new("")
            .fg1(Palette::White)
            .bg(Palette::Black)
            .layer(Layer::Overlay),
        Position::new_f32(1., 1., 0.),
        Visibility::Hidden,
        TargetInfo,
        IgnoreLighting,
        cleanup_marker.clone(),
    ));

    // Spawn target indicator
    cmds.spawn((
        AnimatedGlyph::new(vec![132, 133, 134, 135, 136], 12.0),
        Glyph::new(94, Palette::Yellow, Palette::Black).layer(Layer::Overlay),
        Position::new_f32(0., 0., 0.),
        Visibility::Hidden,
        IgnoreLighting,
        TargetIndicator,
        cleanup_marker,
    ));
}

pub fn collect_valid_targets(
    mut target_cycling: ResMut<TargetCycling>,
    q_player: Query<(Entity, &Position, Option<&EquipmentSlots>), With<Player>>,
    q_health: Query<(Entity, &Position), With<Health>>,
    q_zones: Query<&Zone>,
    q_weapons: Query<&Weapon>,
    registry: Option<Res<StableIdRegistry>>,
) {
    let Ok((player_entity, player_position, equipment_slots)) = q_player.single() else {
        return;
    };

    // Get weapon range
    let weapon_range =
        if let (Some(equipment), Some(registry)) = (equipment_slots, registry.as_deref()) {
            if let Some(weapon_id) = equipment.get_equipped_item(EquipmentSlot::MainHand) {
                if let Some(weapon_entity) = registry.get_entity(StableId(weapon_id)) {
                    // Check if it's a weapon
                    if let Ok(weapon) = q_weapons.get(weapon_entity) {
                        match weapon.weapon_type {
                            WeaponType::Ranged => weapon.range.unwrap_or(DEFAULT_WEAPON_RANGE),
                            WeaponType::Melee => 1, // Melee weapons have range 1 (adjacent only)
                        }
                    }
                    // If weapon entity exists but has no weapon component
                    else {
                        DEFAULT_WEAPON_RANGE
                    }
                } else {
                    DEFAULT_WEAPON_RANGE
                }
            } else {
                DEFAULT_WEAPON_RANGE // No weapon equipped
            }
        } else {
            DEFAULT_WEAPON_RANGE // No equipment or registry
        };

    let player_world = player_position.world();
    let player_zone_idx = world_to_zone_idx(player_world.0, player_world.1, player_world.2);
    let mut targets = Vec::new();

    // Find the player's zone
    let player_zone = q_zones.iter().find(|z| z.idx == player_zone_idx);
    if let Some(zone) = player_zone {
        // Only check entities in the same zone as the player
        for (entity, pos) in q_health.iter() {
            // Skip self-targeting
            if entity == player_entity {
                continue;
            }

            let target_world = pos.world();
            let target_zone_idx = world_to_zone_idx(target_world.0, target_world.1, target_world.2);

            // Skip if not in the same zone
            if target_zone_idx != player_zone_idx {
                continue;
            }

            // Check if target is visible using Zone.visible grid
            let (local_x, local_y) = world_to_zone_local(target_world.0, target_world.1);
            if !*zone.visible.get(local_x, local_y).unwrap_or(&false) {
                continue;
            }

            // Calculate distance (Manhattan distance)
            let distance = ((target_world.0 as i32 - player_world.0 as i32).abs()
                + (target_world.1 as i32 - player_world.1 as i32).abs())
                as f32;

            // Check if within weapon range
            if distance <= weapon_range as f32 {
                targets.push((
                    entity,
                    (
                        target_world.0 as f32,
                        target_world.1 as f32,
                        target_world.2 as f32,
                    ),
                    distance,
                ));
            }
        }
    }

    // Sort by distance
    targets.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap());

    // Store previous selected entity for comparison
    let prev_selected_entity = target_cycling.current_selected_entity;

    // Update the resource
    target_cycling.targets = targets;

    // Try to maintain selection of the same entity, or auto-advance if it's gone
    if let Some(prev_entity) = prev_selected_entity {
        // Look for the previously selected entity in the new targets list
        if let Some(new_idx) = target_cycling
            .targets
            .iter()
            .position(|(entity, _, _)| *entity == prev_entity)
        {
            // Same entity found, update index
            target_cycling.current_index = Some(new_idx);
            target_cycling.current_selected_entity = Some(prev_entity);
        } else {
            // Previously selected entity is gone - auto-advance to next target
            target_cycling.current_index = if target_cycling.targets.is_empty() {
                None
            } else {
                Some(0) // Go to first (nearest) target
            };
            target_cycling.current_selected_entity =
                target_cycling.targets.first().map(|(entity, _, _)| *entity);
        }
    } else {
        // No previous selection - auto-select nearest target if available
        if !target_cycling.targets.is_empty() {
            target_cycling.current_index = Some(0);
            target_cycling.current_selected_entity =
                target_cycling.targets.first().map(|(entity, _, _)| *entity);
        } else {
            target_cycling.current_index = None;
            target_cycling.current_selected_entity = None;
        }
    }
}

pub fn update_target_cycling(mut target_cycling: ResMut<TargetCycling>, keys: Res<KeyInput>) {
    // Check for C key press to cycle targets
    if keys.is_pressed(KeyCode::C) && !target_cycling.targets.is_empty() {
        // Cycle to next target
        target_cycling.current_index = match target_cycling.current_index {
            None => Some(0), // Start at first (nearest) target
            Some(idx) => Some((idx + 1) % target_cycling.targets.len()),
        };

        // Update selected entity
        if let Some(idx) = target_cycling.current_index
            && let Some((entity, _, _)) = target_cycling.targets.get(idx)
        {
            target_cycling.current_selected_entity = Some(*entity);
        }
    }
}

pub fn update_mouse_targeting(mut target_cycling: ResMut<TargetCycling>, mouse: Res<Mouse>) {
    // Only update targeting when mouse moves and we have targets available
    if mouse.has_moved && !target_cycling.targets.is_empty() {
        let mouse_world = mouse.world;

        // Find the closest target to mouse position
        let mut closest_idx = None;
        let mut closest_distance = f32::INFINITY;

        for (idx, (_entity, target_pos, _target_distance)) in
            target_cycling.targets.iter().enumerate()
        {
            // Calculate distance from mouse to target (Euclidean distance)
            let dx = target_pos.0 - mouse_world.0;
            let dy = target_pos.1 - mouse_world.1;
            let distance = (dx * dx + dy * dy).sqrt();

            if distance < closest_distance {
                closest_distance = distance;
                closest_idx = Some(idx);
            }
        }

        // Update targeting to closest target
        if let Some(idx) = closest_idx {
            target_cycling.current_index = Some(idx);
            if let Some((entity, _, _)) = target_cycling.targets.get(idx) {
                target_cycling.current_selected_entity = Some(*entity);
            }
        }
    }
}

pub fn calculate_hit_chance(
    attacker_entity: Entity,
    target_entity: Entity,
    q_stats: &Query<&Stats>,
    q_equipment: &Query<&EquipmentSlots>,
    q_weapons: &Query<&Weapon>,
    q_default_attacks: &Query<&DefaultMeleeAttack>,
    registry: &StableIdRegistry,
) -> i32 {
    // Get target's dodge stat
    let target_dodge = q_stats
        .get(target_entity)
        .map(|stats| stats.get_stat(StatType::Dodge))
        .unwrap_or(0);

    // Determine weapon family for attacker (same logic as resolve_hit_miss)
    let weapon_family = {
        // First try to get equipped weapon
        if let Ok(equipment) = q_equipment.get(attacker_entity)
            && let Some(weapon_id) = equipment.get_equipped_item(EquipmentSlot::MainHand)
            && let Some(weapon_entity) = registry.get_entity(StableId(weapon_id))
        {
            // Check if it's a weapon
            if let Ok(weapon) = q_weapons.get(weapon_entity) {
                weapon.weapon_family
            } else {
                WeaponFamily::Unarmed
            }
        }
        // Fall back to default melee attack
        else if let Ok(default_attack) = q_default_attacks.get(attacker_entity) {
            default_attack.weapon.weapon_family
        }
        // Default to unarmed if no weapon or default attack
        else {
            WeaponFamily::Unarmed
        }
    };

    // Get attacker's weapon proficiency stat
    let weapon_proficiency = q_stats
        .get(attacker_entity)
        .map(|stats| stats.get_stat(weapon_family.to_stat_type()))
        .unwrap_or(0);

    // Calculate hit chance: P(attack_roll + proficiency >= defense_roll + dodge) OR critical hit
    let mut hit_count = 0;
    for attack_roll in 1..=12 {
        let attacker_total = attack_roll + weapon_proficiency;
        for defense_roll in 1..=12 {
            let defense_total = defense_roll + target_dodge;
            // Hit if critical (natural 12) or if attack total >= defense total
            if attack_roll == 12 || attacker_total >= defense_total {
                hit_count += 1;
            }
        }
    }

    // Return percentage (out of 144 total combinations)
    (hit_count * 100) / 144
}

pub fn render_target_crosshair(
    target_cycling: Res<TargetCycling>,
    mut q_crosshair: Query<(&mut Position, &mut Visibility), With<TargetCrosshair>>,
) {
    if let Ok((mut crosshair_pos, mut crosshair_visibility)) = q_crosshair.single_mut() {
        if let Some(idx) = target_cycling.current_index {
            if let Some((_entity, pos, _dist)) = target_cycling.targets.get(idx) {
                crosshair_pos.x = pos.0.floor();
                crosshair_pos.y = pos.1.floor();
                crosshair_pos.z = pos.2.floor();
                *crosshair_visibility = Visibility::Visible;
            } else {
                *crosshair_visibility = Visibility::Hidden;
            }
        } else {
            *crosshair_visibility = Visibility::Hidden;
        }
    }
}

pub fn render_target_info(
    target_cycling: Res<TargetCycling>,
    q_zones: Query<&Zone>,
    q_health: Query<&Health>,
    q_names: Query<&Label>,
    mut q_target_info: Query<(&mut Text, &mut Position, &mut Visibility), With<TargetInfo>>,
    mut q_target_indicator: Query<
        (&mut Position, &mut Visibility),
        (With<TargetIndicator>, Without<TargetInfo>),
    >,
) {
    let Ok((mut text, mut text_pos, mut text_visibility)) = q_target_info.single_mut() else {
        return;
    };

    let Ok((mut indicator_pos, mut indicator_visibility)) = q_target_indicator.single_mut() else {
        return;
    };

    if let Some(idx) = target_cycling.current_index {
        if let Some((entity, pos, _dist)) = target_cycling.targets.get(idx) {
            let target_x = pos.0.floor() as usize;
            let target_y = pos.1.floor() as usize;
            let target_z = pos.2 as usize;

            let zone_idx = world_to_zone_idx(target_x, target_y, target_z);
            let (local_x, local_y) = world_to_zone_local(target_x, target_y);

            let Some(zone) = q_zones.iter().find(|z| z.idx == zone_idx) else {
                *text_visibility = Visibility::Hidden;
                *indicator_visibility = Visibility::Hidden;
                return;
            };

            let Some(entities) = zone.entities.get(local_x, local_y) else {
                *text_visibility = Visibility::Hidden;
                *indicator_visibility = Visibility::Hidden;
                return;
            };

            // Find first valid target (entity with Health)
            let mut target_name = None;
            let mut target_health = None;

            for entity_at_pos in entities {
                if *entity_at_pos == *entity
                    && let Ok(health) = q_health.get(*entity)
                {
                    target_health = Some(health);
                    target_name = Some(if let Ok(label) = q_names.get(*entity) {
                        label.get().to_string()
                    } else {
                        "Unknown".to_string()
                    });
                    break;
                }
            }

            if let (Some(name), Some(_health)) = (target_name, target_health) {
                // Show and update target info text (just the name)
                *text_visibility = Visibility::Visible;

                text.value = name;
                text_pos.x = pos.0.floor() + 1.;
                text_pos.y = pos.1.floor();
                text_pos.z = pos.2.floor();

                // Show and position target indicator above the target
                *indicator_visibility = Visibility::Hidden;
                indicator_pos.x = pos.0.floor();
                indicator_pos.y = pos.1.floor() - 1.0;
                indicator_pos.z = pos.2.floor();
            } else {
                *text_visibility = Visibility::Hidden;
                *indicator_visibility = Visibility::Hidden;
            }
        } else {
            *text_visibility = Visibility::Hidden;
            *indicator_visibility = Visibility::Hidden;
        }
    } else {
        *text_visibility = Visibility::Hidden;
        *indicator_visibility = Visibility::Hidden;
    }
}
