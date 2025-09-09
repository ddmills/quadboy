use bevy_ecs::{prelude::*, system::SystemId};
use macroquad::{input::KeyCode, prelude::trace};
use serde::{Deserialize, Serialize};

use crate::{
    common::Palette,
    domain::{
        EquipmentSlot, EquipmentSlots, Health, IgnoreLighting, IsExplored, Label, Player,
        PlayerDebug, PlayerMovedEvent, PlayerPosition, StackCount, Zone, game_loop,
        handle_item_pickup, player_input, render_player_debug,
    },
    engine::{App, Clock, KeyInput, Mouse, Plugin, SerializableComponent, StableIdRegistry},
    rendering::{
        AnimatedGlyph, Glyph, Layer, LightingData, Position, Text, Visibility, world_to_zone_idx,
        world_to_zone_local,
    },
    states::{CurrentGameState, GameStatePlugin, cleanup_system},
    ui::Button,
};

use super::GameState;

#[derive(Resource)]
struct ExploreCallbacks {
    open_map: SystemId,
    open_inventory: SystemId,
    open_debug_spawn: SystemId,
}

#[derive(Resource)]
pub struct TargetCycling {
    pub targets: Vec<(Entity, (f32, f32, f32), f32)>, // Entity, position, distance
    pub current_index: Option<usize>,
    pub current_selected_entity: Option<Entity>,
}

#[derive(Component)]
struct TargetCrosshair;

#[derive(Component)]
struct TargetInfo;

#[derive(Component)]
struct TargetIndicator;

pub struct ExploreStatePlugin;

impl Plugin for ExploreStatePlugin {
    fn build(&self, app: &mut App) {
        GameStatePlugin::new(GameState::Explore)
            .on_enter(
                app,
                (setup_callbacks, on_enter_explore, center_camera_on_player).chain(),
            )
            .on_update(
                app,
                (
                    collect_valid_targets,
                    update_target_cycling,
                    render_target_crosshair,
                    render_target_info,
                    render_player_debug,
                    render_tick_display,
                    render_lighting_debug,
                    render_lighting_ambient_debug,
                    render_cursor,
                    display_entity_names_at_mouse,
                ),
            )
            .on_update(app, player_input)
            .on_update(app, (handle_item_pickup, game_loop))
            .on_leave(
                app,
                (
                    on_leave_explore,
                    cleanup_system::<CleanupStateExplore>,
                    remove_explore_callbacks,
                )
                    .chain(),
            );
    }
}

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct CleanupStateExplore;

fn setup_callbacks(world: &mut World) {
    let callbacks = ExploreCallbacks {
        open_map: world.register_system(open_map),
        open_inventory: world.register_system(open_inventory),
        open_debug_spawn: world.register_system(open_debug_spawn),
    };

    world.insert_resource(callbacks);
}

fn open_map(mut game_state: ResMut<CurrentGameState>) {
    game_state.next = GameState::Overworld;
}

fn open_inventory(mut game_state: ResMut<CurrentGameState>) {
    game_state.next = GameState::Inventory;
}

fn open_debug_spawn(mut game_state: ResMut<CurrentGameState>) {
    game_state.next = GameState::DebugSpawn;
}

fn remove_explore_callbacks(mut cmds: Commands) {
    cmds.remove_resource::<ExploreCallbacks>();
}

fn on_enter_explore(mut cmds: Commands, callbacks: Res<ExploreCallbacks>) {
    trace!("EnterGameState::<Explore>");

    // Initialize target cycling resource
    cmds.insert_resource(TargetCycling {
        targets: Vec::new(),
        current_index: None,
        current_selected_entity: None,
    });

    // Spawn target crosshair (hidden until target selected)
    cmds.spawn((
        AnimatedGlyph::new(vec![116, 117, 118, 119, 120], 12.0),
        Glyph::new(88, Palette::Yellow, Palette::Yellow).layer(Layer::Overlay),
        Position::new_f32(0., 0., 0.),
        Visibility::Hidden,
        IgnoreLighting,
        TargetCrosshair,
        CleanupStateExplore,
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
        CleanupStateExplore,
    ));

    cmds.spawn((
        AnimatedGlyph::new(vec![132, 133, 134, 135, 136], 12.0),
        Glyph::new(94, Palette::Yellow, Palette::Black).layer(Layer::Overlay),
        Position::new_f32(0., 0., 0.),
        Visibility::Hidden,
        IgnoreLighting,
        TargetIndicator,
        CleanupStateExplore,
    ));

    cmds.spawn((
        Text::new("123").bg(Palette::Black),
        Position::new_f32(6., 0., 0.),
        PlayerDebug,
        CleanupStateExplore,
    ));

    cmds.spawn((
        Text::new("Turn: 0.000").bg(Palette::Black),
        Position::new_f32(0., 0.5, 0.),
        TickDisplay,
        CleanupStateExplore,
    ));

    cmds.spawn((
        Text::new("Light: R:0.0 G:0.0 B:0.0 I:0.0").bg(Palette::Black),
        Position::new_f32(0., 1.0, 0.),
        LightingDebugText,
        CleanupStateExplore,
    ));
    cmds.spawn((
        Text::new("#ff00ff").fg1(Palette::White).bg(0xff00ff_u32),
        Position::new_f32(0., 2.5, 0.),
        LightingDebugAmbient,
        CleanupStateExplore,
    ));

    cmds.spawn((
        Glyph::new(0, Palette::Orange, Palette::Orange)
            .bg(Palette::Orange)
            .layer(Layer::GroundOverlay),
        Position::new_f32(0., 0., 0.),
        CursorGlyph,
        CleanupStateExplore,
    ));

    cmds.spawn((
        Text::new("")
            .fg1(Palette::White)
            .bg(Palette::Black)
            .layer(Layer::Overlay),
        Position::new_f32(0., 0., 0.),
        Visibility::Hidden,
        MouseHoverText,
        IgnoreLighting,
        CleanupStateExplore,
    ));

    cmds.spawn((
        Position::new_f32(3., 1.5, 0.),
        Button::new("({Y|M}) MAP", callbacks.open_map).hotkey(KeyCode::M),
        CleanupStateExplore,
    ));

    cmds.spawn((
        Position::new_f32(7., 1.5, 0.),
        Button::new("({Y|I}) INVENTORY", callbacks.open_inventory).hotkey(KeyCode::I),
        CleanupStateExplore,
    ));

    cmds.spawn((
        Position::new_f32(16., 1.5, 0.),
        Button::new("({Y|B}) DEBUG", callbacks.open_debug_spawn).hotkey(KeyCode::B),
        CleanupStateExplore,
    ));
}

fn on_leave_explore() {
    trace!("LeaveGameState::<Explore>");
}

#[derive(Component)]
struct CursorGlyph;

#[derive(Component)]
struct MouseHoverText;

#[derive(Component)]
struct TickDisplay;

#[derive(Component)]
struct LightingDebugText;

#[derive(Component)]
struct LightingDebugAmbient;

fn render_cursor(
    mouse: Res<Mouse>,
    mut q_cursor: Query<&mut Position, With<CursorGlyph>>,
    player_pos: Res<PlayerPosition>,
) {
    let Ok(mut cursor) = q_cursor.single_mut() else {
        return;
    };

    cursor.x = mouse.world.0.floor();
    cursor.y = mouse.world.1.floor();
    cursor.z = player_pos.z.floor();
}

fn display_entity_names_at_mouse(
    mouse: Res<Mouse>,
    player_pos: Res<PlayerPosition>,
    q_zones: Query<&Zone>,
    q_names: Query<(&Label, Option<&StackCount>), With<IsExplored>>,
    mut q_hover_text: Query<(&mut Text, &mut Position, &mut Visibility), With<MouseHoverText>>,
) {
    let mouse_x = mouse.world.0.floor() as usize;
    let mouse_y = mouse.world.1.floor() as usize;
    let mouse_z = player_pos.z as usize;
    let mut names: Vec<String> = Vec::new();

    let zone_idx = world_to_zone_idx(mouse_x, mouse_y, mouse_z);
    let (local_x, local_y) = world_to_zone_local(mouse_x, mouse_y);

    let Some(zone) = q_zones.iter().find(|z| z.idx == zone_idx) else {
        return;
    };

    let Some(entities) = zone.entities.get(local_x, local_y) else {
        return;
    };

    for entity in entities {
        if let Ok((name, stack_count)) = q_names.get(*entity) {
            let mut name = name.get().to_string();

            if let Some(stack) = stack_count
                && stack.count > 1
            {
                name = format!("{} x{}", name, stack.count)
            }

            names.push(name);
        }
    }

    let Ok((mut text, mut text_pos, mut visibility)) = q_hover_text.single_mut() else {
        return;
    };

    if names.is_empty() {
        *visibility = Visibility::Hidden;
        text.value = String::new();
    } else {
        *visibility = Visibility::Visible;
        text.value = names.join(", ");
        text_pos.x = mouse_x as f32 + 1.0;
        text_pos.y = mouse_y as f32;
        text_pos.z = mouse_z as f32;
    }
}

fn render_tick_display(clock: Res<Clock>, mut q_tick_display: Query<&mut Text, With<TickDisplay>>) {
    let Ok(mut text) = q_tick_display.single_mut() else {
        return;
    };

    text.value = format!(
        "{{G|{}}}.{{g|{:03}}} {{G|Day {}}} {{Y|{:02}}}:{{g|{:02}}}",
        clock.current_turn(),
        clock.sub_turn(),
        clock.get_day() + 1,
        clock.get_hour(),
        clock.get_minute() % 60,
    );
}

fn center_camera_on_player(
    mut e_player_moved: EventWriter<PlayerMovedEvent>,
    q_player: Query<&Position, With<Player>>,
) {
    let p = q_player.single().expect("Expect Player").world();
    e_player_moved.write(PlayerMovedEvent {
        x: p.0,
        y: p.1,
        z: p.2,
    });
}

fn render_lighting_ambient_debug(
    lighting_data: Res<LightingData>,
    mut q_debug_ambient_text: Query<&mut Text, With<LightingDebugAmbient>>,
) {
    let Ok(mut text) = q_debug_ambient_text.single_mut() else {
        return;
    };

    // Get lighting at cursor position
    let light_value = lighting_data.get_ambient_vec4();

    let r = light_value.x;
    let g = light_value.y;
    let b = light_value.z;
    let intensity = light_value.w;

    // Convert to hex for comparison
    let hex_r = (r * 255.0) as u32;
    let hex_g = (g * 255.0) as u32;
    let hex_b = (b * 255.0) as u32;
    let hex_color = (hex_r << 16) | (hex_g << 8) | hex_b;

    text.bg = Some(lighting_data.get_ambient_color());

    text.value = format!("#{:06X} ({:.2})", hex_color, intensity);
}

fn render_lighting_debug(
    mouse: Res<Mouse>,
    lighting_data: Res<LightingData>,
    mut q_debug_text: Query<&mut Text, With<LightingDebugText>>,
) {
    let Ok(mut text) = q_debug_text.single_mut() else {
        return;
    };

    let mouse_x = mouse.world.0.floor() as usize;
    let mouse_y = mouse.world.1.floor() as usize;
    let (local_x, local_y) = world_to_zone_local(mouse_x, mouse_y);

    // Get lighting at cursor position
    let light_info = if let Some(light_value) = lighting_data.get_light(local_x, local_y) {
        let r = light_value.rgb.x;
        let g = light_value.rgb.y;
        let b = light_value.rgb.z;
        let intensity = light_value.intensity;
        let flicker = light_value.flicker;

        // Convert to hex for comparison
        let hex_r = (r * 255.0) as u32;
        let hex_g = (g * 255.0) as u32;
        let hex_b = (b * 255.0) as u32;
        let hex_color = (hex_r << 16) | (hex_g << 8) | hex_b;

        format!(
            "Light: R:{:.2} G:{:.2} B:{:.2} I:{:.2} F:{:.2} (#{:06X})",
            r, g, b, intensity, flicker, hex_color
        )
    } else {
        "Light: No data".to_string()
    };

    // Get ambient info for comparison
    let ambient_color = lighting_data.get_ambient_color();
    let ambient_intensity = lighting_data.get_ambient_intensity();
    let ambient_r = ((ambient_color >> 16) & 0xFF) as f32 / 255.0;
    let ambient_g = ((ambient_color >> 8) & 0xFF) as f32 / 255.0;
    let ambient_b = (ambient_color & 0xFF) as f32 / 255.0;

    text.value = format!(
        "{}\nAmbient: R:{:.2} G:{:.2} B:{:.2} I:{:.2} (#{:06X})",
        light_info, ambient_r, ambient_g, ambient_b, ambient_intensity, ambient_color
    );
}

fn collect_valid_targets(
    mut target_cycling: ResMut<TargetCycling>,
    q_player: Query<(&Position, Option<&EquipmentSlots>), With<Player>>,
    q_health: Query<(Entity, &Position), With<Health>>,
    q_zones: Query<&Zone>,
    registry: Option<Res<StableIdRegistry>>,
) {
    let Ok((player_position, equipment_slots)) = q_player.single() else {
        return;
    };

    // Get weapon range
    let weapon_range =
        if let (Some(equipment), Some(registry)) = (equipment_slots, registry.as_deref()) {
            if let Some(weapon_id) = equipment.get_equipped_item(EquipmentSlot::MainHand) {
                if let Some(_weapon_entity) = registry.get_entity(weapon_id) {
                    12 // Default range for now
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
                target_cycling.targets.get(0).map(|(entity, _, _)| *entity);
        }
    } else {
        // No previous selection - auto-select nearest target if available
        if !target_cycling.targets.is_empty() {
            target_cycling.current_index = Some(0);
            target_cycling.current_selected_entity =
                target_cycling.targets.get(0).map(|(entity, _, _)| *entity);
        } else {
            target_cycling.current_index = None;
            target_cycling.current_selected_entity = None;
        }
    }
}

fn update_target_cycling(mut target_cycling: ResMut<TargetCycling>, keys: Res<KeyInput>) {
    // Check for C key press to cycle targets
    if keys.is_pressed(KeyCode::C) {
        if !target_cycling.targets.is_empty() {
            // Cycle to next target
            target_cycling.current_index = match target_cycling.current_index {
                None => Some(0), // Start at first (nearest) target
                Some(idx) => Some((idx + 1) % target_cycling.targets.len()),
            };

            // Update selected entity
            if let Some(idx) = target_cycling.current_index {
                if let Some((entity, _, _)) = target_cycling.targets.get(idx) {
                    target_cycling.current_selected_entity = Some(*entity);
                }
            }
        }
    }
}

fn render_target_crosshair(
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

fn render_target_info(
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
                if *entity_at_pos == *entity {
                    if let Ok(health) = q_health.get(*entity) {
                        target_health = Some(health);
                        if let Ok(label) = q_names.get(*entity) {
                            target_name = Some(label.get());
                        }
                        break;
                    }
                }
            }

            if let (Some(name), Some(health)) = (target_name, target_health) {
                // Show and update target info text
                *text_visibility = Visibility::Visible;
                text.value = format!("{} ({}/{})", name, health.current, health.max);
                text_pos.x = pos.0.floor() + 1.;
                text_pos.y = pos.1.floor();
                text_pos.z = pos.2.floor();

                // Show and position target indicator above the target
                *indicator_visibility = Visibility::Visible;
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
