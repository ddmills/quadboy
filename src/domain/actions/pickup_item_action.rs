use bevy_ecs::prelude::*;

use crate::{
    domain::{Energy, EnergyActionType, InInventory, Inventory, Zone, get_energy_cost},
    engine::StableIdRegistry,
    rendering::Position,
};

pub struct PickupItemAction {
    pub entity: Entity,
    pub item_stable_id: u64,
    pub spend_energy: bool,
}

impl Command for PickupItemAction {
    fn apply(self, world: &mut World) {
        let Some(id_registry) = world.get_resource::<StableIdRegistry>() else {
            eprintln!("PickupItemAction: StableIdRegistry not found");
            return;
        };

        let Some(item_entity) = id_registry.get_entity(self.item_stable_id) else {
            return;
        };
        let Some(entity_stable_id) = id_registry.get_id(self.entity) else {
            return;
        };

        let mut q_inventory = world.query::<&mut Inventory>();
        let mut q_items = world.query::<&Position>();
        let mut q_zones = world.query::<&mut Zone>();

        let Ok(mut inventory) = q_inventory.get_mut(world, self.entity) else {
            return;
        };

        if !inventory.add_item(self.item_stable_id) {
            return;
        }

        if let Ok(position) = q_items.get(world, item_entity) {
            let zone_idx = position.zone_idx();

            for mut zone in q_zones.iter_mut(world) {
                if zone.idx == zone_idx {
                    zone.entities.remove(&item_entity);
                    break;
                }
            }
        }

        world
            .entity_mut(item_entity)
            .remove::<Position>()
            .remove::<ChildOf>()
            .insert(InInventory::new(entity_stable_id));

        if self.spend_energy
            && let Some(mut energy) = world.get_mut::<Energy>(self.entity)
        {
            let cost = get_energy_cost(EnergyActionType::PickUpItem);
            energy.consume_energy(cost);
        }
    }
}
