use bevy_ecs::prelude::*;

use crate::{
    common::Palette,
    domain::{Map, Player},
    engine::try_load_game,
    rendering::{Glyph, RenderLayer},
    states::{CleanupStatePlay, CurrentGameState, GameState},
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

        world.spawn((
            game_data.player.position,
            Glyph::new(147, Palette::Yellow, Palette::Blue).layer(RenderLayer::Actors),
            Player,
            CleanupStatePlay,
        ));

        world.init_resource::<Map>();

        if let Some(mut game_state) = world.get_resource_mut::<CurrentGameState>() {
            game_state.next = GameState::Explore;
        }

        LoadGameResult {
            success: true,
            message: format!("Loaded game from '{}'", self.save_name),
        }
    }
}
