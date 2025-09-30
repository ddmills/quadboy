use bevy_ecs::prelude::*;

use crate::{
    domain::{
        Energy, EnergyActionType, GameSettings, Player, PlayerMovedEvent, SmoothMovement, Stats,
        actions::GameAction, get_energy_cost,
    },
    rendering::{Glyph, Position},
};

pub struct MoveAction {
    pub entity: Entity,
    pub new_position: (usize, usize, usize),
}

impl GameAction for MoveAction {
    fn try_apply(self, world: &mut World) -> bool {
        let Some(mut position) = world.get_mut::<Position>(self.entity) else {
            return false;
        };

        // Store old position for smooth movement animation
        let old_position = (position.x, position.y);

        // Update position immediately (for game logic)
        position.x = self.new_position.0 as f32;
        position.y = self.new_position.1 as f32;
        position.z = self.new_position.2 as f32;

        // Add smooth movement animation if entity has a Glyph and setting is enabled
        let settings = world.resource::<GameSettings>();
        if settings.smooth_movement && world.get::<Glyph>(self.entity).is_some() {
            let new_position = (self.new_position.0 as f32, self.new_position.1 as f32);

            // Only add smooth movement if there's actual movement
            if old_position != new_position {
                let smooth_movement = SmoothMovement::new(old_position, new_position);
                world.entity_mut(self.entity).insert(smooth_movement);
            }
        }

        if world.get::<Player>(self.entity).is_some() {
            world.send_event(PlayerMovedEvent {
                x: self.new_position.0,
                y: self.new_position.1,
                z: self.new_position.2,
            });
        }

        // Calculate energy cost first
        let cost = {
            let stats = world.get::<Stats>(self.entity);
            get_energy_cost(EnergyActionType::Move, stats)
        };

        // Then consume energy - return false if no Energy component
        if let Some(mut energy) = world.get_mut::<Energy>(self.entity) {
            energy.consume_energy(cost);
            true
        } else {
            false
        }
    }
}

impl Command for MoveAction {
    fn apply(self, world: &mut World) {
        self.try_apply(world);
    }
}
