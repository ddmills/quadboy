use bevy_ecs::prelude::*;

use crate::{
    domain::{GameSettings, PrefabId, Prefabs, SpawnConfig, Zone, gen_zone},
    engine::{deserialize_all, try_load_zone},
    rendering::zone_local_to_world,
    states::CleanupStatePlay,
};

pub struct LoadZoneCommand(pub usize);

impl Command<Result> for LoadZoneCommand {
    fn apply(self, world: &mut World) -> Result {
        let zone_idx = self.0;

        // Check if zone is already loaded
        let mut q_zones = world.query::<&Zone>();
        if q_zones.iter(world).any(|zone| zone.idx == zone_idx) {
            return Err(format!("Zone {} is already loaded", zone_idx).into());
        }

        // Try to load from save data, or generate new zone
        let zone_data = {
            let Some(settings) = world.get_resource::<GameSettings>() else {
                return Err("GameSettings resource not found".into());
            };

            if settings.enable_saves {
                try_load_zone(zone_idx, &settings.save_name)
            } else {
                None
            }
        };

        let Some(zone_data) = zone_data else {
            gen_zone(world, zone_idx);
            return Ok(());
        };

        // Create the zone entity
        let zone_entity_id = world.spawn((ZoneStatus::Dormant, CleanupStatePlay)).id();

        let mut zone = Zone::new(zone_data.idx, zone_data.terrain);
        zone.explored = zone_data.explored;

        world.entity_mut(zone_entity_id).insert(zone);

        // Query back for the zone and collect terrain data
        let terrain_tiles: Vec<_> = {
            let zone = world.entity(zone_entity_id).get::<Zone>().unwrap();
            zone.terrain.iter_xy().map(|(x, y, t)| (x, y, *t)).collect()
        };

        for (x, y, t) in terrain_tiles {
            let wpos = zone_local_to_world(zone_data.idx, x, y);
            let config = SpawnConfig::new(PrefabId::TerrainTile(t), wpos);
            let terrain_entity = Prefabs::spawn_world(world, config);
            world
                .entity_mut(terrain_entity)
                .insert(ChildOf(zone_entity_id));
        }

        deserialize_all(&zone_data.entities, world);

        Ok(())
    }
}

// Import the ZoneStatus enum from the zone module
use super::zone::ZoneStatus;
