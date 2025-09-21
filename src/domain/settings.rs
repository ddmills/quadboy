use crate::{
    engine::SerializedEntity,
    rendering::{CameraMode, CrtCurvature, Position},
};
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Resource, Clone)]
pub struct GameSettings {
    pub input_delay: f64,
    pub input_initial_delay: f64,
    pub zone_boundary_move_delay: f64,
    pub enable_saves: bool,
    pub save_name: String,
    pub camera_mode: CameraMode,
    pub crt_curvature: CrtCurvature,
    pub crt_scanline: bool,
    pub crt_film_grain: bool,
    pub crt_flicker: bool,
    pub crt_vignette: bool,
    pub crt_chromatic_ab: bool,
    pub smooth_movement: bool,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            input_delay: 0.075,
            input_initial_delay: 0.2,
            zone_boundary_move_delay: 0.0,
            enable_saves: true,
            save_name: "test".to_string(),
            camera_mode: CameraMode::Smooth(0.1),
            crt_curvature: CrtCurvature::Off,
            crt_scanline: false,
            crt_film_grain: false,
            crt_flicker: false,
            crt_vignette: true,
            crt_chromatic_ab: true,
            smooth_movement: true,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PlayerSaveData {
    pub position: Position,
    pub entity: SerializedEntity,
    #[serde(default)]
    pub inventory_items: Vec<SerializedEntity>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GameSaveData {
    pub player: PlayerSaveData,
    pub save_timestamp: f64,
    pub tick: u32,
    pub seed: u32,
}

impl GameSaveData {
    pub fn new(player: PlayerSaveData, save_timestamp: f64, tick: u32, seed: u32) -> Self {
        Self {
            player,
            save_timestamp,
            tick,
            seed,
        }
    }
}
