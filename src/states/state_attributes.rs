use bevy_ecs::{prelude::*, system::SystemId};
use macroquad::input::KeyCode;

use crate::{
    common::Palette,
    domain::{Attributes, Player, StatType, Stats, StatModifiers},
    engine::{App, Plugin},
    rendering::{Layer, Position, Text},
    states::{CurrentGameState, GameState, GameStatePlugin, cleanup_system},
    ui::Button,
};

#[derive(Resource)]
struct AttributesCallbacks {
    back_to_explore: SystemId,
}

#[derive(Component, Clone)]
pub struct CleanupStateAttributes;

pub struct AttributesStatePlugin;

impl Plugin for AttributesStatePlugin {
    fn build(&self, app: &mut App) {
        GameStatePlugin::new(GameState::Attributes)
            .on_enter(app, (setup_attributes_callbacks, setup_attributes_screen).chain())
            .on_leave(
                app,
                (
                    cleanup_system::<CleanupStateAttributes>,
                    remove_attributes_callbacks,
                ).chain(),
            );
    }
}

fn setup_attributes_callbacks(world: &mut World) {
    let callbacks = AttributesCallbacks {
        back_to_explore: world.register_system(back_to_explore),
    };

    world.insert_resource(callbacks);
}

fn back_to_explore(mut game_state: ResMut<CurrentGameState>) {
    game_state.next = GameState::Explore;
}

fn remove_attributes_callbacks(mut cmds: Commands) {
    cmds.remove_resource::<AttributesCallbacks>();
}

fn setup_attributes_screen(
    mut cmds: Commands,
    q_player: Query<(Entity, &Attributes, &Stats, &StatModifiers), With<Player>>,
    callbacks: Res<AttributesCallbacks>,
) {
    let Ok((_, attributes, stats, stat_modifiers)) = q_player.single() else {
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

    y_pos += 1.5;

    // Base Attributes Section
    cmds.spawn((
        Text::new("=== BASE ATTRIBUTES ===")
            .fg1(Palette::Cyan)
            .layer(Layer::Ui),
        Position::new_f32(left_x, y_pos, 0.),
        CleanupStateAttributes,
    ));

    y_pos += 0.5;

    cmds.spawn((
        Text::new(&format!("Strength:     {}", attributes.strength))
            .fg1(Palette::White)
            .layer(Layer::Ui),
        Position::new_f32(left_x, y_pos, 0.),
        CleanupStateAttributes,
    ));

    y_pos += 0.5;

    cmds.spawn((
        Text::new(&format!("Dexterity:    {}", attributes.dexterity))
            .fg1(Palette::White)
            .layer(Layer::Ui),
        Position::new_f32(left_x, y_pos, 0.),
        CleanupStateAttributes,
    ));

    y_pos += 0.5;

    cmds.spawn((
        Text::new(&format!("Constitution: {}", attributes.constitution))
            .fg1(Palette::White)
            .layer(Layer::Ui),
        Position::new_f32(left_x, y_pos, 0.),
        CleanupStateAttributes,
    ));

    y_pos += 0.5;

    cmds.spawn((
        Text::new(&format!("Intelligence: {}", attributes.intelligence))
            .fg1(Palette::White)
            .layer(Layer::Ui),
        Position::new_f32(left_x, y_pos, 0.),
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

    cmds.spawn((
        Text::new(&format!(
            "Fortitude:    {}  (Constitution: {} + Modifiers: {:+})",
            fortitude_total, fortitude_base, fortitude_modifiers
        ))
        .fg1(Palette::White)
        .layer(Layer::Ui),
        Position::new_f32(left_x, y_pos, 0.),
        CleanupStateAttributes,
    ));

    y_pos += 0.5;

    // Speed
    let speed_base = StatType::Speed.get_base_value(attributes);
    let speed_modifiers = stat_modifiers.get_total_for_stat(StatType::Speed);
    let speed_total = stats.get_stat(StatType::Speed);

    cmds.spawn((
        Text::new(&format!(
            "Speed:        {}  (Dexterity: {} + Modifiers: {:+})",
            speed_total, speed_base, speed_modifiers
        ))
        .fg1(Palette::White)
        .layer(Layer::Ui),
        Position::new_f32(left_x, y_pos, 0.),
        CleanupStateAttributes,
    ));

    y_pos += 1.5;

    // Back Button
    cmds.spawn((
        Position::new_f32(left_x, y_pos, 0.),
        Button::new("({Y|ESC}) BACK", callbacks.back_to_explore).hotkey(KeyCode::Escape),
        CleanupStateAttributes,
    ));
}

