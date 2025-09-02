use bevy_ecs::prelude::*;
use macroquad::prelude::trace;

use crate::{
    domain::{
        Energy, EnergyActionType, Equipped, InInventory, Inventory, UnequipItemAction, Zone,
        get_energy_cost,
    },
    engine::StableIdRegistry,
    rendering::{Position, world_to_zone_idx, world_to_zone_local},
};

pub struct DropItemAction {
    pub entity: Entity,
    pub item_stable_id: u64,
    pub drop_position: (usize, usize, usize),
}

impl Command for DropItemAction {
    fn apply(self, world: &mut World) {
        let Some(id_registry) = world.get_resource::<StableIdRegistry>() else {
            eprintln!("DropItemAction: StableIdRegistry not found");
            return;
        };

        let Some(item_entity) = id_registry.get_entity(self.item_stable_id) else {
            eprintln!(
                "DropItemAction: Item entity not found for id {}",
                self.item_stable_id
            );
            return;
        };

        let mut q_equipped = world.query::<&Equipped>();

        if q_equipped.get(world, item_entity).is_ok() {
            trace!("eq? {}", self.item_stable_id);
            UnequipItemAction::new(self.item_stable_id).apply(world);
        }

        let Some(mut inventory) = world.get_mut::<Inventory>(self.entity) else {
            eprintln!("DropItemAction: Entity {:?} has no inventory", self.entity);
            return;
        };

        let Some(item_index) = inventory
            .item_ids
            .iter()
            .position(|&id| id == self.item_stable_id)
        else {
            eprintln!(
                "DropItemAction: Item {} not found in inventory",
                self.item_stable_id
            );
            return;
        };

        inventory.item_ids.remove(item_index);

        let position = Position::new_world(self.drop_position);
        world
            .entity_mut(item_entity)
            .insert(position)
            .remove::<InInventory>();

        // Note: Stackable items keep their StackCount component when dropped

        let zone_idx = world_to_zone_idx(
            self.drop_position.0,
            self.drop_position.1,
            self.drop_position.2,
        );
        let (local_x, local_y) = world_to_zone_local(self.drop_position.0, self.drop_position.1);

        let mut zone_found = false;
        let mut zones = world.query::<&mut Zone>();
        for mut zone in zones.iter_mut(world) {
            if zone.idx == zone_idx {
                zone.entities.insert(local_x, local_y, item_entity);
                zone_found = true;
                break;
            }
        }

        if !zone_found {
            eprintln!(
                "DropItemAction: Could not find zone {} to drop item into",
                zone_idx
            );
        }

        if let Some(mut energy) = world.get_mut::<Energy>(self.entity) {
            let cost = get_energy_cost(EnergyActionType::DropItem);
            energy.consume_energy(cost);
        }
    }
}
