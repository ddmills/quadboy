use bevy_ecs::{prelude::*, system::SystemId};
use macroquad::input::KeyCode;

use crate::{
    common::Palette,
    domain::{
        AttributePoints, Attributes, Health, Level, Player, StatModifiers, StatType, Stats,
        game_loop,
    },
    engine::{App, Plugin},
    rendering::{Layer, Position, Text},
    states::{CurrentGameState, GameState, GameStatePlugin, cleanup_system},
    ui::Button,
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
}

#[derive(Component, Clone)]
pub struct CleanupStateAttributes;

#[derive(Resource)]
struct AttributesUIEntities {
    level_display: Entity,
    available_points: Entity,
    strength_value: Entity,
    dexterity_value: Entity,
    constitution_value: Entity,
    intelligence_value: Entity,
    fortitude_display: Entity,
    speed_display: Entity,
    armor_display: Entity,
    armor_regen_display: Entity,
    rifle_display: Entity,
    shotgun_display: Entity,
    pistol_display: Entity,
    blade_display: Entity,
    cudgel_display: Entity,
    unarmed_display: Entity,
    dodge_display: Entity,
    reload_speed_display: Entity,
}

pub struct AttributesStatePlugin;

impl Plugin for AttributesStatePlugin {
    fn build(&self, app: &mut App) {
        GameStatePlugin::new(GameState::Attributes)
            .on_enter(
                app,
                (setup_attributes_callbacks, setup_attributes_screen).chain(),
            )
            .on_update(app, (game_loop, update_attributes_display).chain())
            .on_leave(
                app,
                (
                    cleanup_system::<CleanupStateAttributes>,
                    remove_attributes_callbacks,
                    remove_attributes_ui_entities,
                )
                    .chain(),
            );
    }
}

fn setup_attributes_callbacks(world: &mut World) {
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
    };

    world.insert_resource(callbacks);
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

fn update_attributes_display(
    mut q_text: Query<&mut Text>,
    q_player: Query<
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
                Changed<Health>,
            )>,
        ),
    >,
    ui_entities: Option<Res<AttributesUIEntities>>,
) {
    let Some(ui_entities) = ui_entities else {
        return;
    };

    // Only update if player attributes/points/health changed
    let Ok((level, attributes, attribute_points, stats, stat_modifiers, health)) =
        q_player.single()
    else {
        return;
    };

    // Update level display
    if let Ok(mut text) = q_text.get_mut(ui_entities.level_display) {
        text.value = format!("Level: {}", level.current_level);
    }

    // Update available points display
    if let Ok(mut text) = q_text.get_mut(ui_entities.available_points) {
        text.value = format!("Available Points: {}", attribute_points.available);
    }

    // Update attribute values
    if let Ok(mut text) = q_text.get_mut(ui_entities.strength_value) {
        text.value = format!("Strength:     {}", attributes.strength);
    }

    if let Ok(mut text) = q_text.get_mut(ui_entities.dexterity_value) {
        text.value = format!("Dexterity:    {}", attributes.dexterity);
    }

    if let Ok(mut text) = q_text.get_mut(ui_entities.constitution_value) {
        text.value = format!("Constitution: {}", attributes.constitution);
    }

    if let Ok(mut text) = q_text.get_mut(ui_entities.intelligence_value) {
        text.value = format!("Intelligence: {}", attributes.intelligence);
    }

    // Update calculated stats
    let fortitude_base = StatType::Fortitude.get_base_value(attributes);
    let fortitude_modifiers = stat_modifiers.get_total_for_stat(StatType::Fortitude);
    let fortitude_total = stats.get_stat(StatType::Fortitude);

    if let Ok(mut text) = q_text.get_mut(ui_entities.fortitude_display) {
        text.value = format!(
            "Fortitude:    {}  (Constitution: {} + Modifiers: {:+})",
            fortitude_total, fortitude_base, fortitude_modifiers
        );
    }

    let speed_base = StatType::Speed.get_base_value(attributes);
    let speed_modifiers = stat_modifiers.get_total_for_stat(StatType::Speed);
    let speed_total = stats.get_stat(StatType::Speed);

    if let Ok(mut text) = q_text.get_mut(ui_entities.speed_display) {
        text.value = format!(
            "Speed:        {}  (Dexterity: {} + Modifiers: {:+})",
            speed_total, speed_base, speed_modifiers
        );
    }

    let armor_base = StatType::Armor.get_base_value(attributes);
    let armor_modifiers = stat_modifiers.get_total_for_stat(StatType::Armor);
    let armor_total = stats.get_stat(StatType::Armor);

    if let Ok(mut text) = q_text.get_mut(ui_entities.armor_display) {
        let (current_armor, max_armor) = health.get_current_max_armor(stats);
        text.value = format!(
            "Armor:        {}/{}  (Max: {} | Modifiers: {:+})",
            current_armor, max_armor, armor_total, armor_modifiers
        );
    }

    let armor_regen_base = StatType::ArmorRegen.get_base_value(attributes);
    let armor_regen_modifiers = stat_modifiers.get_total_for_stat(StatType::ArmorRegen);
    let armor_regen_total = stats.get_stat(StatType::ArmorRegen);

    if let Ok(mut text) = q_text.get_mut(ui_entities.armor_regen_display) {
        text.value = format!(
            "Armor Regen:  {}  (Intelligence: {} + Modifiers: {:+})",
            armor_regen_total, armor_regen_base, armor_regen_modifiers
        );
    }

    // Update weapon family stats
    let rifle_base = StatType::Rifle.get_base_value(attributes);
    let rifle_modifiers = stat_modifiers.get_total_for_stat(StatType::Rifle);
    let rifle_total = stats.get_stat(StatType::Rifle);

    if let Ok(mut text) = q_text.get_mut(ui_entities.rifle_display) {
        text.value = format!(
            "Rifle:        {}  (Dexterity: {} + Modifiers: {:+})",
            rifle_total, rifle_base, rifle_modifiers
        );
    }

    let shotgun_base = StatType::Shotgun.get_base_value(attributes);
    let shotgun_modifiers = stat_modifiers.get_total_for_stat(StatType::Shotgun);
    let shotgun_total = stats.get_stat(StatType::Shotgun);

    if let Ok(mut text) = q_text.get_mut(ui_entities.shotgun_display) {
        text.value = format!(
            "Shotgun:      {}  (Strength: {} + Modifiers: {:+})",
            shotgun_total, shotgun_base, shotgun_modifiers
        );
    }

    let pistol_base = StatType::Pistol.get_base_value(attributes);
    let pistol_modifiers = stat_modifiers.get_total_for_stat(StatType::Pistol);
    let pistol_total = stats.get_stat(StatType::Pistol);

    if let Ok(mut text) = q_text.get_mut(ui_entities.pistol_display) {
        text.value = format!(
            "Pistol:       {}  (Strength: {} + Modifiers: {:+})",
            pistol_total, pistol_base, pistol_modifiers
        );
    }

    let blade_base = StatType::Blade.get_base_value(attributes);
    let blade_modifiers = stat_modifiers.get_total_for_stat(StatType::Blade);
    let blade_total = stats.get_stat(StatType::Blade);

    if let Ok(mut text) = q_text.get_mut(ui_entities.blade_display) {
        text.value = format!(
            "Blade:        {}  (Dexterity: {} + Modifiers: {:+})",
            blade_total, blade_base, blade_modifiers
        );
    }

    let cudgel_base = StatType::Cudgel.get_base_value(attributes);
    let cudgel_modifiers = stat_modifiers.get_total_for_stat(StatType::Cudgel);
    let cudgel_total = stats.get_stat(StatType::Cudgel);

    if let Ok(mut text) = q_text.get_mut(ui_entities.cudgel_display) {
        text.value = format!(
            "Cudgel:       {}  (Strength: {} + Modifiers: {:+})",
            cudgel_total, cudgel_base, cudgel_modifiers
        );
    }

    let unarmed_base = StatType::Unarmed.get_base_value(attributes);
    let unarmed_modifiers = stat_modifiers.get_total_for_stat(StatType::Unarmed);
    let unarmed_total = stats.get_stat(StatType::Unarmed);

    if let Ok(mut text) = q_text.get_mut(ui_entities.unarmed_display) {
        text.value = format!(
            "Unarmed:      {}  (Strength: {} + Modifiers: {:+})",
            unarmed_total, unarmed_base, unarmed_modifiers
        );
    }

    let dodge_base = StatType::Dodge.get_base_value(attributes);
    let dodge_modifiers = stat_modifiers.get_total_for_stat(StatType::Dodge);
    let dodge_total = stats.get_stat(StatType::Dodge);

    if let Ok(mut text) = q_text.get_mut(ui_entities.dodge_display) {
        text.value = format!(
            "Dodge:        {}  (Dexterity: {} + Modifiers: {:+})",
            dodge_total, dodge_base, dodge_modifiers
        );
    }

    let reload_speed_base = StatType::ReloadSpeed.get_base_value(attributes);
    let reload_speed_modifiers = stat_modifiers.get_total_for_stat(StatType::ReloadSpeed);
    let reload_speed_total = stats.get_stat(StatType::ReloadSpeed);

    if let Ok(mut text) = q_text.get_mut(ui_entities.reload_speed_display) {
        text.value = format!(
            "Reload Speed: {}  (Dexterity: {} + Modifiers: {:+})",
            reload_speed_total, reload_speed_base, reload_speed_modifiers
        );
    }
}

fn remove_attributes_callbacks(mut cmds: Commands) {
    cmds.remove_resource::<AttributesCallbacks>();
}

fn remove_attributes_ui_entities(mut cmds: Commands) {
    cmds.remove_resource::<AttributesUIEntities>();
}

fn setup_attributes_screen(
    mut cmds: Commands,
    q_player: Query<
        (
            Entity,
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
) {
    let Ok((_, level, attributes, attribute_points, stats, stat_modifiers, health)) =
        q_player.single()
    else {
        return;
    };

    let left_x = 2.0;
    let mut y_pos = 1.0;

    // Title
    cmds.spawn((
        Text::new("PLAYER ATTRIBUTES & STATS")
            .fg1(Palette::Yellow)
            .bg(Palette::Black)
            .layer(Layer::Ui),
        Position::new_f32(left_x, y_pos, 0.),
        CleanupStateAttributes,
    ));

    y_pos += 1.0;

    // Level display
    let level_display = cmds
        .spawn((
            Text::new(&format!("Level: {}", level.current_level))
                .fg1(Palette::White)
                .layer(Layer::Ui),
            Position::new_f32(left_x, y_pos, 0.),
            CleanupStateAttributes,
        ))
        .id();

    y_pos += 0.5;

    // Available points display
    let available_points = cmds
        .spawn((
            Text::new(&format!("Available Points: {}", attribute_points.available))
                .fg1(Palette::Green)
                .layer(Layer::Ui),
            Position::new_f32(left_x, y_pos, 0.),
            CleanupStateAttributes,
        ))
        .id();

    y_pos += 1.0;

    // Base Attributes Section
    cmds.spawn((
        Text::new("=== BASE ATTRIBUTES ===")
            .fg1(Palette::Cyan)
            .layer(Layer::Ui),
        Position::new_f32(left_x, y_pos, 0.),
        CleanupStateAttributes,
    ));

    y_pos += 0.5;

    // Strength row
    let strength_value = cmds
        .spawn((
            Text::new(&format!("Strength:     {}", attributes.strength))
                .fg1(Palette::White)
                .layer(Layer::Ui),
            Position::new_f32(left_x, y_pos, 0.),
            CleanupStateAttributes,
        ))
        .id();

    cmds.spawn((
        Position::new_f32(left_x + 18.0, y_pos, 0.),
        Button::new("[+]", callbacks.increase_strength),
        CleanupStateAttributes,
    ));

    cmds.spawn((
        Position::new_f32(left_x + 20.0, y_pos, 0.),
        Button::new("[-]", callbacks.decrease_strength),
        CleanupStateAttributes,
    ));

    y_pos += 0.5;

    // Dexterity row
    let dexterity_value = cmds
        .spawn((
            Text::new(&format!("Dexterity:    {}", attributes.dexterity))
                .fg1(Palette::White)
                .layer(Layer::Ui),
            Position::new_f32(left_x, y_pos, 0.),
            CleanupStateAttributes,
        ))
        .id();

    cmds.spawn((
        Position::new_f32(left_x + 18.0, y_pos, 0.),
        Button::new("[+]", callbacks.increase_dexterity),
        CleanupStateAttributes,
    ));

    cmds.spawn((
        Position::new_f32(left_x + 20.0, y_pos, 0.),
        Button::new("[-]", callbacks.decrease_dexterity),
        CleanupStateAttributes,
    ));

    y_pos += 0.5;

    // Constitution row
    let constitution_value = cmds
        .spawn((
            Text::new(&format!("Constitution: {}", attributes.constitution))
                .fg1(Palette::White)
                .layer(Layer::Ui),
            Position::new_f32(left_x, y_pos, 0.),
            CleanupStateAttributes,
        ))
        .id();

    cmds.spawn((
        Position::new_f32(left_x + 18.0, y_pos, 0.),
        Button::new("[+]", callbacks.increase_constitution),
        CleanupStateAttributes,
    ));

    cmds.spawn((
        Position::new_f32(left_x + 20.0, y_pos, 0.),
        Button::new("[-]", callbacks.decrease_constitution),
        CleanupStateAttributes,
    ));

    y_pos += 0.5;

    // Intelligence row
    let intelligence_value = cmds
        .spawn((
            Text::new(&format!("Intelligence: {}", attributes.intelligence))
                .fg1(Palette::White)
                .layer(Layer::Ui),
            Position::new_f32(left_x, y_pos, 0.),
            CleanupStateAttributes,
        ))
        .id();

    cmds.spawn((
        Position::new_f32(left_x + 18.0, y_pos, 0.),
        Button::new("[+]", callbacks.increase_intelligence),
        CleanupStateAttributes,
    ));

    cmds.spawn((
        Position::new_f32(left_x + 20.0, y_pos, 0.),
        Button::new("[-]", callbacks.decrease_intelligence),
        CleanupStateAttributes,
    ));

    y_pos += 1.0;

    // Reset All button
    cmds.spawn((
        Position::new_f32(left_x, y_pos, 0.),
        Button::new("[RESET ALL]", callbacks.reset_all),
        CleanupStateAttributes,
    ));

    y_pos += 1.0;

    // Calculated Stats Section
    cmds.spawn((
        Text::new("=== CALCULATED STATS ===")
            .fg1(Palette::Cyan)
            .layer(Layer::Ui),
        Position::new_f32(left_x, y_pos, 0.),
        CleanupStateAttributes,
    ));

    y_pos += 0.5;

    // Fortitude
    let fortitude_base = StatType::Fortitude.get_base_value(attributes);
    let fortitude_modifiers = stat_modifiers.get_total_for_stat(StatType::Fortitude);
    let fortitude_total = stats.get_stat(StatType::Fortitude);

    let fortitude_display = cmds
        .spawn((
            Text::new(&format!(
                "Fortitude:    {}  (Constitution: {} + Modifiers: {:+})",
                fortitude_total, fortitude_base, fortitude_modifiers
            ))
            .fg1(Palette::White)
            .layer(Layer::Ui),
            Position::new_f32(left_x, y_pos, 0.),
            CleanupStateAttributes,
        ))
        .id();

    y_pos += 0.5;

    // Speed
    let speed_base = StatType::Speed.get_base_value(attributes);
    let speed_modifiers = stat_modifiers.get_total_for_stat(StatType::Speed);
    let speed_total = stats.get_stat(StatType::Speed);

    let speed_display = cmds
        .spawn((
            Text::new(&format!(
                "Speed:        {}  (Dexterity: {} + Modifiers: {:+})",
                speed_total, speed_base, speed_modifiers
            ))
            .fg1(Palette::White)
            .layer(Layer::Ui),
            Position::new_f32(left_x, y_pos, 0.),
            CleanupStateAttributes,
        ))
        .id();

    y_pos += 0.5;

    // Armor
    let armor_base = StatType::Armor.get_base_value(attributes);
    let armor_modifiers = stat_modifiers.get_total_for_stat(StatType::Armor);
    let armor_total = stats.get_stat(StatType::Armor);

    let armor_display = cmds
        .spawn((
            Text::new(&format!(
                "Armor:        {}/{}  (Max: {} | Modifiers: {:+})",
                health.current_armor, armor_total, armor_total, armor_modifiers
            ))
            .fg1(Palette::White)
            .layer(Layer::Ui),
            Position::new_f32(left_x, y_pos, 0.),
            CleanupStateAttributes,
        ))
        .id();

    y_pos += 0.5;

    // Armor Regen
    let armor_regen_base = StatType::ArmorRegen.get_base_value(attributes);
    let armor_regen_modifiers = stat_modifiers.get_total_for_stat(StatType::ArmorRegen);
    let armor_regen_total = stats.get_stat(StatType::ArmorRegen);

    let armor_regen_display = cmds
        .spawn((
            Text::new(&format!(
                "Armor Regen:  {}  (Intelligence: {} + Modifiers: {:+})",
                armor_regen_total, armor_regen_base, armor_regen_modifiers
            ))
            .fg1(Palette::White)
            .layer(Layer::Ui),
            Position::new_f32(left_x, y_pos, 0.),
            CleanupStateAttributes,
        ))
        .id();

    y_pos += 0.5;

    // Reload Speed
    let reload_speed_base = StatType::ReloadSpeed.get_base_value(attributes);
    let reload_speed_modifiers = stat_modifiers.get_total_for_stat(StatType::ReloadSpeed);
    let reload_speed_total = stats.get_stat(StatType::ReloadSpeed);

    let reload_speed_display = cmds
        .spawn((
            Text::new(&format!(
                "Reload Speed: {}  (Dexterity: {} + Modifiers: {:+})",
                reload_speed_total, reload_speed_base, reload_speed_modifiers
            ))
            .fg1(Palette::White)
            .layer(Layer::Ui),
            Position::new_f32(left_x, y_pos, 0.),
            CleanupStateAttributes,
        ))
        .id();

    y_pos += 1.0;

    // Weapon Proficiencies Section
    cmds.spawn((
        Text::new("=== WEAPON PROFICIENCIES ===")
            .fg1(Palette::Cyan)
            .layer(Layer::Ui),
        Position::new_f32(left_x, y_pos, 0.),
        CleanupStateAttributes,
    ));

    y_pos += 0.5;

    // Rifle
    let rifle_base = StatType::Rifle.get_base_value(attributes);
    let rifle_modifiers = stat_modifiers.get_total_for_stat(StatType::Rifle);
    let rifle_total = stats.get_stat(StatType::Rifle);

    let rifle_display = cmds
        .spawn((
            Text::new(&format!(
                "Rifle:        {}  (Dexterity: {} + Modifiers: {:+})",
                rifle_total, rifle_base, rifle_modifiers
            ))
            .fg1(Palette::White)
            .layer(Layer::Ui),
            Position::new_f32(left_x, y_pos, 0.),
            CleanupStateAttributes,
        ))
        .id();

    y_pos += 0.5;

    // Shotgun
    let shotgun_base = StatType::Shotgun.get_base_value(attributes);
    let shotgun_modifiers = stat_modifiers.get_total_for_stat(StatType::Shotgun);
    let shotgun_total = stats.get_stat(StatType::Shotgun);

    let shotgun_display = cmds
        .spawn((
            Text::new(&format!(
                "Shotgun:      {}  (Strength: {} + Modifiers: {:+})",
                shotgun_total, shotgun_base, shotgun_modifiers
            ))
            .fg1(Palette::White)
            .layer(Layer::Ui),
            Position::new_f32(left_x, y_pos, 0.),
            CleanupStateAttributes,
        ))
        .id();

    y_pos += 0.5;

    // Pistol
    let pistol_base = StatType::Pistol.get_base_value(attributes);
    let pistol_modifiers = stat_modifiers.get_total_for_stat(StatType::Pistol);
    let pistol_total = stats.get_stat(StatType::Pistol);

    let pistol_display = cmds
        .spawn((
            Text::new(&format!(
                "Pistol:       {}  (Strength: {} + Modifiers: {:+})",
                pistol_total, pistol_base, pistol_modifiers
            ))
            .fg1(Palette::White)
            .layer(Layer::Ui),
            Position::new_f32(left_x, y_pos, 0.),
            CleanupStateAttributes,
        ))
        .id();

    y_pos += 0.5;

    // Blade
    let blade_base = StatType::Blade.get_base_value(attributes);
    let blade_modifiers = stat_modifiers.get_total_for_stat(StatType::Blade);
    let blade_total = stats.get_stat(StatType::Blade);

    let blade_display = cmds
        .spawn((
            Text::new(&format!(
                "Blade:        {}  (Dexterity: {} + Modifiers: {:+})",
                blade_total, blade_base, blade_modifiers
            ))
            .fg1(Palette::White)
            .layer(Layer::Ui),
            Position::new_f32(left_x, y_pos, 0.),
            CleanupStateAttributes,
        ))
        .id();

    y_pos += 0.5;

    // Cudgel
    let cudgel_base = StatType::Cudgel.get_base_value(attributes);
    let cudgel_modifiers = stat_modifiers.get_total_for_stat(StatType::Cudgel);
    let cudgel_total = stats.get_stat(StatType::Cudgel);

    let cudgel_display = cmds
        .spawn((
            Text::new(&format!(
                "Cudgel:       {}  (Strength: {} + Modifiers: {:+})",
                cudgel_total, cudgel_base, cudgel_modifiers
            ))
            .fg1(Palette::White)
            .layer(Layer::Ui),
            Position::new_f32(left_x, y_pos, 0.),
            CleanupStateAttributes,
        ))
        .id();

    y_pos += 0.5;

    // Unarmed
    let unarmed_base = StatType::Unarmed.get_base_value(attributes);
    let unarmed_modifiers = stat_modifiers.get_total_for_stat(StatType::Unarmed);
    let unarmed_total = stats.get_stat(StatType::Unarmed);

    let unarmed_display = cmds
        .spawn((
            Text::new(&format!(
                "Unarmed:      {}  (Strength: {} + Modifiers: {:+})",
                unarmed_total, unarmed_base, unarmed_modifiers
            ))
            .fg1(Palette::White)
            .layer(Layer::Ui),
            Position::new_f32(left_x, y_pos, 0.),
            CleanupStateAttributes,
        ))
        .id();

    y_pos += 0.5;

    // Dodge
    let dodge_base = StatType::Dodge.get_base_value(attributes);
    let dodge_modifiers = stat_modifiers.get_total_for_stat(StatType::Dodge);
    let dodge_total = stats.get_stat(StatType::Dodge);

    let dodge_display = cmds
        .spawn((
            Text::new(&format!(
                "Dodge:        {}  (Dexterity: {} + Modifiers: {:+})",
                dodge_total, dodge_base, dodge_modifiers
            ))
            .fg1(Palette::White)
            .layer(Layer::Ui),
            Position::new_f32(left_x, y_pos, 0.),
            CleanupStateAttributes,
        ))
        .id();

    y_pos += 1.5;

    // Back Button
    cmds.spawn((
        Position::new_f32(left_x, y_pos, 0.),
        Button::new("({Y|ESC}) BACK", callbacks.back_to_explore).hotkey(KeyCode::Escape),
        CleanupStateAttributes,
    ));

    // Insert the resource with all entity IDs for dynamic updates
    cmds.insert_resource(AttributesUIEntities {
        level_display,
        available_points,
        strength_value,
        dexterity_value,
        constitution_value,
        intelligence_value,
        fortitude_display,
        speed_display,
        armor_display,
        armor_regen_display,
        rifle_display,
        shotgun_display,
        pistol_display,
        blade_display,
        cudgel_display,
        unarmed_display,
        dodge_display,
        reload_speed_display,
    });
}
