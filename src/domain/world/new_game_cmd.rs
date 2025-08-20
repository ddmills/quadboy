use bevy_ecs::prelude::*;

use crate::{
    common::Palette,
    domain::{GameSaveData, Map, Player, PlayerSaveData},
    engine::{delete_save, save_game},
    rendering::{Glyph, Position, RenderLayer},
    states::{CleanupStatePlay, CurrentGameState, GameState},
};

pub struct NewGameCommand {
    pub save_name: String,
}

#[derive(Event)]
pub struct NewGameResult {
    pub success: bool,
    pub message: String,
}

impl Command<()> for NewGameCommand {
    fn apply(self, world: &mut World) {
        let result = self.execute_new_game(world);

        if let Some(mut events) = world.get_resource_mut::<Events<NewGameResult>>() {
            events.send(result);
        }
    }
}

impl NewGameCommand {
    fn execute_new_game(&self, world: &mut World) -> NewGameResult {
        delete_save(&self.save_name);

        let starting_position = Position::new(56, 56, 0);
        world.spawn((
            starting_position.clone(),
            Glyph::new(147, Palette::Yellow, Palette::Blue).layer(RenderLayer::Actors),
            Player,
            CleanupStatePlay,
        ));

        world.init_resource::<Map>();

        let player_save_data = PlayerSaveData {
            position: starting_position,
        };
        let game_save_data = GameSaveData::new(player_save_data, 0.0);
        save_game(&game_save_data, &self.save_name);

        if let Some(mut game_state) = world.get_resource_mut::<CurrentGameState>() {
            game_state.next = GameState::Explore;
        }

        NewGameResult {
            success: true,
            message: format!("Started new game with save '{}'", self.save_name),
        }
    }
}
