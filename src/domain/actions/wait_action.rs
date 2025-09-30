use bevy_ecs::prelude::*;

use crate::domain::{Energy, EnergyActionType, actions::GameAction, get_base_energy_cost};

pub struct WaitAction {
    pub entity: Entity,
}

impl GameAction for WaitAction {
    fn try_apply(self, world: &mut World) -> bool {
        if let Some(mut energy) = world.get_mut::<Energy>(self.entity) {
            let cost = get_base_energy_cost(EnergyActionType::Wait);
            energy.consume_energy(cost);
            true
        } else {
            false
        }
    }
}

impl Command for WaitAction {
    fn apply(self, world: &mut World) {
        self.try_apply(world);
    }
}
