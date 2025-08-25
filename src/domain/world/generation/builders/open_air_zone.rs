use crate::{
    cfg::ZONE_SIZE,
    common::Grid,
    domain::{OverworldZone, Terrain, ZoneBuilder, ZoneData},
};

pub struct OpenAirZoneBuilder;

impl ZoneBuilder for OpenAirZoneBuilder {
    fn build(&mut self, ozone: OverworldZone) -> ZoneData {
        let zone_idx = ozone.zone_idx;
        let terrain = Grid::init(ZONE_SIZE.0, ZONE_SIZE.1, Terrain::OpenAir);
        let entities = Grid::init_fill(ZONE_SIZE.0, ZONE_SIZE.1, |_, _| vec![]);

        let mut zone_data = ZoneData {
            zone_idx,
            terrain,
            entities,
        };

        zone_data.apply_vertical_constraints(&ozone.constraints.up);
        zone_data.apply_up_vertical_constraints(&ozone.constraints.down);
        zone_data.apply_edge_constraints(&ozone.constraints.north, &ozone.constraints.south, &ozone.constraints.east, &ozone.constraints.west);

        zone_data
    }
}
