use bevy_ecs::prelude::*;
use macroquad::prelude::get_time;

use crate::{
    domain::{
        GameSaveData, GameSettings, Overworld, Player, PlayerSaveData, UnloadZoneCommand, Zone,
    },
    engine::{Clock, save_game, serialize},
    rendering::Position,
};

pub struct SaveGameCommand;

#[derive(Event)]
pub struct SaveGameResult {
    pub success: bool,
}

impl Command<()> for SaveGameCommand {
    fn apply(self, world: &mut World) {
        let result = self.execute_save(world);

        if let Some(mut events) = world.get_resource_mut::<Events<SaveGameResult>>() {
            events.send(result);
        }
    }
}

impl SaveGameCommand {
    fn execute_save(&self, world: &mut World) -> SaveGameResult {
        let Some(settings) = world.get_resource::<GameSettings>() else {
            return SaveGameResult { success: false };
        };

        if !settings.enable_saves {
            return SaveGameResult { success: false };
        }

        let save_name = settings.save_name.clone();

        let (player_entity, player_position) = {
            let mut q_player = world.query_filtered::<(Entity, &Position), With<Player>>();

            if let Some((entity, position)) = q_player.iter(world).next() {
                (entity, position.clone())
            } else {
                return SaveGameResult { success: false };
            }
        };

        let current_tick = world
            .get_resource::<Clock>()
            .map(|clock| clock.current_tick())
            .unwrap_or(0);

        let seed = world
            .get_resource::<Overworld>()
            .map(|overworld| overworld.seed)
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

        for zone_idx in zone_indicies {
            let save_cmd = UnloadZoneCommand {
                zone_idx,
                despawn: false,
            };

            if save_cmd.apply(world).is_err() {
                return SaveGameResult { success: false };
            }
        }

        SaveGameResult { success: true }
    }
}
