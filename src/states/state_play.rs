use bevy_ecs::prelude::*;
use macroquad::prelude::trace;

use crate::{common::Palette, domain::{activate_zones_by_player, load_nearby_zones, on_load_zone, on_set_zone_status, on_spawn_zone, on_unload_zone, Player, PlayerMovedEvent}, engine::{App, Plugin, ScheduleType}, rendering::{on_zone_status_change, update_camera, Glyph, Position, RenderLayer, ScreenSize}, states::{cleanup_system, enter_app_state, in_app_state, leave_app_state, AppState, AppStatePlugin, CurrentGameState, GameState}, ui::{draw_ui_panels, update_ui_layout, UiLayout}};

pub struct PlayStatePlugin;

impl Plugin for PlayStatePlugin {
    fn build(&self, app: &mut App) {
        AppStatePlugin::new(AppState::Play)
            .on_enter(app, (
                update_ui_layout,
                draw_ui_panels,
                spawn_stuff,
                ).chain()
            )
            .on_update(app, (
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
                update_camera)
            )
            .on_leave(app, (
                    on_leave_play_state,
                    cleanup_system::<CleanupStatePlay>
                ).chain()
            );
    }
}

#[derive(Component)]
pub struct CleanupStatePlay;

fn spawn_stuff(
    mut cmds: Commands,
    mut e_player_moved: EventWriter<PlayerMovedEvent>,
    mut game_state: ResMut<CurrentGameState>,
) {
    trace!("EnterAppState::<Play>");

    cmds.spawn((
        Position::new(15, 12, 0),
        Glyph::new(4, Palette::Orange, Palette::Green)
            .layer(RenderLayer::Actors)
            .bg(Palette::White)
            .outline(Palette::Red),
        CleanupStatePlay,
    ));

    cmds.spawn((
        Position::new(56, 56, 0),
        Glyph::new(147, Palette::Yellow, Palette::LightBlue)
            .layer(RenderLayer::Actors),
        Player,
        CleanupStatePlay,
    ));

    e_player_moved.write(PlayerMovedEvent { x: 56, y: 56, z: 0 });

    game_state.next = GameState::Explore;
}

fn on_leave_play_state(mut game_state: ResMut<CurrentGameState>)
{
    trace!("LeaveAppState::<Play>");
    game_state.next = GameState::None;
}
