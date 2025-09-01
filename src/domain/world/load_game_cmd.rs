use std::collections::HashMap;

use bevy_ecs::prelude::*;
use bevy_ecs::system::RunSystemOnce;

use crate::{
    domain::{LoadZoneCommand, Overworld, PlayerPosition, TerrainNoise, Zones},
    engine::{Clock, StableIdRegistry, deserialize, reconcile_stable_ids, try_load_game},
    rendering::GameCamera,
    states::{CurrentGameState, GameState},
};

pub struct LoadGameCommand {
    pub save_name: String,
}

#[derive(Event)]
pub struct LoadGameResult {
    pub success: bool,
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
            return LoadGameResult { success: false };
        };

        let position = game_data.player.position;
        let zone_idx = position.zone_idx();

        world.insert_resource(Overworld::new(game_data.seed));
        world.insert_resource(TerrainNoise::new(game_data.seed));
        world.insert_resource(PlayerPosition::from_position(&position));
        world.insert_resource(StableIdRegistry::new());
        world.insert_resource(Zones {
            player: zone_idx,
            active: vec![zone_idx],
            cache: HashMap::new(),
        });

        let mut camera = world.get_resource_mut::<GameCamera>().unwrap();
        camera.focus_on(position.x, position.y);

        let _ = LoadZoneCommand(zone_idx).apply(world);

        deserialize(game_data.player.entity, world);

        reconcile_stable_ids(world);

        if let Some(mut clock) = world.get_resource_mut::<Clock>() {
            clock.set_tick(game_data.tick);
        }

        if let Some(mut game_state) = world.get_resource_mut::<CurrentGameState>() {
            game_state.next = GameState::Explore;
        }

        LoadGameResult { success: true }
    }
}
