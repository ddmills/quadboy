use bevy_ecs::prelude::*;
use macroquad::prelude::trace;
use serde::{Deserialize, Serialize};

use crate::{
    domain::{
        activate_zones_by_player, cleanup_despawned_stable_ids, load_nearby_zones, manage_zone_cache, on_load_zone, on_set_zone_status, on_unload_zone, register_game_systems, update_player_position_resource
    },
    engine::{App, Plugin, SerializableComponent},
    rendering::{on_zone_status_change, update_camera, ScreenSize},
    states::{cleanup_system, AppState, AppStatePlugin, CurrentGameState, GameState},
    ui::{draw_ui_panels, update_ui_layout, UiLayout},
};

pub struct PlayStatePlugin;

impl Plugin for PlayStatePlugin {
    fn build(&self, app: &mut App) {
        AppStatePlugin::new(AppState::Play)
            .on_enter(
                app,
                (
                    update_ui_layout,
                    draw_ui_panels,
                    on_enter_play_state,
                    register_game_systems,
                )
                    .chain(),
            )
            .on_update(
                app,
                (
                    (
                        cleanup_despawned_stable_ids,
                        activate_zones_by_player,
                        load_nearby_zones,
                        on_load_zone,
                        on_unload_zone,
                        on_set_zone_status,
                        manage_zone_cache,
                        on_zone_status_change,
                        update_camera,
                        update_player_position_resource,
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

fn on_enter_play_state(mut game_state: ResMut<CurrentGameState>) {
    trace!("EnterAppState::<Play>");
    game_state.next = GameState::Explore;
}

fn on_leave_play_state(mut game_state: ResMut<CurrentGameState>) {
    trace!("LeaveAppState::<Play>");
    game_state.next = GameState::None;
}
