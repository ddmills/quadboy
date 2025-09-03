use bevy_ecs::prelude::*;
use macroquad::prelude::trace;
use serde::{Deserialize, Serialize};

use crate::{
    domain::{GameSettings, Inventory, Zone},
    engine::{SerializableComponent, StableIdRegistry, save_zone, serialize},
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

        let lc = world.last_change_tick();
        let t = world.change_tick();

        let mut q_zones = world.query::<(Entity, &Zone)>();
        let q_save_flag = world.query::<&SaveFlag>();
        let mut q_inventory = world.query::<&Inventory>();
        let Some(id_registry) = world.get_resource::<StableIdRegistry>() else {
            return Err("StableIdRegistry not found".into());
        };

        let Some((zone_e, zone)) = q_zones.iter(world).find(|(_, c)| c.idx == zone_idx) else {
            return Err("Zone not found".into());
        };

        let mut ent_data = vec![];
        let mut despawns = vec![];

        for v in zone.entities.iter() {
            for e in v {
                despawns.push(*e);

                if !q_save_flag.contains(*e, world, t, lc) {
                    continue;
                }

                let e_save = serialize(*e, world);
                ent_data.push(e_save);

                // save inventory items
                if let Ok(inventory) = q_inventory.get(world, *e) {
                    for item_id in inventory.item_ids.iter() {
                        trace!("despawn item_id={}", item_id);
                        let Some(item) = id_registry.get_entity(*item_id) else {
                            trace!("Missing item in stable_registry {}", item_id);
                            continue;
                        };

                        if !q_save_flag.contains(item, world, t, lc) {
                            continue;
                        }

                        let i_save = serialize(item, world);
                        ent_data.push(i_save);
                        despawns.push(item);
                    }
                }
            }
        }

        let mut zone_save = zone.to_save();
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
