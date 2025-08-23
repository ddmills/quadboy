use bevy_ecs::prelude::*;

use crate::{
    domain::{LoadZoneCommand, Map, PlayerPosition},
    engine::{Clock, deserialize, try_load_game},
    rendering::GameCamera,
    states::{CurrentGameState, GameState},
};

pub struct LoadGameCommand {
    pub save_name: String,
}

#[derive(Event)]
pub struct LoadGameResult {
    pub success: bool,
    pub message: String,
}

impl Command<()> for LoadGameCommand {
    fn apply(self, world: &mut World) {
        let result = self.execute_load(world);

        if let Some(mut events) = world.get_resource_mut::<Events<LoadGameResult>>() {
            events.send(result);
        }
    }
}

impl LoadGameCommand {
    fn execute_load(&self, world: &mut World) -> LoadGameResult {
        let Some(game_data) = try_load_game(&self.save_name) else {
            return LoadGameResult {
                success: false,
                message: format!("No save found for '{}'", self.save_name),
            };
        };

        let position = game_data.player.position;

        world.insert_resource(Map::new(game_data.seed));
        world.insert_resource(PlayerPosition::from_position(&position));

        let mut camera = world.get_resource_mut::<GameCamera>().unwrap();
        camera.focus_on(position.x, position.y);

        let _ = LoadZoneCommand(position.zone_idx()).apply(world);

        deserialize(game_data.player.entity, world);

        if let Some(mut clock) = world.get_resource_mut::<Clock>() {
            clock.set_tick(game_data.tick);
        }

        if let Some(mut game_state) = world.get_resource_mut::<CurrentGameState>() {
            game_state.next = GameState::Explore;
        }

        LoadGameResult {
            success: true,
            message: format!("Loaded game from '{}'", self.save_name),
        }
    }
}
