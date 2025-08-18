use bevy_ecs::prelude::*;
use macroquad::prelude::trace;
use serde::{Deserialize, Serialize};

use crate::{
    common::Palette,
    domain::{
        Player, PlayerMovedEvent, activate_zones_by_player, load_nearby_zones, on_load_zone,
        on_set_zone_status, on_unload_zone,
    },
    engine::{App, Plugin, SerializableComponent},
    rendering::{
        Glyph, Position, RenderLayer, ScreenSize, on_zone_status_change, update_camera,
        update_entity_pos,
    },
    states::{AppState, AppStatePlugin, CurrentGameState, GameState, cleanup_system},
    ui::{UiLayout, draw_ui_panels, update_ui_layout},
};

pub struct PlayStatePlugin;

impl Plugin for PlayStatePlugin {
    fn build(&self, app: &mut App) {
        AppStatePlugin::new(AppState::Play)
            .on_enter(app, (update_ui_layout, draw_ui_panels, spawn_stuff).chain())
            .on_update(
                app,
                (
                    (
                        activate_zones_by_player,
                        load_nearby_zones,
                        update_entity_pos,
                        on_load_zone,
                        on_unload_zone,
                        on_set_zone_status,
                        on_zone_status_change,
                        update_camera,
                    )
                        .chain(),
                    update_ui_layout.run_if(resource_changed::<ScreenSize>),
                    draw_ui_panels.run_if(resource_changed::<UiLayout>),
                ),
            )
            .on_leave(
                app,
                (on_leave_play_state, cleanup_system::<CleanupStatePlay>).chain(),
            );
    }
}

#[derive(Component, Serialize, SerializableComponent, Deserialize, Clone)]
pub struct CleanupStatePlay;

fn spawn_stuff(
    mut cmds: Commands,
    mut e_player_moved: EventWriter<PlayerMovedEvent>,
    mut game_state: ResMut<CurrentGameState>,
) {
    trace!("EnterAppState::<Play>");

    cmds.spawn((
        Position::new(56, 56, 0),
        Glyph::new(147, Palette::Yellow, Palette::Blue).layer(RenderLayer::Actors),
        Player,
        // StoreZonePosition,
        CleanupStatePlay,
    ));

    e_player_moved.write(PlayerMovedEvent { x: 56, y: 56, z: 0 });

    game_state.next = GameState::Explore;
}

fn on_leave_play_state(mut game_state: ResMut<CurrentGameState>) {
    trace!("LeaveAppState::<Play>");
    game_state.next = GameState::None;
}
