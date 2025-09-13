use bevy_ecs::prelude::*;

use crate::{
    domain::{Energy, EnergyActionType, Player, PlayerMovedEvent, Stats, get_energy_cost},
    rendering::Position,
};

pub struct MoveAction {
    pub entity: Entity,
    pub new_position: (usize, usize, usize),
}

impl Command for MoveAction {
    fn apply(self, world: &mut World) {
        let Some(mut position) = world.get_mut::<Position>(self.entity) else {
            eprintln!("MoveAction: Entity {:?} has no Position", self.entity);
            return;
        };

        position.x = self.new_position.0 as f32;
        position.y = self.new_position.1 as f32;
        position.z = self.new_position.2 as f32;

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

        // Then consume energy
        if let Some(mut energy) = world.get_mut::<Energy>(self.entity) {
            energy.consume_energy(cost);
        }
    }
}
