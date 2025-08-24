use bevy_ecs::world::World;
use macroquad::prelude::trace;

use crate::{
    common::Grid,
    domain::{
        DesertZoneBuilder, Overworld, OverworldZone, SpawnConfig, Terrain, ZoneType, world::zone,
    },
};

pub trait ZoneBuilder {
    fn build(&mut self, ozone: OverworldZone) -> ZoneData;
}

pub struct ZoneData {
    pub zone_idx: usize,
    pub terrain: Grid<Terrain>,
    pub entities: Grid<Vec<SpawnConfig>>,
}

pub struct ZoneGenerator;

impl ZoneGenerator {
    pub fn generate_zone(world: &mut World, zone_idx: usize) -> ZoneData {
        trace!("GENERATE ZONE {}", zone_idx);
        let mut overworld = world.get_resource_mut::<Overworld>().unwrap();
        let ozone = overworld.get_overworld_zone(zone_idx);
        let mut builder = Self::get_builder(ozone.zone_type);

        builder.build(ozone)
    }

    fn get_builder(zone_type: ZoneType) -> impl ZoneBuilder {
        match zone_type {
            ZoneType::OpenAir => DesertZoneBuilder,
            ZoneType::Forest => DesertZoneBuilder,
            ZoneType::Desert => DesertZoneBuilder,
            ZoneType::Cavern => DesertZoneBuilder,
        }
    }
}
