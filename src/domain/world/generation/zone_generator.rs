use bevy_ecs::world::World;

use crate::{
    common::Grid,
    domain::{BiomeRegistry, Overworld, Prefab, Terrain, ZoneFactory},
};

pub struct ZoneData {
    pub zone_idx: usize,
    pub terrain: Grid<Terrain>,
    pub entities: Grid<Vec<Prefab>>,
}

pub struct ZoneGenerator;

impl ZoneGenerator {
    pub fn generate_zone(world: &mut World, zone_idx: usize) -> ZoneData {
        let mut overworld = world.get_resource_mut::<Overworld>().unwrap();
        let ozone = overworld.get_overworld_zone(zone_idx);

        // Get or create BiomeRegistry (for now, create a new one each time)
        let registry = BiomeRegistry::new();

        ZoneFactory::new(ozone, &registry).build(&registry, world)
    }
}
