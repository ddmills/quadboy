use bevy_ecs::prelude::*;

use crate::{
    domain::{GameSettings, Zone},
    engine::{save_zone, serialize},
};

#[derive(Component)]
pub struct SaveFlag;

pub struct UnloadZoneCommand {
    pub zone_idx: usize,
    pub despawn_entities: bool,
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

        for v in zone.entities.iter() {
            for e in v {
                despawns.push(*e);

                if q_save_flag.contains(*e, world, t, lc) {
                    let e_save = serialize(*e, world);
                    ent_data.push(e_save);
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

        if self.despawn_entities {
            world.despawn(zone_e);

            for e in despawns.iter() {
                world.despawn(*e);
            }
        }

        Ok(())
    }
}
