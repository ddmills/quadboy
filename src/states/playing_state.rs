use bevy_ecs::prelude::*;
use macroquad::prelude::trace;

use crate::{common::Palette, domain::{activate_zones_by_player, load_nearby_zones, on_load_zone, on_set_zone_status, on_spawn_zone, on_unload_zone, player_input, render_player_debug, Player, PlayerDebug, PlayerMovedEvent}, engine::{App, Plugin, ScheduleType}, rendering::{on_zone_status_change, update_camera, Glyph, Position, RenderLayer, ScreenSize, Text}, states::{cleanup_system, enter_state, in_state, leave_state, GameState}, ui::{draw_ui_panels, update_ui_layout, UiLayout}};

pub struct PlayingStatePlugin;

impl Plugin for PlayingStatePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(ScheduleType::PreUpdate, (
                update_ui_layout,
                draw_ui_panels,
                spawn_stuff,
            ).chain().run_if(enter_state(GameState::Playing)))
            .add_systems(ScheduleType::Update, (
                (
                    activate_zones_by_player,
                    load_nearby_zones,
                    on_load_zone,
                    on_spawn_zone,
                    on_unload_zone,
                    on_set_zone_status,
                    on_zone_status_change,
                ).chain(),
                update_ui_layout.run_if(resource_changed::<ScreenSize>),
                draw_ui_panels.run_if(resource_changed::<UiLayout>),
                player_input,
                update_camera,
                render_player_debug,
            ).run_if(in_state(GameState::Playing)))
            .add_systems(ScheduleType::PostUpdate, cleanup_system::<CleanupPlaying>.run_if(leave_state(GameState::Playing)));
    }
}

#[derive(Component)]
pub struct CleanupPlaying;

fn spawn_stuff(mut cmds: Commands, mut e_player_moved: EventWriter<PlayerMovedEvent>)
{
    trace!("enter playing");

    cmds.spawn((
        Text::new("123")
            .fg1(Palette::White)
            .bg(Palette::Purple)
            .layer(RenderLayer::Ui),
        Position::new_f32(0., 0.5, 0.),
        PlayerDebug,
        CleanupPlaying,
    ));
    
    cmds.spawn((
        Position::new(15, 12, 0),
        Glyph::new(4, Palette::Orange, Palette::Green)
            .layer(RenderLayer::Actors)
            .bg(Palette::White)
            .outline(Palette::Red),
        CleanupPlaying,
    ));

    cmds.spawn((
        Position::new(56, 56, 0),
        Glyph::new(147, Palette::Yellow, Palette::LightBlue)
            .layer(RenderLayer::Actors),
        Player,
        CleanupPlaying,
    ));
    e_player_moved.write(PlayerMovedEvent { x: 56, y: 56, z: 0 });

    let hp = (9.5, 0.5);

    cmds.spawn((
        Text::new("HP             ")
            .fg1(Palette::White)
            .bg(Palette::Red)
            .layer(RenderLayer::Ui),
        Position::new_f32(hp.0, hp.1, 0.),
        CleanupPlaying,
    ));

    cmds.spawn((
        Text::new("            ")
            .fg1(Palette::Black)
            .bg(Palette::Gray)
            .layer(RenderLayer::Ui),
        Position::new_f32(hp.0 + 7.5, hp.1, 0.),
        CleanupPlaying,
    ));
}
