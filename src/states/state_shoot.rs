use bevy_ecs::{prelude::*, system::SystemId};
use macroquad::{input::KeyCode, prelude::trace};
use serde::{Deserialize, Serialize};

use crate::{
    common::Palette,
    domain::{
        EquipmentSlot, EquipmentSlots, Health, IgnoreLighting, Player, PlayerPosition, ShootAction,
        TurnState, Zone, game_loop,
    },
    engine::{App, KeyInput, Mouse, Plugin, SerializableComponent, StableIdRegistry},
    rendering::{
        AnimatedGlyph, Glyph, Layer, Position, Text, Visibility, world_to_zone_idx,
        world_to_zone_local,
    },
    states::{CurrentGameState, GameStatePlugin, cleanup_system},
};

use super::GameState as GS;

#[derive(Resource)]
struct ShootCallbacks {
    back_to_explore: SystemId,
}

#[derive(Resource)]
struct TargetCycling {
    targets: Vec<(Entity, (f32, f32, f32), f32)>, // Entity, position, distance
    current_index: Option<usize>,
    current_target_pos: (f32, f32, f32),
    keyboard_focus: bool,
}

pub struct ShootStatePlugin;

impl Plugin for ShootStatePlugin {
    fn build(&self, app: &mut App) {
        GameStatePlugin::new(GS::Shoot)
            .on_enter(app, (setup_callbacks, on_enter_shoot).chain())
            .on_update(
                app,
                (
                    collect_valid_targets,
                    update_target_cycling,
                    render_crosshair,
                    render_target_info,
                    handle_shoot_input,
                    game_loop,
                ),
            )
            .on_leave(
                app,
                (
                    on_leave_shoot,
                    cleanup_system::<CleanupStateShoot>,
                    remove_shoot_callbacks,
                    remove_target_cycling,
                )
                    .chain(),
            );
    }
}

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct CleanupStateShoot;

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct ShootCrosshair;

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct ShootTargetInfo;

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct ShootTargetIndicator;

fn setup_callbacks(world: &mut World) {
    let callbacks = ShootCallbacks {
        back_to_explore: world.register_system(back_to_explore),
    };

    world.insert_resource(callbacks);
}

fn back_to_explore(mut game_state: ResMut<CurrentGameState>) {
    game_state.next = GS::Explore;
}

fn remove_shoot_callbacks(mut cmds: Commands) {
    cmds.remove_resource::<ShootCallbacks>();
}

fn remove_target_cycling(mut cmds: Commands) {
    cmds.remove_resource::<TargetCycling>();
}

fn on_enter_shoot(
    mut cmds: Commands,
    _callbacks: Res<ShootCallbacks>,
    mouse: Res<Mouse>,
    player_pos: Res<PlayerPosition>,
) {
    trace!("EnterGameState::<Shoot>");

    // Initialize target cycling resource
    cmds.insert_resource(TargetCycling {
        targets: Vec::new(),
        current_index: None,
        current_target_pos: (mouse.world.0, mouse.world.1, player_pos.z),
        keyboard_focus: false,
    });

    // Spawn crosshair
    cmds.spawn((
        Glyph::new(21, Palette::Yellow, Palette::Yellow).layer(Layer::Overlay),
        AnimatedGlyph::new(vec![116, 118, 118, 119, 118], 11.0),
        Position::new_f32(0., 0., 0.),
        Visibility::Visible,
        IgnoreLighting,
        ShootCrosshair,
        CleanupStateShoot,
    ));

    // Spawn target info display
    cmds.spawn((
        Text::new("")
            .fg1(Palette::White)
            .bg(Palette::Black)
            .layer(Layer::Overlay),
        Position::new_f32(1., 1., 0.),
        Visibility::Hidden,
        ShootTargetInfo,
        IgnoreLighting,
        CleanupStateShoot,
    ));

    // Spawn target indicator (animated glyph that appears above targets)
    cmds.spawn((
        Glyph::new(132, Palette::Yellow, Palette::Yellow).layer(Layer::Overlay),
        AnimatedGlyph::new(vec![132, 133, 134, 135, 136], 11.0),
        Position::new_f32(0., 0., 0.),
        Visibility::Hidden,
        ShootTargetIndicator,
        IgnoreLighting,
        CleanupStateShoot,
    ));
}

fn on_leave_shoot(_cmds: Commands) {
    trace!("LeaveGameState::<Shoot>");
}

fn collect_valid_targets(
    mut target_cycling: ResMut<TargetCycling>,
    q_player: Query<(&Position, Option<&EquipmentSlots>), With<Player>>,
    q_health: Query<(Entity, &Position), With<Health>>,
    q_zones: Query<&Zone>,
    registry: Option<Res<StableIdRegistry>>,
    _player_pos: Res<PlayerPosition>,
) {
    let Ok((player_position, equipment_slots)) = q_player.single() else {
        return;
    };

    // Get weapon range
    let weapon_range =
        if let (Some(equipment), Some(registry)) = (equipment_slots, registry.as_deref()) {
            if let Some(weapon_id) = equipment.get_equipped_item(EquipmentSlot::MainHand) {
                if let Some(_weapon_entity) = registry.get_entity(weapon_id) {
                    // Need to access world to get RangedWeapon component - this is tricky in a system
                    // For now, use a default range of 12
                    12
                } else {
                    12
                }
            } else {
                12
            }
        } else {
            12
        };

    let player_world = player_position.world();
    let player_zone_idx = world_to_zone_idx(player_world.0, player_world.1, player_world.2);
    let mut targets = Vec::new();

    // Find the player's zone
    let player_zone = q_zones.iter().find(|z| z.idx == player_zone_idx);
    if let Some(zone) = player_zone {
        // Only check entities in the same zone as the player
        for (entity, pos) in q_health.iter() {
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

            // Calculate distance (Manhattan distance for now)
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
    let prev_selected_entity = target_cycling
        .current_index
        .and_then(|idx| target_cycling.targets.get(idx))
        .map(|(entity, _, _)| *entity);

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
        } else if target_cycling.keyboard_focus {
            // Previously selected entity is gone and we're in keyboard mode
            // Auto-advance to next target or wrap to first
            target_cycling.current_index = if target_cycling.targets.is_empty() {
                None
            } else {
                Some(0) // Go to first (nearest) target
            };
        }
    } else {
        // No previous selection, reset if list changed
        if let Some(current_idx) = target_cycling.current_index {
            if current_idx >= target_cycling.targets.len() {
                target_cycling.current_index = if target_cycling.targets.is_empty() {
                    None
                } else {
                    Some(0)
                };
            }
        }
    }

    // Update current_target_pos if we have a valid selection in keyboard mode
    if target_cycling.keyboard_focus {
        if let Some(idx) = target_cycling.current_index {
            if let Some((_entity, pos, _dist)) = target_cycling.targets.get(idx) {
                target_cycling.current_target_pos = *pos;
            }
        }
    }
}

fn update_target_cycling(
    mut target_cycling: ResMut<TargetCycling>,
    mouse: Res<Mouse>,
    keys: Res<KeyInput>,
    player_pos: Res<PlayerPosition>,
) {
    // Check for TAB key press to cycle targets (forward)
    if keys.is_pressed(KeyCode::Tab)
        && !keys.is_down(KeyCode::LeftShift)
        && !keys.is_down(KeyCode::RightShift)
    {
        if !target_cycling.targets.is_empty() {
            // Enter keyboard focus mode
            target_cycling.keyboard_focus = true;

            // Cycle to next target - if no target selected, start at first
            target_cycling.current_index = match target_cycling.current_index {
                None => Some(0), // Start at first (nearest) target
                Some(idx) => Some((idx + 1) % target_cycling.targets.len()),
            };

            // Update current_target_pos to the selected target
            if let Some(idx) = target_cycling.current_index {
                if let Some((_entity, pos, _dist)) = target_cycling.targets.get(idx) {
                    target_cycling.current_target_pos = *pos;
                }
            }
        }
        return;
    }

    // Check for Shift+TAB key press to cycle targets (reverse)
    if keys.is_pressed(KeyCode::Tab)
        && (keys.is_down(KeyCode::LeftShift) || keys.is_down(KeyCode::RightShift))
    {
        if !target_cycling.targets.is_empty() {
            // Enter keyboard focus mode
            target_cycling.keyboard_focus = true;

            // Cycle to previous target - if no target selected, start at first (nearest) target
            target_cycling.current_index = match target_cycling.current_index {
                None => Some(0),                                   // Start at first (nearest) target
                Some(0) => Some(target_cycling.targets.len() - 1), // Wrap to last
                Some(idx) => Some(idx - 1),                        // Go to previous
            };

            // Update current_target_pos to the selected target
            if let Some(idx) = target_cycling.current_index {
                if let Some((_entity, pos, _dist)) = target_cycling.targets.get(idx) {
                    target_cycling.current_target_pos = *pos;
                }
            }
        }
        return;
    }

    // Check if mouse has moved
    if mouse.has_moved && target_cycling.keyboard_focus {
        // Break keyboard focus
        target_cycling.keyboard_focus = false;
        target_cycling.current_index = None;
    }

    // Update target position based on current mode
    if !target_cycling.keyboard_focus {
        // Use mouse position
        target_cycling.current_target_pos = (mouse.world.0, mouse.world.1, player_pos.z);
    }
    // If in keyboard focus, current_target_pos is already set by TAB cycling or collect_valid_targets
}

fn render_crosshair(
    target_cycling: Res<TargetCycling>,
    mut q_crosshair: Query<&mut Position, With<ShootCrosshair>>,
) {
    if let Ok(mut crosshair_pos) = q_crosshair.single_mut() {
        crosshair_pos.x = target_cycling.current_target_pos.0.floor();
        crosshair_pos.y = target_cycling.current_target_pos.1.floor();
        crosshair_pos.z = target_cycling.current_target_pos.2.floor();
    }
}

fn render_target_info(
    target_cycling: Res<TargetCycling>,
    q_zones: Query<&Zone>,
    q_health: Query<&Health>,
    q_names: Query<&crate::domain::Label>,
    mut q_target_info: Query<(&mut Text, &mut Position, &mut Visibility), With<ShootTargetInfo>>,
    mut q_target_indicator: Query<
        (&mut Position, &mut Visibility),
        (With<ShootTargetIndicator>, Without<ShootTargetInfo>),
    >,
) {
    let target_x = target_cycling.current_target_pos.0.floor() as usize;
    let target_y = target_cycling.current_target_pos.1.floor() as usize;
    let target_z = target_cycling.current_target_pos.2 as usize;

    let zone_idx = world_to_zone_idx(target_x, target_y, target_z);
    let (local_x, local_y) = world_to_zone_local(target_x, target_y);

    let Ok((mut text, mut text_pos, mut text_visibility)) = q_target_info.single_mut() else {
        return;
    };

    let Ok((mut indicator_pos, mut indicator_visibility)) = q_target_indicator.single_mut() else {
        return;
    };

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

    for entity in entities {
        if let Ok(health) = q_health.get(*entity) {
            target_health = Some(health);
            if let Ok(label) = q_names.get(*entity) {
                target_name = Some(label.get());
            }
            break;
        }
    }

    if let (Some(name), Some(health)) = (target_name, target_health) {
        // Show and update target info text
        *text_visibility = Visibility::Visible;
        text.value = format!("{} ({}/{})", name, health.current, health.max);
        text.fg1 = if health.current <= health.max / 4 {
            Some(Palette::Red.into())
        } else if health.current <= health.max / 2 {
            Some(Palette::Yellow.into())
        } else {
            Some(Palette::Green.into())
        };
        text_pos.x = target_x as f32 + 1.0;
        text_pos.y = target_y as f32;
        text_pos.z = target_z as f32;

        // Show and position target indicator above the target
        *indicator_visibility = Visibility::Visible;
        indicator_pos.x = target_x as f32;
        indicator_pos.y = target_y as f32 - 1.0; // 1 tile above the target
        indicator_pos.z = target_z as f32;
    } else {
        // Hide both target info and indicator when not over a valid target
        *text_visibility = Visibility::Hidden;
        *indicator_visibility = Visibility::Hidden;
    }
}

fn handle_shoot_input(
    mut cmds: Commands,
    keys: Res<KeyInput>,
    mouse: Res<Mouse>,
    mut game_state: ResMut<CurrentGameState>,
    turn_state: Res<TurnState>,
    q_player: Query<(Entity, &Position, Option<&EquipmentSlots>), With<Player>>,
    q_zones: Query<&Zone>,
    q_health: Query<&Health>,
    target_cycling: Res<TargetCycling>,
    registry: Option<Res<StableIdRegistry>>,
) {
    // ESC or right-click to cancel
    if keys.is_pressed(KeyCode::Escape) || mouse.right_just_pressed {
        game_state.next = GS::Explore;
        return;
    }

    // Only allow shooting on player's turn
    if !turn_state.is_players_turn {
        return;
    }

    // F key or left-click to fire
    let should_fire = mouse.left_just_pressed || keys.is_pressed(KeyCode::F);
    if should_fire {
        let Ok((player_entity, _player_position, equipment_slots)) = q_player.single() else {
            return;
        };

        // Check if player has a ranged weapon equipped
        let has_ranged_weapon =
            if let (Some(equipment), Some(registry)) = (equipment_slots, registry.as_deref()) {
                if let Some(weapon_id) = equipment.get_equipped_item(EquipmentSlot::MainHand) {
                    if let Some(weapon_entity) = registry.get_entity(weapon_id) {
                        // Check if it's a ranged weapon by trying to get the component
                        cmds.get_entity(weapon_entity)
                            .map(|_entity_cmds| {
                                // This is a bit hacky, but we need to check if RangedWeapon component exists
                                // In a real implementation, we'd want a better way to check this
                                true // For now, assume any equipped weapon can be used for shooting
                            })
                            .unwrap_or(false)
                    } else {
                        false
                    }
                } else {
                    false
                }
            } else {
                false
            };

        if !has_ranged_weapon {
            // TODO: Show "No ranged weapon equipped" message
            return;
        }

        let target_x = target_cycling.current_target_pos.0.floor() as usize;
        let target_y = target_cycling.current_target_pos.1.floor() as usize;
        let target_z = target_cycling.current_target_pos.2 as usize;

        // Check if there's a valid target at target position
        let zone_idx = world_to_zone_idx(target_x, target_y, target_z);
        let (local_x, local_y) = world_to_zone_local(target_x, target_y);

        let has_target = q_zones
            .iter()
            .find(|z| z.idx == zone_idx)
            .and_then(|zone| zone.entities.get(local_x, local_y))
            .map(|entities| entities.iter().any(|entity| q_health.get(*entity).is_ok()))
            .unwrap_or(false);

        if has_target {
            // Execute shoot action
            cmds.queue(ShootAction {
                shooter_entity: player_entity,
                target_pos: (target_x, target_y, target_z),
            });
        }

        // Return to explore state after shooting (or attempting to shoot)
        // game_state.next = GS::Explore;
    }
}
