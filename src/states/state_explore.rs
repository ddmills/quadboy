use bevy_ecs::{prelude::*, schedule::common_conditions::resource_changed, system::SystemId};

use macroquad::{input::KeyCode, prelude::trace};
use serde::{Deserialize, Serialize};

use crate::{
    DebugMode,
    cfg::ZONE_SIZE,
    common::{Palette, hex},
    domain::{
        ActiveConditions, AiController, ConditionType, CreatureType, DefaultMeleeAttack,
        Description, EquipmentSlot, EquipmentSlots, FactionId, Health, IgnoreLighting,
        Item, Label, Level, Player, PlayerDebug, PlayerMovedEvent, PlayerPosition, Stats,
        TargetCycling, Weapon, WeaponType, Zone, collect_valid_targets, game_loop,
        handle_item_pickup, init_targeting_resource, player_input, render_player_debug,
        render_target_crosshair, render_target_info, spawn_targeting_ui, update_mouse_targeting,
        update_target_cycling,
    },
    engine::{App, KeyInput, Mouse, Plugin, SerializableComponent, StableId, StableIdRegistry},
    rendering::{
        Glyph, Layer, Position, ScreenSize, Text, Visibility, setup_zone_outline_state,
        spawn_zone_outline, world_to_zone_idx, world_to_zone_local, zone_local_to_world,
    },
    states::{CurrentGameState, GameStatePlugin, cleanup_system},
    ui::{
        Bar, Button, Dialog, DialogState, UiLayout, XPProgressBar, center_dialogs_on_screen_change,
        debug_collider_flags, display_entity_names_at_mouse, draw_ui_panels,
        render_ai_debug_indicators, render_cursor, render_lighting_debug, render_tick_display,
        spawn_ai_debug_dialog, spawn_debug_ui_entities, spawn_event_log_ui, spawn_examine_dialog,
        update_bars, update_ui_layout, update_xp_progress_bars,
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
                (
                    setup_callbacks,
                    setup_zone_outline_state,
                    on_enter_explore,
                    center_camera_on_player,
                    update_ui_layout,
                    draw_ui_panels,
                    spawn_target_panel_ui,
                )
                    .chain(),
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
                    render_ai_debug_indicators,
                ),
            )
            .on_update(
                app,
                (
                    update_bars,
                    update_xp_progress_bars,
                    update_player_hp_bar,
                    update_player_armor_bar,
                    update_player_ammo_bar,
                    update_player_condition_display,
                    update_player_xp_label,
                    update_player_hp_label,
                    update_player_armor_label,
                    update_target_panel_visibility,
                    update_target_panel_glyph,
                    update_target_panel_name,
                    update_target_panel_hp_label,
                    update_target_panel_hp_bar,
                    update_target_panel_armor_label,
                    update_target_panel_armor_bar,
                    update_target_panel_hit_chance,
                    update_target_condition_display,
                    handle_examine_input,
                    handle_debug_input,
                ),
            )
            .on_update(app, debug_collider_flags)
            .on_update(app, spawn_zone_outline)
            .on_update(app, player_input)
            .on_update(app, (handle_item_pickup, game_loop))
            .on_update(
                app,
                center_dialogs_on_screen_change.run_if(resource_changed::<ScreenSize>),
            )
            .on_update(
                app,
                update_target_panel_positioning.run_if(resource_changed::<UiLayout>),
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
pub struct PlayerXPLabel;

#[derive(Component)]
pub struct PlayerHPLabel;

#[derive(Component)]
pub struct PlayerArmorLabel;

#[derive(Component)]
pub struct PlayerHPBar;

#[derive(Component)]
pub struct PlayerArmorBar;

#[derive(Component)]
pub struct PlayerAmmoBar;

#[derive(Component)]
pub struct PlayerConditionDisplay;

#[derive(Component)]
pub struct PlayerConditionItem {
    pub condition_type: ConditionType,
}

#[derive(Component)]
pub struct TargetPanelGlyph;

#[derive(Component)]
pub struct TargetPanelName;

#[derive(Component)]
pub struct TargetPanelHPLabel;

#[derive(Component)]
pub struct TargetPanelHPBar;

#[derive(Component)]
pub struct TargetPanelArmorLabel;

#[derive(Component)]
pub struct TargetPanelArmorBar;

#[derive(Component)]
pub struct TargetPanelHitChance;

#[derive(Component)]
pub struct TargetConditionItem {
    pub condition_type: ConditionType,
}

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
        Position::new_f32(12., 1., 0.),
        PlayerDebug,
        CleanupStateExplore,
    ));

    // Spawn UI buttons
    spawn_ui_buttons(&mut cmds, &callbacks);

    // Spawn XP label
    cmds.spawn((
        Text::new("XP: 0/100").fg1(Palette::White).layer(Layer::Ui),
        Position::new_f32(0.5, 3., 0.),
        PlayerXPLabel,
        CleanupStateExplore,
    ));

    // Spawn XP progress bar
    cmds.spawn((
        Text::new("").fg2(Palette::DarkGray).layer(Layer::Ui),
        Position::new_f32(0.5, 3.5, 0.),
        Bar::new(0, 100, 18, Palette::Yellow, Palette::DarkGray),
        XPProgressBar::new(30),
        CleanupStateExplore,
    ));

    // Spawn HP label
    cmds.spawn((
        Text::new("Health: 1/1")
            .fg1(Palette::White)
            .layer(Layer::Ui),
        Position::new_f32(0.5, 4., 0.),
        PlayerHPLabel,
        CleanupStateExplore,
    ));

    // Spawn player HP display
    cmds.spawn((
        Text::new("")
            .fg1(Palette::White)
            .fg2(Palette::DarkGray)
            .layer(Layer::Ui),
        Position::new_f32(0.5, 4.5, 0.),
        Bar::new(1, 1, 18, Palette::Red, Palette::DarkGray),
        PlayerHPBar,
        CleanupStateExplore,
    ));

    // Spawn armor label
    cmds.spawn((
        Text::new("Armor: 0/1").fg1(Palette::White).layer(Layer::Ui),
        Position::new_f32(0.5, 5., 0.),
        PlayerArmorLabel,
        CleanupStateExplore,
    ));

    // Spawn player armor display
    cmds.spawn((
        Text::new("")
            .fg1(Palette::White)
            .fg2(Palette::DarkGray)
            .layer(Layer::Ui),
        Position::new_f32(0.5, 5.5, 0.),
        Bar::new(0, 1, 18, Palette::Cyan, Palette::DarkGray),
        PlayerArmorBar,
        CleanupStateExplore,
    ));

    // Spawn player ammo display
    cmds.spawn((
        Text::new("")
            .fg1(Palette::White)
            .fg2(Palette::DarkGray)
            .layer(Layer::Ui),
        Position::new_f32(0.5, 6., 0.),
        PlayerAmmoBar,
        CleanupStateExplore,
    ));

    // Player condition displays are now spawned dynamically in update_player_condition_display

    // Spawn event log UI
    spawn_event_log_ui(&mut cmds);
}

fn spawn_target_panel_ui(mut cmds: Commands, ui: Res<UiLayout>) {
    let panel_x = ui.target_panel.x as f32;
    let panel_y = ui.target_panel.y as f32;

    // Target Name (row 0, y = panel_y + 0.5, left-aligned)
    cmds.spawn((
        Text::new("").fg1(Palette::White).layer(Layer::Ui),
        Position::new_f32(panel_x + 0.5, panel_y + 0.5, 0.0),
        Visibility::Hidden,
        TargetPanelName,
        CleanupStateExplore,
    ));

    // Target Glyph (row 1, y = panel_y + 1.0, 1.0 tall game glyph)
    cmds.spawn((
        Glyph::new(0, Palette::White, Palette::White).layer(Layer::Ui),
        Position::new_f32(panel_x + 0.5, panel_y + 1.0, 0.0),
        Visibility::Hidden,
        TargetPanelGlyph,
        CleanupStateExplore,
    ));

    // Health Label (row 3, y = panel_y + 2.5)
    cmds.spawn((
        Text::new("Health: 1/1")
            .fg1(Palette::White)
            .layer(Layer::Ui),
        Position::new_f32(panel_x + 0.5, panel_y + 2.5, 0.0),
        Visibility::Hidden,
        TargetPanelHPLabel,
        CleanupStateExplore,
    ));

    // HP Bar (row 3.5, y = panel_y + 3.0)
    cmds.spawn((
        Text::new("")
            .fg1(Palette::White)
            .fg2(Palette::DarkGray)
            .layer(Layer::Ui),
        Position::new_f32(panel_x + 0.5, panel_y + 3.0, 0.0),
        Bar::new(1, 1, 18, Palette::Red, Palette::DarkGray),
        Visibility::Hidden,
        TargetPanelHPBar,
        CleanupStateExplore,
    ));

    // Armor Label (row 4.5, y = panel_y + 4.0)
    cmds.spawn((
        Text::new("Armor: 0/1").fg1(Palette::White).layer(Layer::Ui),
        Position::new_f32(panel_x + 0.5, panel_y + 4.0, 0.0),
        Visibility::Hidden,
        TargetPanelArmorLabel,
        CleanupStateExplore,
    ));

    // Armor Bar (row 5, y = panel_y + 4.5)
    cmds.spawn((
        Text::new("")
            .fg1(Palette::White)
            .fg2(Palette::DarkGray)
            .layer(Layer::Ui),
        Position::new_f32(panel_x + 0.5, panel_y + 4.5, 0.0),
        Bar::new(0, 1, 18, Palette::Cyan, Palette::DarkGray),
        Visibility::Hidden,
        TargetPanelArmorBar,
        CleanupStateExplore,
    ));

    // Hit Chance (row 5.5, y = panel_y + 5.0)
    cmds.spawn((
        Text::new("Hit: 0%").fg1(Palette::White).layer(Layer::Ui),
        Position::new_f32(panel_x + 0.5, panel_y + 5.0, 0.0),
        Visibility::Hidden,
        TargetPanelHitChance,
        CleanupStateExplore,
    ));

    // Target condition displays are spawned dynamically in update_target_condition_display
}

fn update_target_panel_positioning(
    ui: Res<UiLayout>,
    mut q_name: Query<&mut Position, With<TargetPanelName>>,
    mut q_glyph: Query<&mut Position, (With<TargetPanelGlyph>, Without<TargetPanelName>)>,
    mut q_hp_label: Query<
        &mut Position,
        (
            With<TargetPanelHPLabel>,
            Without<TargetPanelName>,
            Without<TargetPanelGlyph>,
        ),
    >,
    mut q_hp_bar: Query<
        &mut Position,
        (
            With<TargetPanelHPBar>,
            Without<TargetPanelName>,
            Without<TargetPanelGlyph>,
            Without<TargetPanelHPLabel>,
        ),
    >,
    mut q_armor_label: Query<
        &mut Position,
        (
            With<TargetPanelArmorLabel>,
            Without<TargetPanelName>,
            Without<TargetPanelGlyph>,
            Without<TargetPanelHPLabel>,
            Without<TargetPanelHPBar>,
        ),
    >,
    mut q_armor_bar: Query<
        &mut Position,
        (
            With<TargetPanelArmorBar>,
            Without<TargetPanelName>,
            Without<TargetPanelGlyph>,
            Without<TargetPanelHPLabel>,
            Without<TargetPanelHPBar>,
            Without<TargetPanelArmorLabel>,
        ),
    >,
    mut q_hit_chance: Query<
        &mut Position,
        (
            With<TargetPanelHitChance>,
            Without<TargetPanelName>,
            Without<TargetPanelGlyph>,
            Without<TargetPanelHPLabel>,
            Without<TargetPanelHPBar>,
            Without<TargetPanelArmorLabel>,
            Without<TargetPanelArmorBar>,
        ),
    >,
) {
    let panel_x = ui.target_panel.x as f32;
    let panel_y = ui.target_panel.y as f32;

    // Update name position (row 0)
    if let Ok(mut pos) = q_name.single_mut() {
        pos.x = panel_x + 0.5;
        pos.y = panel_y + 0.5;
    }

    // Update glyph position (row 1)
    if let Ok(mut pos) = q_glyph.single_mut() {
        pos.x = panel_x + 0.5;
        pos.y = panel_y + 1.0;
    }

    // Update HP label position (row 3)
    if let Ok(mut pos) = q_hp_label.single_mut() {
        pos.x = panel_x + 0.5;
        pos.y = panel_y + 2.5;
    }

    // Update HP bar position (row 3.5)
    if let Ok(mut pos) = q_hp_bar.single_mut() {
        pos.x = panel_x + 0.5;
        pos.y = panel_y + 3.0;
    }

    // Update armor label position (row 4.5)
    if let Ok(mut pos) = q_armor_label.single_mut() {
        pos.x = panel_x + 0.5;
        pos.y = panel_y + 4.0;
    }

    // Update armor bar position (row 5)
    if let Ok(mut pos) = q_armor_bar.single_mut() {
        pos.x = panel_x + 0.5;
        pos.y = panel_y + 4.5;
    }

    // Update hit chance position (row 5.5)
    if let Ok(mut pos) = q_hit_chance.single_mut() {
        pos.x = panel_x + 0.5;
        pos.y = panel_y + 5.0;
    }

    // Update condition positions (will be handled in update_target_condition_display when spawning)
}

fn update_target_panel_visibility(
    target_cycling: Res<TargetCycling>,
    mut q_name: Query<&mut Visibility, With<TargetPanelName>>,
    mut q_glyph: Query<&mut Visibility, (With<TargetPanelGlyph>, Without<TargetPanelName>)>,
    mut q_hp_label: Query<
        &mut Visibility,
        (
            With<TargetPanelHPLabel>,
            Without<TargetPanelName>,
            Without<TargetPanelGlyph>,
        ),
    >,
    mut q_hp_bar: Query<
        &mut Visibility,
        (
            With<TargetPanelHPBar>,
            Without<TargetPanelName>,
            Without<TargetPanelGlyph>,
            Without<TargetPanelHPLabel>,
        ),
    >,
    mut q_armor_label: Query<
        &mut Visibility,
        (
            With<TargetPanelArmorLabel>,
            Without<TargetPanelName>,
            Without<TargetPanelGlyph>,
            Without<TargetPanelHPLabel>,
            Without<TargetPanelHPBar>,
        ),
    >,
    mut q_armor_bar: Query<
        &mut Visibility,
        (
            With<TargetPanelArmorBar>,
            Without<TargetPanelName>,
            Without<TargetPanelGlyph>,
            Without<TargetPanelHPLabel>,
            Without<TargetPanelHPBar>,
            Without<TargetPanelArmorLabel>,
        ),
    >,
    mut q_hit_chance: Query<
        &mut Visibility,
        (
            With<TargetPanelHitChance>,
            Without<TargetPanelName>,
            Without<TargetPanelGlyph>,
            Without<TargetPanelHPLabel>,
            Without<TargetPanelHPBar>,
            Without<TargetPanelArmorLabel>,
            Without<TargetPanelArmorBar>,
        ),
    >,
) {
    let is_visible = target_cycling.current_selected_entity.is_some();

    if let Ok(mut vis) = q_name.single_mut() {
        *vis = if is_visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    if let Ok(mut vis) = q_glyph.single_mut() {
        *vis = if is_visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    if let Ok(mut vis) = q_hp_label.single_mut() {
        *vis = if is_visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    if let Ok(mut vis) = q_hp_bar.single_mut() {
        *vis = if is_visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    if let Ok(mut vis) = q_armor_label.single_mut() {
        *vis = if is_visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    if let Ok(mut vis) = q_armor_bar.single_mut() {
        *vis = if is_visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    if let Ok(mut vis) = q_hit_chance.single_mut() {
        *vis = if is_visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

fn update_target_panel_glyph(
    target_cycling: Res<TargetCycling>,
    q_target_glyphs: Query<&Glyph, Without<TargetPanelGlyph>>,
    mut q_panel_glyph: Query<&mut Glyph, With<TargetPanelGlyph>>,
) {
    let Ok(mut panel_glyph) = q_panel_glyph.single_mut() else {
        return;
    };

    if let Some(target_entity) = target_cycling.current_selected_entity {
        if let Ok(target_glyph) = q_target_glyphs.get(target_entity) {
            panel_glyph.idx = target_glyph.idx;
            panel_glyph.fg1 = target_glyph.fg1;
            panel_glyph.fg2 = target_glyph.fg2;
            panel_glyph.texture_id = target_glyph.texture_id;
        }
    }
}

fn update_target_panel_name(
    target_cycling: Res<TargetCycling>,
    q_labels: Query<&Label>,
    mut q_panel_name: Query<&mut Text, With<TargetPanelName>>,
) {
    let Ok(mut panel_name) = q_panel_name.single_mut() else {
        return;
    };

    if let Some(target_entity) = target_cycling.current_selected_entity {
        if let Ok(label) = q_labels.get(target_entity) {
            panel_name.value = label.get().to_string();
        } else {
            panel_name.value = "Unknown".to_string();
        }
    } else {
        panel_name.value = String::new();
    }
}

fn update_target_panel_hp_bar(
    target_cycling: Res<TargetCycling>,
    q_target_health: Query<(&Health, &Level, &Stats), Or<(Changed<Health>, Changed<Stats>)>>,
    mut q_hp_bar: Query<&mut Bar, With<TargetPanelHPBar>>,
) {
    let Ok(mut hp_bar) = q_hp_bar.single_mut() else {
        return;
    };

    if let Some(target_entity) = target_cycling.current_selected_entity {
        if let Ok((health, level, stats)) = q_target_health.get(target_entity) {
            let max_hp = Health::get_max_hp(level, stats);
            hp_bar.update_values(health.current as usize, max_hp as usize);
        }
    }
}

fn update_target_panel_armor_bar(
    target_cycling: Res<TargetCycling>,
    q_target: Query<(&Health, &Stats), Or<(Changed<Health>, Changed<Stats>)>>,
    mut q_armor_bar: Query<&mut Bar, With<TargetPanelArmorBar>>,
) {
    let Ok(mut armor_bar) = q_armor_bar.single_mut() else {
        return;
    };

    if let Some(target_entity) = target_cycling.current_selected_entity {
        if let Ok((health, stats)) = q_target.get(target_entity) {
            let (current_armor, max_armor) = health.get_current_max_armor(stats);
            armor_bar.update_values(current_armor as usize, max_armor as usize);
        }
    }
}

fn update_target_panel_hp_label(
    target_cycling: Res<TargetCycling>,
    q_target_health: Query<(&Health, &Level, &Stats)>,
    mut q_hp_label: Query<&mut Text, With<TargetPanelHPLabel>>,
) {
    let Ok(mut hp_label) = q_hp_label.single_mut() else {
        return;
    };

    if let Some(target_entity) = target_cycling.current_selected_entity {
        if let Ok((health, level, stats)) = q_target_health.get(target_entity) {
            let max_hp = Health::get_max_hp(level, stats);
            hp_label.value = format!("Health: {}/{}", health.current, max_hp);
        }
    }
}

fn update_target_panel_armor_label(
    target_cycling: Res<TargetCycling>,
    q_target: Query<(&Health, &Stats)>,
    mut q_armor_label: Query<&mut Text, With<TargetPanelArmorLabel>>,
) {
    let Ok(mut armor_label) = q_armor_label.single_mut() else {
        return;
    };

    if let Some(target_entity) = target_cycling.current_selected_entity {
        if let Ok((health, stats)) = q_target.get(target_entity) {
            let (current_armor, max_armor) = health.get_current_max_armor(stats);
            armor_label.value = format!("Armor: {}/{}", current_armor, max_armor);
        }
    }
}

fn update_target_panel_hit_chance(
    target_cycling: Res<TargetCycling>,
    q_player: Query<Entity, With<Player>>,
    q_stats: Query<&Stats>,
    q_equipment: Query<&EquipmentSlots>,
    q_weapons: Query<&Weapon>,
    q_default_attacks: Query<&DefaultMeleeAttack>,
    registry: Res<StableIdRegistry>,
    mut q_hit_chance_text: Query<&mut Text, With<TargetPanelHitChance>>,
) {
    let Ok(mut hit_chance_text) = q_hit_chance_text.single_mut() else {
        return;
    };

    if let Some(target_entity) = target_cycling.current_selected_entity {
        if let Ok(player_entity) = q_player.single() {
            let hit_chance = crate::domain::calculate_hit_chance(
                player_entity,
                target_entity,
                &q_stats,
                &q_equipment,
                &q_weapons,
                &q_default_attacks,
                &registry,
            );
            hit_chance_text.value = format!("Hit: {}%", hit_chance);
        } else {
            hit_chance_text.value = "Hit: --".to_string();
        }
    } else {
        hit_chance_text.value = String::new();
    }
}

fn update_target_condition_display(
    target_cycling: Res<TargetCycling>,
    ui: Res<UiLayout>,
    q_target_conditions: Query<&ActiveConditions>,
    mut q_existing_displays: Query<
        (Entity, &TargetConditionItem, &mut Text),
        With<TargetConditionItem>,
    >,
    mut cmds: Commands,
) {
    let panel_x = ui.target_panel.x as f32;
    let panel_y = ui.target_panel.y as f32;

    // If no target selected, despawn all condition displays
    let Some(target_entity) = target_cycling.current_selected_entity else {
        for (entity, _, _) in q_existing_displays.iter() {
            cmds.entity(entity).despawn();
        }
        return;
    };

    // Get target's conditions
    let Ok(active_conditions) = q_target_conditions.get(target_entity) else {
        // Target has no conditions component, despawn all displays
        for (entity, _, _) in q_existing_displays.iter() {
            cmds.entity(entity).despawn();
        }
        return;
    };

    // Despawn displays for conditions that no longer exist
    for (entity, item, _) in q_existing_displays.iter() {
        let condition_still_exists = active_conditions
            .conditions
            .iter()
            .any(|c| c.condition_type == item.condition_type);

        if !condition_still_exists {
            cmds.entity(entity).despawn();
        }
    }

    // Update existing displays and spawn new ones
    for (index, condition) in active_conditions.conditions.iter().enumerate() {
        let y_position = panel_y + 5.5 + (index as f32 * 0.5);

        // Check if display already exists for this condition type
        let mut found_existing = false;
        for (entity, item, mut text) in q_existing_displays.iter_mut() {
            if item.condition_type == condition.condition_type {
                // Update existing display
                let color_char = condition.condition_type.get_display_color_char();
                let icon = condition.condition_type.get_icon_glyph();
                text.value = format!(
                    "{{{}|{} {} ({})}}",
                    color_char, icon, condition.condition_type, condition.duration_remaining
                );

                // Update position in case order changed
                cmds.entity(entity)
                    .insert(Position::new_f32(panel_x + 0.5, y_position, 0.));
                found_existing = true;
                break;
            }
        }

        if !found_existing {
            // Spawn new display
            let color_char = condition.condition_type.get_display_color_char();
            let icon = condition.condition_type.get_icon_glyph();
            let text_value = format!(
                "{{{}|{} {} ({})}}",
                color_char, icon, condition.condition_type, condition.duration_remaining
            );

            cmds.spawn((
                TargetConditionItem {
                    condition_type: condition.condition_type.clone(),
                },
                Text::new(&text_value).layer(Layer::Ui),
                Position::new_f32(panel_x + 0.5, y_position, 0.),
                CleanupStateExplore,
            ));
        }
    }
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
    mut q_hp_display: Query<&mut Bar, With<PlayerHPBar>>,
) {
    let Ok((health, level, stats)) = q_player.single() else {
        return;
    };
    let Ok(mut hp_bar) = q_hp_display.single_mut() else {
        return;
    };

    let max_hp = Health::get_max_hp(level, stats);
    hp_bar.update_values(health.current as usize, max_hp as usize);
}

fn update_player_armor_bar(
    q_player: Query<(&Health, &Stats), (With<Player>, Or<(Changed<Health>, Changed<Stats>)>)>,
    mut q_armor_display: Query<&mut Bar, With<PlayerArmorBar>>,
) {
    let Ok((health, stats)) = q_player.single() else {
        return;
    };
    let Ok(mut armor_bar) = q_armor_display.single_mut() else {
        return;
    };

    let (current_armor, max_armor) = health.get_current_max_armor(stats);
    armor_bar.update_values(current_armor as usize, max_armor as usize);
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

    let Some(weapon_entity) = registry.get_entity(StableId(weapon_id)) else {
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

fn update_player_condition_display(
    q_player: Query<&ActiveConditions, With<Player>>,
    mut q_existing_displays: Query<
        (Entity, &PlayerConditionItem, &mut Text),
        With<PlayerConditionItem>,
    >,
    mut cmds: Commands,
) {
    let Ok(active_conditions) = q_player.single() else {
        // Player not found, despawn all condition displays
        for (entity, _, _) in q_existing_displays.iter() {
            cmds.entity(entity).despawn();
        }
        return;
    };

    // Despawn displays for conditions that no longer exist
    for (entity, item, _) in q_existing_displays.iter() {
        let condition_still_exists = active_conditions
            .conditions
            .iter()
            .any(|c| c.condition_type == item.condition_type);

        if !condition_still_exists {
            cmds.entity(entity).despawn();
        }
    }

    // Update existing displays and spawn new ones
    for (index, condition) in active_conditions.conditions.iter().enumerate() {
        let y_position = 7.0 + (index as f32 * 0.5);

        // Check if display already exists for this condition type
        let mut found_existing = false;
        for (entity, item, mut text) in q_existing_displays.iter_mut() {
            if item.condition_type == condition.condition_type {
                // Update existing display
                let color_char = condition.condition_type.get_display_color_char();
                let icon = condition.condition_type.get_icon_glyph();
                text.value = format!(
                    "{{{}|{} {} ({})}}",
                    color_char, icon, condition.condition_type, condition.duration_remaining
                );

                // Update position in case order changed
                cmds.entity(entity)
                    .insert(Position::new_f32(0.5, y_position, 0.));
                found_existing = true;
                break;
            }
        }

        if !found_existing {
            // Spawn new display
            let color_char = condition.condition_type.get_display_color_char();
            let icon = condition.condition_type.get_icon_glyph();
            let text_value = format!(
                "{{{}|{} {} ({})}}",
                color_char, icon, condition.condition_type, condition.duration_remaining
            );

            cmds.spawn((
                PlayerConditionItem {
                    condition_type: condition.condition_type.clone(),
                },
                Text::new(&text_value).layer(Layer::Ui),
                Position::new_f32(0.5, y_position, 0.),
                CleanupStateExplore,
            ));
        }
    }
}

fn update_player_xp_label(
    q_player: Query<&Level, (With<Player>, Changed<Level>)>,
    mut q_xp_label: Query<&mut Text, With<PlayerXPLabel>>,
) {
    let Ok(level) = q_player.single() else {
        return;
    };
    let Ok(mut xp_text) = q_xp_label.single_mut() else {
        return;
    };

    let current_xp = level.current_xp;
    let next_level_xp = level.xp_to_next_level;
    xp_text.value = format!("XP: {}/{}", current_xp, next_level_xp);
}

fn update_player_hp_label(
    q_player: Query<
        (&Health, &Level, &Stats),
        (With<Player>, Or<(Changed<Health>, Changed<Stats>)>),
    >,
    mut q_hp_label: Query<&mut Text, With<PlayerHPLabel>>,
) {
    let Ok((health, level, stats)) = q_player.single() else {
        return;
    };
    let Ok(mut hp_text) = q_hp_label.single_mut() else {
        return;
    };

    let max_hp = Health::get_max_hp(level, stats);
    hp_text.value = format!("Health: {}/{}", health.current as usize, max_hp as usize);
}

fn update_player_armor_label(
    q_player: Query<(&Health, &Stats), (With<Player>, Or<(Changed<Health>, Changed<Stats>)>)>,
    mut q_armor_label: Query<&mut Text, With<PlayerArmorLabel>>,
) {
    let Ok((health, stats)) = q_player.single() else {
        return;
    };
    let Ok(mut armor_text) = q_armor_label.single_mut() else {
        return;
    };

    let (current_armor, max_armor) = health.get_current_max_armor(stats);
    armor_text.value = format!("Armor: {}/{}", current_armor as usize, max_armor as usize);
}

fn spawn_ui_buttons(cmds: &mut Commands, callbacks: &ExploreCallbacks) {
    let ui_button_y = 15.;

    cmds.spawn((
        Position::new_f32(0.5, ui_button_y, 0.),
        Button::new("({Y|M}) MAP", callbacks.open_map).hotkey(macroquad::input::KeyCode::M),
        CleanupStateExplore,
    ));

    cmds.spawn((
        Position::new_f32(0.5, ui_button_y + 0.5, 0.),
        Button::new("({Y|I}) INVENTORY", callbacks.open_inventory)
            .hotkey(macroquad::input::KeyCode::I),
        CleanupStateExplore,
    ));

    cmds.spawn((
        Position::new_f32(0.5, ui_button_y + 1.0, 0.),
        Button::new("({Y|B}) DEBUG", callbacks.open_debug_spawn)
            .hotkey(macroquad::input::KeyCode::B),
        CleanupStateExplore,
    ));

    cmds.spawn((
        Position::new_f32(0.5, ui_button_y + 1.5, 0.),
        Button::new("({Y|Y}) ATTRIBUTES", callbacks.open_attributes)
            .hotkey(macroquad::input::KeyCode::Y),
        CleanupStateExplore,
    ));

    cmds.spawn((
        Position::new_f32(0.5, ui_button_y + 2.0, 0.),
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

fn handle_debug_input(
    keys: Res<KeyInput>,
    mut debug_mode: ResMut<DebugMode>,
    dialog_state: Res<DialogState>,
) {
    // Only handle F3 key if no dialog is currently open
    if !dialog_state.is_open && keys.is_pressed(macroquad::input::KeyCode::F3) {
        debug_mode.ai_debug = !debug_mode.ai_debug;
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

        // Check if debug mode is active and entity has AI controller
        let debug_mode = world.get_resource::<DebugMode>().unwrap();
        let has_ai = world.get::<AiController>(entity).is_some();

        if debug_mode.ai_debug && has_ai {
            // Show AI debug dialog
            spawn_ai_debug_dialog(world, entity, close_examine_dialog_id);
        } else {
            // Show regular examine dialog
            spawn_examine_dialog(world, entity, player_entity, close_examine_dialog_id);
        }

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

