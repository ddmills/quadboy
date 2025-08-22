use bevy_ecs::prelude::*;

use crate::{
    cfg::ZONE_SIZE,
    common::Grid,
    domain::{GameSettings, Terrain, Zone, gen_zone},
    engine::{deserialize_all, try_load_zone},
    rendering::{Glyph, Layer, Position, zone_local_to_world},
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

        Grid::init_fill(ZONE_SIZE.0, ZONE_SIZE.1, |x, y| {
            let wpos = zone_local_to_world(zone_data.idx, x, y);
            let terrain = zone_data.terrain.get(x, y).unwrap_or(&Terrain::Dirt);

            let idx = terrain.tile();
            let (bg, fg) = terrain.colors();

            world
                .spawn((
                    Position::new(wpos.0, wpos.1, wpos.2),
                    Glyph::idx(idx).bg_opt(bg).fg1_opt(fg).layer(Layer::Terrain),
                    ChildOf(zone_entity_id),
                    ZoneStatus::Dormant,
                    CleanupStatePlay,
                ))
                .id()
        });

        deserialize_all(&zone_data.entities, world);

        world
            .entity_mut(zone_entity_id)
            .insert(Zone::new(zone_data.idx, zone_data.terrain));

        Ok(())
    }
}

// Import the ZoneStatus enum from the zone module
use super::zone::ZoneStatus;
