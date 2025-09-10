use crate::domain::{
    ConstraintHandler, OverworldZone, Prefab, RiverBuilder, RoadBuilder, Terrain, ZoneData,
    ZoneGridData,
};
use bevy_ecs::world::World;

pub struct ZoneFactory {
    pub zone_idx: usize,
    pub ozone: OverworldZone,
    pub grid_data: ZoneGridData,
}

impl ZoneFactory {
    pub fn new(ozone: OverworldZone, registry: &crate::domain::BiomeRegistry) -> Self {
        Self {
            grid_data: ZoneGridData::new(ozone.biome_type.get_primary_terrain(registry)),
            zone_idx: ozone.zone_idx,
            ozone,
        }
    }

    pub fn build(&mut self, registry: &crate::domain::BiomeRegistry, world: &World) -> ZoneData {
        let mut road_builder = RoadBuilder::new();
        let mut river_builder = RiverBuilder::new();
        let road_terrain = self.ozone.biome_type.get_road_terrain(registry);
        let zone_idx = self.zone_idx;
        let biome_type = self.ozone.biome_type;

        // Apply all constraints and set up roads and rivers
        {
            let (terrain, entities, locked) = self.grid_data.get_all_grids_mut();
            ConstraintHandler::apply_all_constraints(
                &self.ozone,
                terrain,
                entities,
                locked,
                &mut road_builder,
                &mut river_builder,
                road_terrain,
            );
        }

        // Build river connections first (they have priority)
        {
            let locked_grid = self.grid_data.locked_grid();
            river_builder.build_rivers(locked_grid, zone_idx);
        }

        // Apply rivers to terrain
        {
            let (terrain, locked) = self.grid_data.get_terrain_and_locked_mut();
            river_builder.apply_rivers_to_terrain(terrain, locked);
        }

        // Build road connections
        {
            let locked_grid = self.grid_data.locked_grid();
            road_builder.build_roads(locked_grid, zone_idx);
        }

        // Apply roads to terrain (will create shallows where roads cross rivers)
        {
            let (terrain, locked) = self.grid_data.get_terrain_and_locked_mut();
            road_builder.apply_roads_to_terrain(terrain, locked, road_terrain);
        }

        if let Some(biome) = registry.get(biome_type) {
            biome.generate(self, world);
        }

        self.to_zone_data()
    }

    pub fn to_zone_data(&self) -> ZoneData {
        ZoneData {
            terrain: self.grid_data.terrain.clone(),
            entities: self.grid_data.entities.clone(),
        }
    }

    pub fn push_entity(&mut self, x: usize, y: usize, config: Prefab) {
        self.grid_data.push_entity(x, y, config);
    }


    pub fn set_terrain(&mut self, x: usize, y: usize, terrain: Terrain) {
        self.grid_data.set_terrain(x, y, terrain);
    }

    pub fn is_locked_tile(&mut self, x: usize, y: usize) -> bool {
        self.grid_data.is_locked_tile(x, y)
    }
}
