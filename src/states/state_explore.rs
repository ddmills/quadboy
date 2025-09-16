use bevy_ecs::{prelude::*, schedule::common_conditions::resource_changed, system::SystemId};
use macroquad::{input::KeyCode, prelude::trace};
use serde::{Deserialize, Serialize};

use crate::{
    cfg::ZONE_SIZE,
    common::{Palette, hex},
    domain::{
        CreatureType, Description, EquipmentSlot, EquipmentSlots, FactionId, FactionMap, Health,
        IgnoreLighting, Item, Label, Level, Player, PlayerDebug, PlayerMovedEvent, PlayerPosition,
        Stats, Weapon, WeaponType, Zone, collect_valid_targets, game_loop, handle_item_pickup,
        init_targeting_resource, player_input, render_player_debug, render_target_crosshair,
        render_target_info, spawn_targeting_ui, update_mouse_targeting, update_target_cycling,
    },
    engine::{App, KeyInput, Mouse, Plugin, SerializableComponent, StableIdRegistry},
    rendering::{
        Layer, Position, ScreenSize, Text, Visibility, world_to_zone_idx,
        world_to_zone_local, zone_local_to_world,
    },
    states::{CurrentGameState, GameStatePlugin, cleanup_system},
    ui::{
        Button, Dialog, DialogState, XPProgressBar, center_dialogs_on_screen_change,
        display_entity_names_at_mouse, render_cursor, render_lighting_debug, render_tick_display,
        spawn_debug_ui_entities, spawn_examine_dialog, update_xp_progress_bars,
    },
};

use super::GameState;

#[derive(Resource)]
struct ExploreCallbacks {
    open_map: SystemId,
    open_inventory: SystemId,
    open_debug_spawn: SystemId,
    open_attributes: SystemId,
    open_pause: SystemId,
    examine_entity: SystemId,
    close_examine_dialog: SystemId,
}

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
                    update_mouse_targeting,
                    render_target_crosshair,
                    render_target_info,
                    render_player_debug,
                    render_tick_display,
                    render_lighting_debug,
                    render_cursor,
                    display_entity_names_at_mouse,
                    render_player_map_overlay,
                    update_xp_progress_bars,
                    update_player_hp_bar,
                    update_player_armor_bar,
                    update_player_ammo_bar,
                    handle_examine_input,
                ),
            )
            .on_update(app, player_input)
            .on_update(app, (handle_item_pickup, game_loop))
            .on_update(
                app,
                center_dialogs_on_screen_change.run_if(resource_changed::<ScreenSize>),
            )
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

#[derive(Component)]
pub struct PlayerHPBar;

#[derive(Component)]
pub struct PlayerArmorBar;

#[derive(Component)]
pub struct PlayerAmmoBar;

#[derive(Component)]
pub struct FactionMapOverlay;

fn setup_callbacks(world: &mut World) {
    let callbacks = ExploreCallbacks {
        open_map: world.register_system(open_map),
        open_inventory: world.register_system(open_inventory),
        open_debug_spawn: world.register_system(open_debug_spawn),
        open_attributes: world.register_system(open_attributes),
        open_pause: world.register_system(open_pause),
        examine_entity: world.register_system(examine_entity_at_mouse),
        close_examine_dialog: world.register_system(close_examine_dialog),
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

fn open_attributes(mut game_state: ResMut<CurrentGameState>) {
    game_state.next = GameState::Attributes;
}

fn open_pause(mut game_state: ResMut<CurrentGameState>) {
    game_state.next = GameState::Pause;
}

fn remove_explore_callbacks(mut cmds: Commands) {
    cmds.remove_resource::<ExploreCallbacks>();
}

fn on_enter_explore(mut cmds: Commands, callbacks: Res<ExploreCallbacks>) {
    trace!("EnterGameState::<Explore>");

    // Initialize targeting system
    init_targeting_resource(&mut cmds);
    spawn_targeting_ui(&mut cmds, CleanupStateExplore);

    // Spawn debug UI elements
    spawn_debug_ui_entities(&mut cmds, CleanupStateExplore);

    // Spawn player debug info
    cmds.spawn((
        Text::new("123").bg(Palette::Black),
        Position::new_f32(6., 0., 0.),
        PlayerDebug,
        CleanupStateExplore,
    ));

    // Spawn UI buttons
    spawn_ui_buttons(&mut cmds, &callbacks);

    // Spawn XP progress bar
    cmds.spawn((
        Text::new("").layer(Layer::Ui),
        Position::new_f32(30., 1.5, 0.),
        XPProgressBar::new(30),
        CleanupStateExplore,
    ));

    // Spawn player HP display
    cmds.spawn((
        Text::new("HP").fg1(Palette::White).layer(Layer::Ui),
        Position::new_f32(1., 3., 0.),
        PlayerHPBar,
        CleanupStateExplore,
    ));

    // Spawn player armor display
    cmds.spawn((
        Text::new("").fg1(Palette::White).layer(Layer::Ui),
        Position::new_f32(1., 3.5, 0.),
        PlayerArmorBar,
        CleanupStateExplore,
    ));

    // Spawn player ammo display
    cmds.spawn((
        Text::new("").fg1(Palette::White).layer(Layer::Ui),
        Position::new_f32(1., 4., 0.),
        PlayerAmmoBar,
        CleanupStateExplore,
    ));
}

fn on_leave_explore() {
    trace!("LeaveGameState::<Explore>");
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

fn update_player_hp_bar(
    q_player: Query<
        (&Health, &Level, &Stats),
        (With<Player>, Or<(Changed<Health>, Changed<Stats>)>),
    >,
    mut q_hp_display: Query<&mut Text, With<PlayerHPBar>>,
) {
    let Ok((health, level, stats)) = q_player.single() else {
        return;
    };
    let Ok(mut hp_text) = q_hp_display.single_mut() else {
        return;
    };

    let max_hp = Health::get_max_hp(level, stats);
    hp_text.value = format!("HP: {}/{}", health.current, max_hp);
}

fn update_player_armor_bar(
    q_player: Query<(&Health, &Stats), (With<Player>, Or<(Changed<Health>, Changed<Stats>)>)>,
    mut q_armor_display: Query<&mut Text, With<PlayerArmorBar>>,
) {
    let Ok((health, stats)) = q_player.single() else {
        return;
    };
    let Ok(mut armor_text) = q_armor_display.single_mut() else {
        return;
    };

    let (current_armor, max_armor) = health.get_current_max_armor(stats);
    armor_text.value = format!("Armor: {}/{}", current_armor, max_armor);
}

fn update_player_ammo_bar(
    q_player_equipment: Query<&EquipmentSlots, With<Player>>,
    q_weapons: Query<&Weapon>,
    mut q_ammo_display: Query<&mut Text, With<PlayerAmmoBar>>,
    registry: Option<Res<StableIdRegistry>>,
) {
    let Ok(mut ammo_text) = q_ammo_display.single_mut() else {
        return;
    };

    let Some(registry) = registry else {
        ammo_text.value = "".to_string();
        return;
    };

    let Ok(equipment_slots) = q_player_equipment.single() else {
        ammo_text.value = "".to_string();
        return;
    };

    let Some(weapon_id) = equipment_slots.get_equipped_item(EquipmentSlot::MainHand) else {
        ammo_text.value = "".to_string();
        return;
    };

    let Some(weapon_entity) = registry.get_entity(weapon_id) else {
        ammo_text.value = "".to_string();
        return;
    };

    let Ok(weapon) = q_weapons.get(weapon_entity) else {
        ammo_text.value = "".to_string();
        return;
    };

    if weapon.weapon_type != WeaponType::Ranged {
        ammo_text.value = "".to_string();
        return;
    }

    let (Some(clip_size), Some(current_ammo)) = (weapon.clip_size, weapon.current_ammo) else {
        ammo_text.value = "".to_string();
        return;
    };

    let bar_chars = (0..clip_size)
        .map(|i| if i < current_ammo { 'X' } else { '-' })
        .collect::<String>();

    ammo_text.value = format!("[{}] {}/{}", bar_chars, current_ammo, clip_size);
}

fn spawn_ui_buttons(cmds: &mut Commands, callbacks: &ExploreCallbacks) {
    cmds.spawn((
        Position::new_f32(3., 1.5, 0.),
        Button::new("({Y|M}) MAP", callbacks.open_map).hotkey(macroquad::input::KeyCode::M),
        CleanupStateExplore,
    ));

    cmds.spawn((
        Position::new_f32(7., 1.5, 0.),
        Button::new("({Y|I}) INVENTORY", callbacks.open_inventory)
            .hotkey(macroquad::input::KeyCode::I),
        CleanupStateExplore,
    ));

    cmds.spawn((
        Position::new_f32(16., 1.5, 0.),
        Button::new("({Y|B}) DEBUG", callbacks.open_debug_spawn)
            .hotkey(macroquad::input::KeyCode::B),
        CleanupStateExplore,
    ));

    cmds.spawn((
        Position::new_f32(22., 1.5, 0.),
        Button::new("({Y|Y}) ATTRIBUTES", callbacks.open_attributes)
            .hotkey(macroquad::input::KeyCode::Y),
        CleanupStateExplore,
    ));

    cmds.spawn((
        Position::new_f32(30., 1.5, 0.),
        Button::new("({Y|ESC}) PAUSE", callbacks.open_pause)
            .hotkey(macroquad::input::KeyCode::Escape),
        CleanupStateExplore,
    ));
}

fn handle_examine_input(
    keys: Res<KeyInput>,
    dialog_state: Res<DialogState>,
    callbacks: Res<ExploreCallbacks>,
    mut cmds: Commands,
) {
    // Only handle X key if no dialog is currently open
    if !dialog_state.is_open && keys.is_pressed(macroquad::input::KeyCode::X) {
        cmds.run_system(callbacks.examine_entity);
    }
}

fn examine_entity_at_mouse(world: &mut World) {
    let (mouse_x, mouse_y, mouse_z, close_examine_dialog_id) = {
        let mouse = world.get_resource::<Mouse>().unwrap();
        let player_pos = world.get_resource::<PlayerPosition>().unwrap();
        let callbacks = world.get_resource::<ExploreCallbacks>().unwrap();

        (
            mouse.world.0.floor() as usize,
            mouse.world.1.floor() as usize,
            player_pos.z as usize,
            callbacks.close_examine_dialog,
        )
    };

    let zone_idx = world_to_zone_idx(mouse_x, mouse_y, mouse_z);
    let (local_x, local_y) = world_to_zone_local(mouse_x, mouse_y);

    let zone = {
        let mut q_zones = world.query::<&Zone>();
        q_zones.iter(world).find(|z| z.idx == zone_idx)
    };

    let Some(zone) = zone else {
        return;
    };

    let Some(entities) = zone.entities.get(local_x, local_y) else {
        return;
    };

    // Find the best entity to examine based on priority
    let mut best_entity: Option<(Entity, u32)> = None;

    for entity in entities {
        let priority = get_examinable_entity_priority_world(*entity, world);

        if let Some((_, new_priority)) = priority {
            match best_entity {
                None => best_entity = Some((*entity, new_priority)),
                Some((_, current_priority)) => {
                    if new_priority < current_priority {
                        best_entity = Some((*entity, new_priority));
                    }
                }
            }
        }
    }

    if let Some((entity, _)) = best_entity {
        let player_entity = {
            let mut q_player = world.query_filtered::<Entity, With<Player>>();
            q_player.single(world).unwrap()
        };

        spawn_examine_dialog(world, entity, player_entity, close_examine_dialog_id);

        if let Some(mut dialog_state) = world.get_resource_mut::<DialogState>() {
            dialog_state.is_open = true;
        }
    }
}

fn get_examinable_entity_priority(
    entity: Entity,
    q_creatures: &Query<&CreatureType>,
    q_items: &Query<&Item>,
    q_descriptions: &Query<&Description>,
    q_labels: &Query<&Label>,
) -> Option<(Entity, u32)> {
    // Priority levels (lower number = higher priority)
    if q_creatures.get(entity).is_ok() {
        Some((entity, 1)) // Highest priority: Creatures
    } else if q_items.get(entity).is_ok() {
        Some((entity, 2)) // Second priority: Items
    } else if q_descriptions.get(entity).is_ok() {
        Some((entity, 3)) // Third priority: Entities with descriptions
    } else if q_labels.get(entity).is_ok() {
        Some((entity, 4)) // Lowest priority: Any labeled entity
    } else {
        None // Not examinable
    }
}

fn get_examinable_entity_priority_world(entity: Entity, world: &World) -> Option<(Entity, u32)> {
    // Priority levels (lower number = higher priority)
    if world.get::<CreatureType>(entity).is_some() {
        Some((entity, 1)) // Highest priority: Creatures
    } else if world.get::<Item>(entity).is_some() {
        Some((entity, 2)) // Second priority: Items
    } else if world.get::<Description>(entity).is_some() {
        Some((entity, 3)) // Third priority: Entities with descriptions
    } else if world.get::<Label>(entity).is_some() {
        Some((entity, 4)) // Lowest priority: Any labeled entity
    } else {
        None // Not examinable
    }
}

fn close_examine_dialog(
    mut cmds: Commands,
    q_dialogs: Query<Entity, With<Dialog>>,
    mut dialog_state: ResMut<DialogState>,
) {
    for dialog_entity in q_dialogs.iter() {
        cmds.entity(dialog_entity).despawn();
    }
    dialog_state.is_open = false;
}

fn render_player_map_overlay(
    mut cmds: Commands,
    keys: Res<KeyInput>,
    faction_map: Res<FactionMap>,
    player_pos: Res<PlayerPosition>,
    q_zones: Query<&Zone>,
    q_overlay: Query<Entity, With<FactionMapOverlay>>,
    mut overlay_entities: Local<Vec<Entity>>,
    mut overlay_enabled: Local<bool>,
    mut last_player_pos: Local<Option<(usize, usize, usize)>>,
) {
    // Toggle overlay on J key press
    if keys.is_pressed(KeyCode::J) {
        *overlay_enabled = !*overlay_enabled;
    }

    let has_overlay = !overlay_entities.is_empty();
    let current_player_pos = player_pos.world();
    let player_moved = last_player_pos.is_none_or(|last_pos| last_pos != current_player_pos);

    // Update last known player position
    *last_player_pos = Some(current_player_pos);

    if *overlay_enabled && (!has_overlay || player_moved) {
        // Remove existing overlay if player moved
        if player_moved && has_overlay {
            trace!("Player moved, regenerating FactionMap overlay");
            for entity in overlay_entities.drain(..) {
                cmds.entity(entity).despawn();
            }
        }
        // Spawn overlay
        let zone_idx = player_pos.zone_idx();
        if let Some(zone) = q_zones.iter().find(|z| z.idx == zone_idx) {
            trace!("Spawning FactionMap overlay for zone {}", zone_idx);
            if let Some(dijkstra_map) = faction_map.get_map(FactionId::Player) {
                for x in 0..ZONE_SIZE.0 {
                    for y in 0..ZONE_SIZE.1 {
                        let world_pos = zone_local_to_world(zone.idx, x, y);

                        if dijkstra_map.is_blocked(x, y) {
                            // Show blocked tiles as red X
                            let entity = cmds
                                .spawn((
                                    Text::new("X")
                                        .fg1(0xB62DAF_u32) // Bright red
                                        .layer(Layer::Overlay),
                                    Position::new_world(world_pos),
                                    Visibility::Visible,
                                    IgnoreLighting,
                                    FactionMapOverlay,
                                    CleanupStateExplore,
                                ))
                                .id();

                            overlay_entities.push(entity);
                        } else if let Some(cost) = dijkstra_map.get_cost(x, y)
                            && cost.is_finite() && cost >= 0.0 {
                                let display_num = (cost.min(12.0) as u32).to_string();

                                // Color gradient from green to red (0-12 range)
                                let t = (cost / 12.0).min(1.0);
                                let r = (t * 255.0) as u8;
                                let g = ((1.0 - t) * 255.0) as u8;
                                let color = hex(r, g, 0);

                                let entity = cmds
                                    .spawn((
                                        Text::new(&display_num).fg1(color).layer(Layer::Overlay),
                                        Position::new_world(world_pos),
                                        Visibility::Visible,
                                        IgnoreLighting,
                                        FactionMapOverlay,
                                        CleanupStateExplore,
                                    ))
                                    .id();

                                overlay_entities.push(entity);
                            }
                    }
                }
            }
        }
    } else if !*overlay_enabled && has_overlay {
        // Remove overlay
        trace!(
            "Removing FactionMap overlay ({} entities)",
            overlay_entities.len()
        );
        for entity in overlay_entities.drain(..) {
            cmds.entity(entity).despawn();
        }
    }
}
