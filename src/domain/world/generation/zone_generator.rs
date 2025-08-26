use bevy_ecs::world::World;
use macroquad::prelude::trace;

use crate::{
    common::Grid,
    domain::{Overworld, SpawnConfig, Terrain, ZoneFactory},
};

pub trait BiomeBuilder {
    fn build(&mut self, zone: &mut ZoneFactory);
}

pub struct ZoneData {
    pub zone_idx: usize,
    pub terrain: Grid<Terrain>,
    pub entities: Grid<Vec<SpawnConfig>>,
}

pub struct ZoneGenerator;

impl ZoneGenerator {
    pub fn generate_zone(world: &mut World, zone_idx: usize) -> ZoneData {
        let mut overworld = world.get_resource_mut::<Overworld>().unwrap();
        let ozone = overworld.get_overworld_zone(zone_idx);

        trace!("Generating zone... {}, {}", zone_idx, ozone.zone_type);

        ZoneFactory::new(ozone).build()
    }
}
