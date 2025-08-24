use std::collections::HashMap;

use bevy_ecs::prelude::*;

use crate::{
    cfg::SURFACE_LEVEL_Z,
    common::Palette,
    domain::{
        ApplyVisibilityEffects, Collider, Energy, GameSaveData, Label, LoadZoneCommand, Overworld,
        Player, PlayerPosition, PlayerSaveData, Vision, Zones,
    },
    engine::{Clock, delete_save, save_game, serialize},
    rendering::{GameCamera, Glyph, Layer, Position, RecordZonePosition},
    states::{CleanupStatePlay, CurrentGameState, GameState},
};

pub struct NewGameCommand {
    pub save_name: String,
    pub seed: u32,
}

#[derive(Event)]
pub struct NewGameResult {
    pub success: bool,
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

        let starting_position = Position::new(136, 74, SURFACE_LEVEL_Z);
        let start_zone = starting_position.zone_idx();

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

        let mut camera = world.get_resource_mut::<GameCamera>().unwrap();
        camera.focus_on(starting_position.x, starting_position.y);

        world.insert_resource(PlayerPosition::from_position(&starting_position));
        world.insert_resource(Overworld::new(self.seed));
        world.insert_resource(Clock::new());
        world.insert_resource(Zones {
            player: start_zone,
            active: vec![start_zone],
            cache: HashMap::new(),
        });

        let _ = LoadZoneCommand(start_zone).apply(world);

        let serialized_player = serialize(player_entity, world);

        let player_save_data = PlayerSaveData {
            position: starting_position,
            entity: serialized_player,
        };

        let game_save_data = GameSaveData::new(player_save_data, 0.0, 0, self.seed);
        save_game(&game_save_data, &self.save_name);

        if let Some(mut game_state) = world.get_resource_mut::<CurrentGameState>() {
            game_state.next = GameState::Explore;
        }

        NewGameResult { success: true }
    }
}
