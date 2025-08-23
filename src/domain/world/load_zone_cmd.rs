use bevy_ecs::prelude::*;

use crate::{
    domain::{GameSettings, Zone, spawn_zone, spawn_zone_load},
    engine::try_load_zone,
};

pub struct LoadZoneCommand(pub usize);

impl Command<Result> for LoadZoneCommand {
    fn apply(self, world: &mut World) -> Result {
        let zone_idx = self.0;

        let mut q_zones = world.query::<&Zone>();
        if q_zones.iter(world).any(|zone| zone.idx == zone_idx) {
            return Err(format!("Zone {} is already loaded", zone_idx).into());
        }

        let zone_save_data = {
            let Some(settings) = world.get_resource::<GameSettings>() else {
                return Err("GameSettings resource not found".into());
            };

            if settings.enable_saves {
                try_load_zone(zone_idx, &settings.save_name)
            } else {
                None
            }
        };

        let Some(zone_data) = zone_save_data else {
            spawn_zone(world, zone_idx);
            return Ok(());
        };

        spawn_zone_load(world, zone_data);

        Ok(())
    }
}
