use bevy_ecs::prelude::*;
use macroquad::prelude::get_time;

use crate::{
    domain::{GameSaveData, GameSettings, Player, PlayerSaveData, UnloadZoneCommand, Zones},
    engine::save_game,
    rendering::Position,
};

pub struct SaveGameCommand;

#[derive(Event)]
pub struct SaveGameResult {
    pub success: bool,
    pub zone_count: usize,
    pub save_name: String,
    pub message: String,
}

impl Command<()> for SaveGameCommand {
    fn apply(self, world: &mut World) {
        let result = self.execute_save(world);

        // Send the result as an event
        if let Some(mut events) = world.get_resource_mut::<Events<SaveGameResult>>() {
            events.send(result);
        }
    }
}

impl SaveGameCommand {
    fn execute_save(&self, world: &mut World) -> SaveGameResult {
        let Some(zones) = world.get_resource::<Zones>() else {
            return SaveGameResult {
                success: false,
                zone_count: 0,
                save_name: String::new(),
                message: "Zones resource not found".to_string(),
            };
        };

        let Some(settings) = world.get_resource::<GameSettings>() else {
            return SaveGameResult {
                success: false,
                zone_count: 0,
                save_name: String::new(),
                message: "GameSettings resource not found".to_string(),
            };
        };

        if !settings.enable_saves {
            return SaveGameResult {
                success: false,
                zone_count: 0,
                save_name: settings.save_name.clone(),
                message: "Saves are disabled in settings".to_string(),
            };
        }

        // Clone what we need before the mutable borrows
        let active_zones = zones.active.clone();
        let zone_count = active_zones.len();
        let save_name = settings.save_name.clone();

        // Get player position
        let player_position = {
            let mut q_player = world.query_filtered::<&Position, With<Player>>();

            if let Some(position) = q_player.iter(world).next() {
                position.clone()
            } else {
                return SaveGameResult {
                    success: false,
                    zone_count: 0,
                    save_name: save_name.clone(),
                    message: "Player not found".to_string(),
                };
            }
        };

        // Save game data (player position, timestamp, etc.)
        let player_save = PlayerSaveData {
            position: player_position,
        };
        let game_data = GameSaveData::new(player_save, get_time());
        save_game(&game_data, &save_name);

        // Save all active zones without despawning them
        for zone_idx in active_zones {
            let save_cmd = UnloadZoneCommand {
                zone_idx,
                despawn_entities: false,
            };
            if let Err(e) = save_cmd.apply(world) {
                return SaveGameResult {
                    success: false,
                    zone_count: 0,
                    save_name: save_name.clone(),
                    message: format!("Failed to save zone {}: {}", zone_idx, e),
                };
            }
        }

        SaveGameResult {
            success: true,
            zone_count,
            save_name: save_name.clone(),
            message: format!("Saved {} zones to '{}'", zone_count, save_name),
        }
    }
}
