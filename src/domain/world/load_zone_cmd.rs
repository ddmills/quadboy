use bevy_ecs::prelude::*;

use crate::{
    cfg::ZONE_SIZE,
    common::{Grid, HashGrid},
    domain::{GameSettings, PrefabId, Prefabs, SpawnConfig, Zone, spawn_zone},
    engine::{deserialize_all, try_load_zone},
    rendering::zone_local_to_world,
    states::CleanupStatePlay,
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

        let zone_entity_id = world
            .spawn((
                ZoneStatus::Dormant,
                CleanupStatePlay,
                Zone {
                    idx: zone_data.idx,
                    terrain: zone_data.terrain.clone(),
                    entities: HashGrid::init(ZONE_SIZE.0, ZONE_SIZE.1),
                    visible: Grid::init(ZONE_SIZE.0, ZONE_SIZE.1, false),
                    explored: zone_data.explored,
                },
            ))
            .id();

        for (x, y, t) in zone_data.terrain.iter_xy() {
            let wpos = zone_local_to_world(zone_data.idx, x, y);
            let config = SpawnConfig::new(PrefabId::TerrainTile(*t), wpos);
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
