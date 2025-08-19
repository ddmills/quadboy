use bevy_ecs::prelude::*;

use crate::{
    common::Palette,
    domain::{Player, PlayerMovedEvent},
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

        // Send the result as an event
        if let Some(mut events) = world.get_resource_mut::<Events<LoadGameResult>>() {
            events.send(result);
        }
    }
}

impl LoadGameCommand {
    fn execute_load(&self, world: &mut World) -> LoadGameResult {
        // Try to load game data
        let Some(game_data) = try_load_game(&self.save_name) else {
            return LoadGameResult {
                success: false,
                message: format!("No save found for '{}'", self.save_name),
            };
        };

        // Get position data before moving
        let player_position = game_data.player.position.clone();
        let pos = player_position.world();

        // Spawn the player at the saved position
        let _player_entity = world
            .spawn((
                player_position,
                Glyph::new(147, Palette::Yellow, Palette::Blue).layer(RenderLayer::Actors),
                Player,
                CleanupStatePlay,
            ))
            .id();

        // Emit PlayerMovedEvent to trigger zone loading
        if let Some(mut events) = world.get_resource_mut::<Events<PlayerMovedEvent>>() {
            events.send(PlayerMovedEvent {
                x: pos.0,
                y: pos.1,
                z: pos.2,
            });
        }

        // Set game state to Explore
        if let Some(mut game_state) = world.get_resource_mut::<CurrentGameState>() {
            game_state.next = GameState::Explore;
        }

        LoadGameResult {
            success: true,
            message: format!("Loaded game from '{}'", self.save_name),
        }
    }
}
