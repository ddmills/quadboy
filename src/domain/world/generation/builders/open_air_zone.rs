use crate::{
    cfg::ZONE_SIZE,
    domain::{BiomeBuilder, Terrain, ZoneFactory},
};

pub struct OpenAirBiomeBuilder;

impl BiomeBuilder for OpenAirBiomeBuilder {
    fn build(&mut self, zone: &mut ZoneFactory) {
        for x in 0..ZONE_SIZE.0 {
            for y in 0..ZONE_SIZE.1 {
                if !zone.is_locked_tile(x, y) {
                    zone.set_terrain(x, y, Terrain::OpenAir);
                }
            }
        }
    }
}
