use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    domain::{GameSettings, InInventory, Zone},
    engine::{SerializableComponent, StableIdRegistry, save_zone, serialize},
    rendering::{Position, world_to_zone_idx},
};

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct SaveFlag;

pub struct UnloadZoneCommand {
    pub zone_idx: usize,
    pub despawn: bool,
}

impl Command<Result> for UnloadZoneCommand {
    fn apply(self, world: &mut World) -> Result {
        let zone_idx = self.zone_idx;

        let mut q_zones = world.query::<(Entity, &Zone)>();
        let q_save_flag = world.query::<&SaveFlag>();

        let lc = world.last_change_tick();
        let t = world.change_tick();

        let Some((zone_e, zone)) = q_zones.iter(world).find(|(_, c)| c.idx == zone_idx) else {
            return Err("Zone not found".into());
        };

        let mut ent_data = vec![];
        let mut despawns = vec![];

        // Save entities with positions (existing behavior)
        for v in zone.entities.iter() {
            for e in v {
                despawns.push(*e);

                if q_save_flag.contains(*e, world, t, lc) {
                    let e_save = serialize(*e, world);
                    ent_data.push(e_save);
                }
            }
        }

        // Create zone save data before doing inventory items query
        let mut zone_save = zone.to_save();

        // Also save inventory items whose owners are in this zone
        // First, collect inventory items and their owner IDs
        let mut inventory_items_to_check = Vec::new();
        {
            let mut q_inventory_items = world.query::<(Entity, &InInventory, &SaveFlag)>();
            for (item_entity, in_inventory, _) in q_inventory_items.iter(world) {
                inventory_items_to_check.push((item_entity, in_inventory.owner_id));
            }
        }

        // Now check each item's owner position
        let Some(id_registry) = world.get_resource::<StableIdRegistry>() else {
            return Err("StableIdRegistry not found".into());
        };

        for (item_entity, owner_id) in inventory_items_to_check {
            // Find the owner entity using stable ID
            if let Some(owner_entity) = id_registry.get_entity(owner_id) {
                // Check if owner has a position and is in this zone
                if let Some(owner_pos) = world.get::<Position>(owner_entity) {
                    let owner_zone_idx = world_to_zone_idx(
                        owner_pos.x as usize,
                        owner_pos.y as usize,
                        owner_pos.z as usize,
                    );

                    if owner_zone_idx == zone_idx {
                        // Save this inventory item with this zone
                        let e_save = serialize(item_entity, world);
                        ent_data.push(e_save);

                        // Also add to despawn list if we're despawning the zone
                        if self.despawn {
                            despawns.push(item_entity);
                        }
                    }
                }
            }
        }

        zone_save.entities = ent_data;

        let Some(settings) = world.get_resource::<GameSettings>() else {
            return Err("GameSettings resource not found".into());
        };

        if settings.enable_saves {
            save_zone(&zone_save, &settings.save_name);
        }

        if self.despawn {
            world.despawn(zone_e);

            for e in despawns.iter() {
                world.despawn(*e);
            }
        }

        Ok(())
    }
}
