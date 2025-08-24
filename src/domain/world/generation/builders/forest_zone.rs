use crate::{
    cfg::ZONE_SIZE,
    common::{Grid, Rand},
    domain::{OverworldZone, PrefabId, SpawnConfig, Terrain, ZoneBuilder, ZoneData},
    rendering::zone_local_to_world,
};

pub struct ForestZoneBuilder;

impl ZoneBuilder for ForestZoneBuilder {
    fn build(&mut self, ozone: OverworldZone) -> ZoneData {
        let zone_idx = ozone.zone_idx;
        let terrain = Grid::init(ZONE_SIZE.0, ZONE_SIZE.1, Terrain::Grass);
        let mut rand = Rand::seed(zone_idx as u32);

        let entities = Grid::init_fill(ZONE_SIZE.0, ZONE_SIZE.1, |x, y| {
            let wpos = zone_local_to_world(zone_idx, x, y);

            if rand.bool(0.01) {
                return vec![SpawnConfig::new(PrefabId::PineTree, wpos)];
            }

            if rand.bool(0.0005) {
                return vec![SpawnConfig::new(PrefabId::Bandit, wpos)];
            }

            vec![]
        });

        ZoneData {
            zone_idx,
            terrain,
            entities,
        }
    }
}
