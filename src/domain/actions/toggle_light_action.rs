use bevy_ecs::prelude::*;

use crate::{
    domain::{Energy, EnergyActionType, LightSource, Lightable, get_energy_cost},
    engine::StableIdRegistry,
};

pub struct ToggleLightAction {
    pub item_id: u64,  // Stable ID of the item to toggle
    pub actor: Entity, // Entity that is doing the action
}

impl ToggleLightAction {
    pub fn new(item_id: u64, actor: Entity) -> Self {
        Self { item_id, actor }
    }
}

impl Command for ToggleLightAction {
    fn apply(self, world: &mut World) {
        // Get the item entity from stable ID
        let item_entity = {
            let Some(registry) = world.get_resource::<StableIdRegistry>() else {
                return;
            };
            let Some(entity) = registry.get_entity(self.item_id) else {
                return;
            };
            entity
        };

        // Check if the item has both Lightable and LightSource components
        let has_components = world.get::<Lightable>(item_entity).is_some()
            && world.get::<LightSource>(item_entity).is_some();

        if !has_components {
            return;
        }

        // Toggle the light source
        let is_enabled = if let Some(mut light_source) = world.get_mut::<LightSource>(item_entity) {
            light_source.is_enabled = !light_source.is_enabled;
            light_source.is_enabled
        } else {
            return;
        };

        // Update the lightable component's action label
        if let Some(mut lightable) = world.get_mut::<Lightable>(item_entity) {
            lightable.update_label(is_enabled);
        }

        if let Some(mut energy) = world.get_mut::<Energy>(self.actor) {
            let cost = get_energy_cost(EnergyActionType::ToggleLight);
            energy.consume_energy(cost);
        }
    }
}
