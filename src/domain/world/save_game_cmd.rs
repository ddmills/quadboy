use bevy_ecs::prelude::*;
use macroquad::prelude::get_time;

use crate::{
    domain::{
        GameSaveData, GameSettings, Map, Player, PlayerSaveData, UnloadZoneCommand, Zone, Zones,
    },
    engine::{Clock, save_game, serialize},
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

        let save_name = settings.save_name.clone();

        let (player_entity, player_position) = {
            let mut q_player = world.query_filtered::<(Entity, &Position), With<Player>>();

            if let Some((entity, position)) = q_player.iter(world).next() {
                (entity, position.clone())
            } else {
                return SaveGameResult {
                    success: false,
                    zone_count: 0,
                    save_name: save_name.clone(),
                    message: "Player not found".to_string(),
                };
            }
        };

        let current_tick = world
            .get_resource::<Clock>()
            .map(|clock| clock.current_tick())
            .unwrap_or(0);

        let seed = world
            .get_resource::<Map>()
            .map(|map| map.seed)
            .unwrap_or(12345);

        let serialized_player = serialize(player_entity, world);

        let player_save = PlayerSaveData {
            position: player_position,
            entity: serialized_player,
        };
        let game_data = GameSaveData::new(player_save, get_time(), current_tick, seed);
        save_game(&game_data, &save_name);

        let mut q_zones = world.query::<&Zone>();
        let zone_indicies = q_zones.iter(world).map(|z| z.idx).collect::<Vec<_>>();
        let mut zone_count = 0;

        for zone_idx in zone_indicies {
            zone_count += 1;

            let save_cmd = UnloadZoneCommand {
                zone_idx,
                despawn: false,
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
