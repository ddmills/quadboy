use bevy_ecs::prelude::*;

use crate::{
    common::Palette,
    domain::{
        ApplyVisibilityEffects, Collider, Energy, GameSaveData, Label, Map, Player, PlayerPosition,
        PlayerSaveData, Vision,
    },
    engine::{Clock, delete_save, save_game, serialize},
    rendering::{Glyph, Layer, Position, RecordZonePosition},
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

        let starting_position = Position::new(45, 56, 0);
        let player_entity = world
            .spawn((
                starting_position.clone(),
                Glyph::new(147, Palette::Yellow, Palette::Blue).layer(Layer::Actors),
                Player,
                Vision::new(20),
                ApplyVisibilityEffects,
                Collider,
                Energy::new(-10),
                Label::new("{Y-y repeat|Cowboy}"),
                RecordZonePosition,
                CleanupStatePlay,
            ))
            .id();

        world.insert_resource(PlayerPosition::from_position(&starting_position));
        world.insert_resource(Map { seed: 12345 });
        world.insert_resource(Clock::default());

        let serialized_player = serialize(player_entity, world);

        let player_save_data = PlayerSaveData {
            position: starting_position,
            entity: serialized_player,
        };
        let game_save_data = GameSaveData::new(player_save_data, 0.0, 0, 12345);
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
