use crate::{
    cfg::ZONE_SIZE,
    common::{Grid, Rand},
    domain::{BiomeBuilder, PrefabId, SpawnConfig, Terrain, ZoneData, ZoneFactory},
    rendering::zone_local_to_world,
};

pub struct DesertBiomeBuilder;

impl BiomeBuilder for DesertBiomeBuilder {
    fn build(&mut self, zone: &mut ZoneFactory) {
        let mut rand = Rand::seed(zone.zone_idx as u32);

        for x in 0..ZONE_SIZE.0 {
            for y in 0..ZONE_SIZE.1 {
                if !zone.is_locked_tile(x, y) {
                    zone.set_terrain(x, y, Terrain::Sand);

                    let wpos = zone_local_to_world(zone.zone_idx, x, y);

                    if rand.bool(0.01) {
                        zone.push_entity(x, y, SpawnConfig::new(PrefabId::Cactus, wpos));
                    }
                    
                    if rand.bool(0.005) {
                        zone.push_entity(x, y, SpawnConfig::new(PrefabId::Bandit, wpos));
                    }
                }
            }
        }
    }
}
