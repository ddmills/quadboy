use bevy_ecs::prelude::*;

use crate::states::{CurrentGameState, GameState};

pub struct OpenContainerAction {
    pub player_entity: Entity,
    pub container_entity: Entity,
}

impl Command for OpenContainerAction {
    fn apply(self, world: &mut World) {
        world.insert_resource(crate::states::InventoryContext {
            player_entity: self.player_entity,
            container_entity: Some(self.container_entity),
        });

        if let Some(mut game_state) = world.get_resource_mut::<CurrentGameState>() {
            game_state.next = GameState::Container;
        }
    }
}
