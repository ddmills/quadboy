use std::collections::HashMap;

use bevy_ecs::{prelude::*, system::SystemId};
use macroquad::input::KeyCode;

use crate::{
    common::Palette,
    domain::{
        AttributeGroup, AttributePoints, Attributes, Health, Level, ModifierSource, Player,
        StatModifiers, StatType, Stats, game_loop,
    },
    engine::{App, Plugin},
    rendering::{Glyph, Layer, Position, ScreenSize, Text},
    states::{CurrentGameState, GameState, GameStatePlugin, cleanup_system},
    ui::{Button, FullScreenBackground, setup_fullscreen_backgrounds},
};

#[derive(Resource)]
struct AttributesCallbacks {
    back_to_explore: SystemId,
    increase_strength: SystemId,
    decrease_strength: SystemId,
    increase_dexterity: SystemId,
    decrease_dexterity: SystemId,
    increase_constitution: SystemId,
    decrease_constitution: SystemId,
    increase_intelligence: SystemId,
    decrease_intelligence: SystemId,
    reset_all: SystemId,
    select_strength: SystemId,
    select_dexterity: SystemId,
    select_constitution: SystemId,
    select_intelligence: SystemId,
    select_special: SystemId,
    select_stat: HashMap<StatType, SystemId>,
    close_stat_dialog: SystemId,
}

#[derive(Component, Clone)]
pub struct CleanupStateAttributes;

#[derive(Component, Clone)]
struct CleanupStatDialog;

#[derive(Resource)]
struct SelectedAttribute {
    current: AttributeGroup,
}

impl Default for SelectedAttribute {
    fn default() -> Self {
        Self {
            current: AttributeGroup::Strength,
        }
    }
}

#[derive(Resource)]
struct SelectedStat {
    current: Option<StatType>,
}

impl Default for SelectedStat {
    fn default() -> Self {
        Self { current: None }
    }
}

#[derive(Resource)]
struct AttributesUIEntities {
    level_display: Entity,
    available_points: Entity,
    strength_value: Entity,
    dexterity_value: Entity,
    constitution_value: Entity,
    intelligence_value: Entity,
    strength_indicator: Entity,
    dexterity_indicator: Entity,
    constitution_indicator: Entity,
    intelligence_indicator: Entity,
    special_indicator: Entity,
    stat_displays: HashMap<StatType, Entity>,
    stat_section_title: Entity,
    stat_section_desc: Entity,
}

pub struct AttributesStatePlugin;

impl Plugin for AttributesStatePlugin {
    fn build(&self, app: &mut App) {
        GameStatePlugin::new(GameState::Attributes)
            .on_enter(
                app,
                (
                    setup_attributes_callbacks,
                    setup_attributes_screen,
                    setup_attributes_background,
                    setup_fullscreen_backgrounds,
                )
                    .chain(),
            )
            .on_update(app, (game_loop, update_attributes_display, spawn_stat_dialog, cleanup_stat_dialog).chain())
            .on_update(
                app,
                setup_fullscreen_backgrounds.run_if(resource_changed::<ScreenSize>),
            )
            .on_leave(
                app,
                (
                    cleanup_system::<CleanupStateAttributes>,
                    remove_attributes_callbacks,
                    remove_attributes_ui_entities,
                    remove_selected_attribute,
                    remove_selected_stat,
                )
                    .chain(),
            );
    }
}

fn setup_attributes_callbacks(world: &mut World) {
    let mut select_stat_callbacks = HashMap::new();
    for stat_type in StatType::all() {
        let stat = *stat_type;
        let system_id = world.register_system(move |mut selected: ResMut<SelectedStat>| {
            selected.current = Some(stat);
        });
        select_stat_callbacks.insert(*stat_type, system_id);
    }

    let callbacks = AttributesCallbacks {
        back_to_explore: world.register_system(back_to_explore),
        increase_strength: world.register_system(increase_strength),
        decrease_strength: world.register_system(decrease_strength),
        increase_dexterity: world.register_system(increase_dexterity),
        decrease_dexterity: world.register_system(decrease_dexterity),
        increase_constitution: world.register_system(increase_constitution),
        decrease_constitution: world.register_system(decrease_constitution),
        increase_intelligence: world.register_system(increase_intelligence),
        decrease_intelligence: world.register_system(decrease_intelligence),
        reset_all: world.register_system(reset_all_attributes),
        select_strength: world.register_system(select_strength),
        select_dexterity: world.register_system(select_dexterity),
        select_constitution: world.register_system(select_constitution),
        select_intelligence: world.register_system(select_intelligence),
        select_special: world.register_system(select_special),
        select_stat: select_stat_callbacks,
        close_stat_dialog: world.register_system(close_stat_dialog),
    };

    world.insert_resource(callbacks);
    world.insert_resource(SelectedAttribute::default());
    world.insert_resource(SelectedStat::default());
}

fn back_to_explore(mut game_state: ResMut<CurrentGameState>) {
    game_state.next = GameState::Explore;
}

fn increase_strength(mut q_player: Query<(&mut Attributes, &mut AttributePoints), With<Player>>) {
    if let Ok((mut attributes, mut points)) = q_player.single_mut()
        && points.increase_attribute()
    {
        attributes.strength += 1;
    }
}

fn decrease_strength(mut q_player: Query<(&mut Attributes, &mut AttributePoints), With<Player>>) {
    if let Ok((mut attributes, mut points)) = q_player.single_mut()
        && attributes.strength > 0
        && points.decrease_attribute()
    {
        attributes.strength -= 1;
    }
}

fn increase_dexterity(mut q_player: Query<(&mut Attributes, &mut AttributePoints), With<Player>>) {
    if let Ok((mut attributes, mut points)) = q_player.single_mut()
        && points.increase_attribute()
    {
        attributes.dexterity += 1;
    }
}

fn decrease_dexterity(mut q_player: Query<(&mut Attributes, &mut AttributePoints), With<Player>>) {
    if let Ok((mut attributes, mut points)) = q_player.single_mut()
        && attributes.dexterity > 0
        && points.decrease_attribute()
    {
        attributes.dexterity -= 1;
    }
}

fn increase_constitution(
    mut q_player: Query<(&mut Attributes, &mut AttributePoints), With<Player>>,
) {
    if let Ok((mut attributes, mut points)) = q_player.single_mut()
        && points.increase_attribute()
    {
        attributes.constitution += 1;
    }
}

fn decrease_constitution(
    mut q_player: Query<(&mut Attributes, &mut AttributePoints), With<Player>>,
) {
    if let Ok((mut attributes, mut points)) = q_player.single_mut()
        && attributes.constitution > 0
        && points.decrease_attribute()
    {
        attributes.constitution -= 1;
    }
}

fn increase_intelligence(
    mut q_player: Query<(&mut Attributes, &mut AttributePoints), With<Player>>,
) {
    if let Ok((mut attributes, mut points)) = q_player.single_mut()
        && points.increase_attribute()
    {
        attributes.intelligence += 1;
    }
}

fn decrease_intelligence(
    mut q_player: Query<(&mut Attributes, &mut AttributePoints), With<Player>>,
) {
    if let Ok((mut attributes, mut points)) = q_player.single_mut()
        && attributes.intelligence > 0
        && points.decrease_attribute()
    {
        attributes.intelligence -= 1;
    }
}

fn reset_all_attributes(
    mut q_player: Query<(&mut Attributes, &mut AttributePoints), With<Player>>,
) {
    if let Ok((mut attributes, mut points)) = q_player.single_mut() {
        attributes.strength = 0;
        attributes.dexterity = 0;
        attributes.constitution = 0;
        attributes.intelligence = 0;
        points.reset_all();
    }
}

fn select_strength(mut selected: ResMut<SelectedAttribute>) {
    selected.current = AttributeGroup::Strength;
}

fn select_dexterity(mut selected: ResMut<SelectedAttribute>) {
    selected.current = AttributeGroup::Dexterity;
}

fn select_constitution(mut selected: ResMut<SelectedAttribute>) {
    selected.current = AttributeGroup::Constitution;
}

fn select_intelligence(mut selected: ResMut<SelectedAttribute>) {
    selected.current = AttributeGroup::Intelligence;
}

fn select_special(mut selected: ResMut<SelectedAttribute>) {
    selected.current = AttributeGroup::Special;
}

fn close_stat_dialog(mut selected: ResMut<SelectedStat>) {
    selected.current = None;
}

fn update_attributes_display(
    mut q_text: Query<&mut Text>,
    mut q_button: Query<&mut Button>,
    mut q_position: Query<&mut Position>,
    q_player_changed: Query<
        (
            &Level,
            &Attributes,
            &AttributePoints,
            &Stats,
            &StatModifiers,
            &Health,
        ),
        (
            With<Player>,
            Or<(
                Changed<Attributes>,
                Changed<AttributePoints>,
                Changed<Stats>,
                Changed<Health>,
            )>,
        ),
    >,
    q_player: Query<
        (
            &Level,
            &Attributes,
            &AttributePoints,
            &Stats,
            &StatModifiers,
            &Health,
        ),
        With<Player>,
    >,
    ui_entities: Option<Res<AttributesUIEntities>>,
    selected: Res<SelectedAttribute>,
) {
    let Some(ui_entities) = ui_entities else {
        return;
    };

    let player_data_changed = !q_player_changed.is_empty();
    let selection_changed = selected.is_changed();

    if !player_data_changed && !selection_changed {
        return;
    }

    let Ok((level, attributes, attribute_points, stats, stat_modifiers, health)) =
        q_player.get_single()
    else {
        return;
    };

    if let Ok(mut text) = q_text.get_mut(ui_entities.level_display) {
        text.value = format!("Level: {}", level.current_level);
    }

    if let Ok(mut text) = q_text.get_mut(ui_entities.available_points) {
        text.value = format!("Available: {} pts", attribute_points.available);
    }

    if let Ok(mut button) = q_button.get_mut(ui_entities.strength_value) {
        if let Button::Button { label, .. } = &mut *button {
            *label = format!("STRENGTH      {}", attributes.strength);
        }
    }

    if let Ok(mut button) = q_button.get_mut(ui_entities.dexterity_value) {
        if let Button::Button { label, .. } = &mut *button {
            *label = format!("DEXTERITY     {}", attributes.dexterity);
        }
    }

    if let Ok(mut button) = q_button.get_mut(ui_entities.constitution_value) {
        if let Button::Button { label, .. } = &mut *button {
            *label = format!("CONSTITUTION  {}", attributes.constitution);
        }
    }

    if let Ok(mut button) = q_button.get_mut(ui_entities.intelligence_value) {
        if let Button::Button { label, .. } = &mut *button {
            *label = format!("INTELLIGENCE  {}", attributes.intelligence);
        }
    }

    let (indicator_char, title, desc) = match selected.current {
        AttributeGroup::Strength => (
            "→",
            "STRENGTH STATS",
            "Raw physical power affects weapon handling and melee damage.",
        ),
        AttributeGroup::Dexterity => (
            "→",
            "DEXTERITY STATS",
            "Agility and precision affects speed, accuracy, and evasion.",
        ),
        AttributeGroup::Constitution => (
            "→",
            "CONSTITUTION STATS",
            "Durability and health determines max HP and fortitude.",
        ),
        AttributeGroup::Intelligence => (
            "→",
            "INTELLIGENCE STATS",
            "Knowledge and tactics affects armor regeneration.",
        ),
        AttributeGroup::Special => (
            "→",
            "SPECIAL STATS",
            "Stats that don't derive from base attributes.",
        ),
    };

    if let Ok(mut text) = q_text.get_mut(ui_entities.strength_indicator) {
        text.value = if selected.current == AttributeGroup::Strength {
            indicator_char.to_string()
        } else {
            " ".to_string()
        };
    }

    if let Ok(mut text) = q_text.get_mut(ui_entities.dexterity_indicator) {
        text.value = if selected.current == AttributeGroup::Dexterity {
            indicator_char.to_string()
        } else {
            " ".to_string()
        };
    }

    if let Ok(mut text) = q_text.get_mut(ui_entities.constitution_indicator) {
        text.value = if selected.current == AttributeGroup::Constitution {
            indicator_char.to_string()
        } else {
            " ".to_string()
        };
    }

    if let Ok(mut text) = q_text.get_mut(ui_entities.intelligence_indicator) {
        text.value = if selected.current == AttributeGroup::Intelligence {
            indicator_char.to_string()
        } else {
            " ".to_string()
        };
    }

    if let Ok(mut text) = q_text.get_mut(ui_entities.special_indicator) {
        text.value = if selected.current == AttributeGroup::Special {
            indicator_char.to_string()
        } else {
            " ".to_string()
        };
    }

    if let Ok(mut text) = q_text.get_mut(ui_entities.stat_section_title) {
        text.value = format!("=== {} ===", title);
    }

    if let Ok(mut text) = q_text.get_mut(ui_entities.stat_section_desc) {
        text.value = desc.to_string();
    }

    let right_x = 32.0;
    let mut y_pos = 4.5;

    for stat_type in StatType::all() {
        let is_visible = stat_type.get_attribute_group() == selected.current;

        if let Some(&entity) = ui_entities.stat_displays.get(stat_type) {
            if let Ok(mut button) = q_button.get_mut(entity) {
                let base = stat_type.get_base_value(attributes);
                let modifiers = stat_modifiers.get_total_for_stat(*stat_type);
                let total = stats.get_stat(*stat_type);

                let stat_name = format!("{:?}", stat_type);

                let display_text = if *stat_type == StatType::Armor {
                    let (current_armor, max_armor) = health.get_current_max_armor(stats);
                    format!(
                        "  {:14} {:3}  ({}/{} current | {} max | {:+} mods)",
                        stat_name, total, current_armor, max_armor, total, modifiers
                    )
                } else {
                    format!(
                        "  {:14} {:3}  ({} base {:+} mods)",
                        stat_name, total, base, modifiers
                    )
                };

                if let Button::Button { label, .. } = &mut *button {
                    *label = display_text;
                }
            }

            if let Ok(mut pos) = q_position.get_mut(entity) {
                if is_visible {
                    *pos = Position::new_f32(right_x, y_pos, 0.);
                    y_pos += 0.5;
                } else {
                    *pos = Position::new_f32(-1000.0, -1000.0, 0.);
                }
            }
        }
    }
}

fn remove_attributes_callbacks(mut cmds: Commands) {
    cmds.remove_resource::<AttributesCallbacks>();
}

fn remove_attributes_ui_entities(mut cmds: Commands) {
    cmds.remove_resource::<AttributesUIEntities>();
}

fn remove_selected_attribute(mut cmds: Commands) {
    cmds.remove_resource::<SelectedAttribute>();
}

fn remove_selected_stat(mut cmds: Commands) {
    cmds.remove_resource::<SelectedStat>();
}

fn spawn_stat_dialog(
    mut cmds: Commands,
    selected_stat: Res<SelectedStat>,
    q_dialog: Query<Entity, With<CleanupStatDialog>>,
    q_player: Query<(&Attributes, &Stats, &StatModifiers, &Health), With<Player>>,
    callbacks: Option<Res<AttributesCallbacks>>,
) {
    if !selected_stat.is_changed() {
        return;
    }

    if !q_dialog.is_empty() {
        return;
    }

    let Some(stat_type) = selected_stat.current else {
        return;
    };

    let Some(callbacks) = callbacks else {
        return;
    };

    let Ok((attributes, stats, stat_modifiers, health)) = q_player.single() else {
        return;
    };

    let screen_width = 80.0;
    let screen_height = 40.0;
    let dialog_width = 60.0;
    let dialog_height = 30.0;
    let dialog_x = (screen_width - dialog_width) / 2.0;
    let dialog_y = (screen_height - dialog_height) / 2.0;

    cmds.spawn((
        Glyph::new(6, Palette::Black, Palette::Black)
            .bg(Palette::Black)
            .alpha(0.7)
            .scale((screen_width, screen_height))
            .layer(Layer::DialogPanels),
        Position::new_f32(0.0, 0.0, 0.),
        FullScreenBackground,
        CleanupStatDialog,
    ));

    let stat_name = format!("{:?}", stat_type);
    let description = stat_type.description();
    let base = stat_type.get_base_value(attributes);
    let total = stats.get_stat(stat_type);
    let modifiers_list = stat_modifiers.modifiers.get(&stat_type);

    let mut y = dialog_y + 2.0;

    cmds.spawn((
        Text::new(&format!("=== {} ===", stat_name))
            .fg1(Palette::Yellow)
            .layer(Layer::DialogContent),
        Position::new_f32(dialog_x + 2.0, y, 0.),
        CleanupStatDialog,
    ));

    y += 1.0;

    cmds.spawn((
        Text::new(description)
            .fg1(Palette::DarkGray)
            .layer(Layer::DialogContent),
        Position::new_f32(dialog_x + 2.0, y, 0.),
        CleanupStatDialog,
    ));

    y += 1.5;

    cmds.spawn((
        Text::new(&format!("Base Value: {}", base))
            .fg1(Palette::White)
            .layer(Layer::DialogContent),
        Position::new_f32(dialog_x + 2.0, y, 0.),
        CleanupStatDialog,
    ));

    y += 1.0;

    if let Some(mods) = modifiers_list {
        if !mods.is_empty() {
            cmds.spawn((
                Text::new("Modifiers:")
                    .fg1(Palette::Cyan)
                    .layer(Layer::DialogContent),
                Position::new_f32(dialog_x + 2.0, y, 0.),
                CleanupStatDialog,
            ));

            y += 0.5;

            for modifier in mods {
                let source_text = match &modifier.source {
                    ModifierSource::Equipment { item_id } => format!("  Equipment #{}: {:+}", item_id, modifier.value),
                    ModifierSource::Intrinsic { name } => format!("  {}: {:+}", name, modifier.value),
                    ModifierSource::Condition { condition_id } => format!("  Condition {}: {:+}", condition_id, modifier.value),
                };

                cmds.spawn((
                    Text::new(&source_text)
                        .fg1(Palette::White)
                        .layer(Layer::DialogContent),
                    Position::new_f32(dialog_x + 2.0, y, 0.),
                    CleanupStatDialog,
                ));

                y += 0.5;
            }
        }
    }

    y += 0.5;

    if stat_type == StatType::Armor {
        let (current_armor, max_armor) = health.get_current_max_armor(stats);
        cmds.spawn((
            Text::new(&format!("Current Armor: {}/{}", current_armor, max_armor))
                .fg1(Palette::Green)
                .layer(Layer::DialogContent),
            Position::new_f32(dialog_x + 2.0, y, 0.),
            CleanupStatDialog,
        ));
        y += 0.5;
    }

    cmds.spawn((
        Text::new(&format!("Total: {}", total))
            .fg1(Palette::Green)
            .layer(Layer::DialogContent),
        Position::new_f32(dialog_x + 2.0, y, 0.),
        CleanupStatDialog,
    ));

    y += 2.0;

    cmds.spawn((
        Position::new_f32(dialog_x + 2.0, y, 0.),
        Button::new("({Y|ESC}) CLOSE", callbacks.close_stat_dialog).hotkey(KeyCode::Escape),
        CleanupStatDialog,
    ));
}

fn cleanup_stat_dialog(
    mut cmds: Commands,
    selected_stat: Res<SelectedStat>,
    q_dialog: Query<Entity, With<CleanupStatDialog>>,
) {
    if !selected_stat.is_changed() {
        return;
    }

    if selected_stat.current.is_some() {
        return;
    }

    for entity in q_dialog.iter() {
        cmds.entity(entity).despawn();
    }
}

fn setup_attributes_screen(
    mut cmds: Commands,
    q_player: Query<
        (
            &Level,
            &Attributes,
            &AttributePoints,
            &Stats,
            &StatModifiers,
            &Health,
        ),
        With<Player>,
    >,
    callbacks: Res<AttributesCallbacks>,
    selected: Res<SelectedAttribute>,
) {
    let Ok((level, attributes, attribute_points, stats, stat_modifiers, health)) =
        q_player.single()
    else {
        return;
    };

    let left_x = 2.0;
    let right_x = 32.0;
    let mut y_pos = 1.0;

    cmds.spawn((
        Text::new("=== PLAYER ATTRIBUTES & STATS ===")
            .fg1(Palette::Yellow)
            .layer(Layer::Ui),
        Position::new_f32(left_x, y_pos, 0.),
        CleanupStateAttributes,
    ));

    y_pos += 0.5;

    let level_display = cmds
        .spawn((
            Text::new(&format!("Level: {}", level.current_level))
                .fg1(Palette::White)
                .layer(Layer::Ui),
            Position::new_f32(left_x, y_pos, 0.),
            CleanupStateAttributes,
        ))
        .id();

    let available_points = cmds
        .spawn((
            Text::new(&format!("Available: {} pts", attribute_points.available))
                .fg1(Palette::Green)
                .layer(Layer::Ui),
            Position::new_f32(left_x + 15.0, y_pos, 0.),
            CleanupStateAttributes,
        ))
        .id();

    y_pos += 1.5;

    let strength_indicator = cmds
        .spawn((
            Text::new(if selected.current == AttributeGroup::Strength {
                "→"
            } else {
                " "
            })
            .fg1(Palette::Yellow)
            .layer(Layer::Ui),
            Position::new_f32(left_x, y_pos, 0.),
            CleanupStateAttributes,
        ))
        .id();

    let strength_value = cmds
        .spawn((
            Position::new_f32(left_x + 2.0, y_pos, 0.),
            Button::new(
                &format!("STRENGTH      {}", attributes.strength),
                callbacks.select_strength,
            )
            .hotkey(KeyCode::Key1),
            CleanupStateAttributes,
        ))
        .id();

    cmds.spawn((
        Position::new_f32(left_x + 18.0, y_pos, 0.),
        Button::new("[+]", callbacks.increase_strength),
        CleanupStateAttributes,
    ));

    cmds.spawn((
        Position::new_f32(left_x + 22.0, y_pos, 0.),
        Button::new("[-]", callbacks.decrease_strength),
        CleanupStateAttributes,
    ));

    y_pos += 0.5;

    cmds.spawn((
        Text::new("  Raw physical power")
            .fg1(Palette::DarkGray)
            .layer(Layer::Ui),
        Position::new_f32(left_x + 2.0, y_pos, 0.),
        CleanupStateAttributes,
    ));

    y_pos += 0.5;

    let dexterity_indicator = cmds
        .spawn((
            Text::new(if selected.current == AttributeGroup::Dexterity {
                "→"
            } else {
                " "
            })
            .fg1(Palette::Yellow)
            .layer(Layer::Ui),
            Position::new_f32(left_x, y_pos, 0.),
            CleanupStateAttributes,
        ))
        .id();

    let dexterity_value = cmds
        .spawn((
            Position::new_f32(left_x + 2.0, y_pos, 0.),
            Button::new(
                &format!("DEXTERITY     {}", attributes.dexterity),
                callbacks.select_dexterity,
            )
            .hotkey(KeyCode::Key2),
            CleanupStateAttributes,
        ))
        .id();

    cmds.spawn((
        Position::new_f32(left_x + 18.0, y_pos, 0.),
        Button::new("[+]", callbacks.increase_dexterity),
        CleanupStateAttributes,
    ));

    cmds.spawn((
        Position::new_f32(left_x + 22.0, y_pos, 0.),
        Button::new("[-]", callbacks.decrease_dexterity),
        CleanupStateAttributes,
    ));

    y_pos += 0.5;

    cmds.spawn((
        Text::new("  Agility & precision")
            .fg1(Palette::DarkGray)
            .layer(Layer::Ui),
        Position::new_f32(left_x + 2.0, y_pos, 0.),
        CleanupStateAttributes,
    ));

    y_pos += 0.5;

    let constitution_indicator = cmds
        .spawn((
            Text::new(if selected.current == AttributeGroup::Constitution {
                "→"
            } else {
                " "
            })
            .fg1(Palette::Yellow)
            .layer(Layer::Ui),
            Position::new_f32(left_x, y_pos, 0.),
            CleanupStateAttributes,
        ))
        .id();

    let constitution_value = cmds
        .spawn((
            Position::new_f32(left_x + 2.0, y_pos, 0.),
            Button::new(
                &format!("CONSTITUTION  {}", attributes.constitution),
                callbacks.select_constitution,
            )
            .hotkey(KeyCode::Key3),
            CleanupStateAttributes,
        ))
        .id();

    cmds.spawn((
        Position::new_f32(left_x + 18.0, y_pos, 0.),
        Button::new("[+]", callbacks.increase_constitution),
        CleanupStateAttributes,
    ));

    cmds.spawn((
        Position::new_f32(left_x + 22.0, y_pos, 0.),
        Button::new("[-]", callbacks.decrease_constitution),
        CleanupStateAttributes,
    ));

    y_pos += 0.5;

    cmds.spawn((
        Text::new("  Durability & health")
            .fg1(Palette::DarkGray)
            .layer(Layer::Ui),
        Position::new_f32(left_x + 2.0, y_pos, 0.),
        CleanupStateAttributes,
    ));

    y_pos += 0.5;

    let intelligence_indicator = cmds
        .spawn((
            Text::new(if selected.current == AttributeGroup::Intelligence {
                "→"
            } else {
                " "
            })
            .fg1(Palette::Yellow)
            .layer(Layer::Ui),
            Position::new_f32(left_x, y_pos, 0.),
            CleanupStateAttributes,
        ))
        .id();

    let intelligence_value = cmds
        .spawn((
            Position::new_f32(left_x + 2.0, y_pos, 0.),
            Button::new(
                &format!("INTELLIGENCE  {}", attributes.intelligence),
                callbacks.select_intelligence,
            )
            .hotkey(KeyCode::Key4),
            CleanupStateAttributes,
        ))
        .id();

    cmds.spawn((
        Position::new_f32(left_x + 18.0, y_pos, 0.),
        Button::new("[+]", callbacks.increase_intelligence),
        CleanupStateAttributes,
    ));

    cmds.spawn((
        Position::new_f32(left_x + 22.0, y_pos, 0.),
        Button::new("[-]", callbacks.decrease_intelligence),
        CleanupStateAttributes,
    ));

    y_pos += 0.5;

    cmds.spawn((
        Text::new("  Knowledge & tactics")
            .fg1(Palette::DarkGray)
            .layer(Layer::Ui),
        Position::new_f32(left_x + 2.0, y_pos, 0.),
        CleanupStateAttributes,
    ));

    y_pos += 0.5;

    let special_indicator = cmds
        .spawn((
            Text::new(if selected.current == AttributeGroup::Special {
                "→"
            } else {
                " "
            })
            .fg1(Palette::Yellow)
            .layer(Layer::Ui),
            Position::new_f32(left_x, y_pos, 0.),
            CleanupStateAttributes,
        ))
        .id();

    cmds.spawn((
        Position::new_f32(left_x + 2.0, y_pos, 0.),
        Button::new("SPECIAL", callbacks.select_special).hotkey(KeyCode::Key5),
        CleanupStateAttributes,
    ));

    y_pos += 1.0;

    cmds.spawn((
        Position::new_f32(left_x, y_pos, 0.),
        Button::new("[RESET ALL]", callbacks.reset_all),
        CleanupStateAttributes,
    ));

    y_pos += 1.5;

    cmds.spawn((
        Position::new_f32(left_x, y_pos, 0.),
        Button::new("({Y|ESC}) BACK", callbacks.back_to_explore).hotkey(KeyCode::Escape),
        CleanupStateAttributes,
    ));

    let (title_text, desc_text) = match selected.current {
        AttributeGroup::Strength => (
            "=== STRENGTH STATS ===",
            "Raw physical power affects weapon handling and melee damage.",
        ),
        AttributeGroup::Dexterity => (
            "=== DEXTERITY STATS ===",
            "Agility and precision affects speed, accuracy, and evasion.",
        ),
        AttributeGroup::Constitution => (
            "=== CONSTITUTION STATS ===",
            "Durability and health determines max HP and fortitude.",
        ),
        AttributeGroup::Intelligence => (
            "=== INTELLIGENCE STATS ===",
            "Knowledge and tactics affects armor regeneration.",
        ),
        AttributeGroup::Special => (
            "=== SPECIAL STATS ===",
            "Stats that don't derive from base attributes.",
        ),
    };

    y_pos = 3.0;

    let stat_section_title = cmds
        .spawn((
            Text::new(title_text).fg1(Palette::Cyan).layer(Layer::Ui),
            Position::new_f32(right_x, y_pos, 0.),
            CleanupStateAttributes,
        ))
        .id();

    y_pos += 0.5;

    let stat_section_desc = cmds
        .spawn((
            Text::new(desc_text).fg1(Palette::DarkGray).layer(Layer::Ui),
            Position::new_f32(right_x, y_pos, 0.),
            CleanupStateAttributes,
        ))
        .id();

    y_pos += 1.0;

    let mut stat_displays = HashMap::new();

    for stat_type in StatType::all() {
        let is_visible = stat_type.get_attribute_group() == selected.current;

        let base = stat_type.get_base_value(attributes);
        let modifiers = stat_modifiers.get_total_for_stat(*stat_type);
        let total = stats.get_stat(*stat_type);

        let stat_name = format!("{:?}", stat_type);

        let display_text = if *stat_type == StatType::Armor {
            let (current_armor, max_armor) = health.get_current_max_armor(stats);
            format!(
                "  {:14} {:3}  ({}/{} current | {} max | {:+} mods)",
                stat_name, total, current_armor, max_armor, total, modifiers
            )
        } else {
            format!(
                "  {:14} {:3}  ({} base {:+} mods)",
                stat_name, total, base, modifiers
            )
        };

        let position = if is_visible {
            Position::new_f32(right_x, y_pos, 0.)
        } else {
            Position::new_f32(-1000.0, -1000.0, 0.)
        };

        let callback = *callbacks.select_stat.get(stat_type).unwrap();

        let entity = cmds
            .spawn((
                position,
                Button::new(&display_text, callback),
                CleanupStateAttributes,
            ))
            .id();

        stat_displays.insert(*stat_type, entity);

        if is_visible {
            y_pos += 0.5;
        }
    }

    cmds.insert_resource(AttributesUIEntities {
        level_display,
        available_points,
        strength_value,
        dexterity_value,
        constitution_value,
        intelligence_value,
        strength_indicator,
        dexterity_indicator,
        constitution_indicator,
        intelligence_indicator,
        special_indicator,
        stat_displays,
        stat_section_title,
        stat_section_desc,
    });
}

fn setup_attributes_background(mut cmds: Commands, screen: Res<ScreenSize>) {
    let color = Palette::Clear;
    cmds.spawn((
        FullScreenBackground,
        CleanupStateAttributes,
        Position::new(0, 0, 0),
        Glyph::new(6, color, color)
            .bg(color)
            .scale((screen.tile_w as f32, screen.tile_h as f32))
            .layer(Layer::UiPanels),
    ));
}
