use crate::{
    cfg::ZONE_SIZE,
    common::Grid,
    domain::{OverworldZone, Terrain, ZoneBuilder, ZoneData},
};

pub struct CavernZoneBuilder;

impl ZoneBuilder for CavernZoneBuilder {
    fn build(&mut self, ozone: OverworldZone) -> ZoneData {
        let zone_idx = ozone.zone_idx;
        let terrain = Grid::init(ZONE_SIZE.0, ZONE_SIZE.1, Terrain::Dirt);
        let entities = Grid::init_fill(ZONE_SIZE.0, ZONE_SIZE.1, |_, _| vec![]);

        ZoneData {
            zone_idx,
            terrain,
            entities,
        }
    }
}
