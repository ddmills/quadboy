use bevy_ecs::prelude::*;

use crate::domain::{Energy, EnergyActionType, get_base_energy_cost};

pub struct WaitAction {
    pub entity: Entity,
}

impl Command for WaitAction {
    fn apply(self, world: &mut World) {
        if let Some(mut energy) = world.get_mut::<Energy>(self.entity) {
            let cost = get_base_energy_cost(EnergyActionType::Wait);
            energy.consume_energy(cost);
        }
    }
}
