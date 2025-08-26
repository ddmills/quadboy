use macroquad::prelude::trace;

use crate::{
    cfg::ZONE_SIZE,
    common::Grid,
    domain::{
        BiomeBuilder, CavernBiomeBuilder, DesertBiomeBuilder, ForestBiomeBuilder,
        OpenAirBiomeBuilder, OverworldZone, PrefabId, SpawnConfig, Terrain, ZoneConstraintType,
        ZoneData, ZoneType,
    },
    rendering::zone_local_to_world,
};

pub struct ZoneFactory {
    pub zone_idx: usize,
    pub ozone: OverworldZone,
    pub terrain: Grid<Terrain>,
    pub entities: Grid<Vec<SpawnConfig>>,
    pub locked: Grid<bool>,
}

impl ZoneFactory {
    pub fn new(ozone: OverworldZone) -> Self {
        Self {
            zone_idx: ozone.zone_idx,
            ozone,
            terrain: Grid::init_fill(ZONE_SIZE.0, ZONE_SIZE.1, |_, _| Terrain::Grass),
            entities: Grid::init_fill(ZONE_SIZE.0, ZONE_SIZE.1, |_, _| vec![]),
            locked: Grid::init_fill(ZONE_SIZE.0, ZONE_SIZE.1, |_, _| false),
        }
    }

    pub fn build(&mut self) -> ZoneData {
        self.apply_edge_constraints();
        self.apply_up_vertical_constraints();
        self.apply_vertical_constraints();
        self.apply_biome();

        self.to_zone_data()
    }

    pub fn to_zone_data(&self) -> ZoneData {
        ZoneData {
            zone_idx: self.zone_idx,
            terrain: self.terrain.clone(),
            entities: self.entities.clone(),
        }
    }

    pub fn apply_biome(&mut self) {
        let mut builder = Self::get_biome_builder(self.ozone.zone_type);
        builder.build(self);
    }

    pub fn push_entity(&mut self, x: usize, y: usize, config: SpawnConfig) {
        if let Some(ents) = self.entities.get_mut(x, y) {
            ents.push(config);
        };
    }

    pub fn lock_tile(&mut self, x: usize, y: usize) {
        self.locked.set(x, y, true);
    }

    pub fn set_terrain(&mut self, x: usize, y: usize, terrain: Terrain) {
        self.terrain.set(x, y, terrain);
    }

    pub fn is_locked_tile(&mut self, x: usize, y: usize) -> bool {
        *self.locked.get(x, y).unwrap_or(&false)
    }

    fn get_biome_builder(zone_type: ZoneType) -> Box<dyn BiomeBuilder> {
        match zone_type {
            ZoneType::OpenAir => Box::new(OpenAirBiomeBuilder),
            ZoneType::Forest => Box::new(ForestBiomeBuilder),
            ZoneType::Desert => Box::new(DesertBiomeBuilder),
            ZoneType::Cavern => Box::new(CavernBiomeBuilder),
        }
    }

    pub fn apply_vertical_constraints(&mut self) {
        for constraint in self.ozone.constraints.up.0.iter() {
            if constraint.constraint == ZoneConstraintType::StairDown {
                let (x, y) = constraint.position;
                let world_pos = zone_local_to_world(self.zone_idx, x, y);
                let stair_config = SpawnConfig::new(PrefabId::StairDown, world_pos);

                if let Some(entities_at_pos) = self.entities.get_mut(x, y) {
                    entities_at_pos.push(stair_config);
                }
                self.locked.set(x, y, true);
            }
        }
    }

    pub fn apply_up_vertical_constraints(&mut self) {
        for constraint in self.ozone.constraints.down.0.iter() {
            if constraint.constraint == ZoneConstraintType::StairDown {
                let (x, y) = constraint.position;
                let world_pos = zone_local_to_world(self.zone_idx, x, y);
                let stair_config = SpawnConfig::new(PrefabId::StairUp, world_pos);

                if let Some(entities_at_pos) = self.entities.get_mut(x, y) {
                    entities_at_pos.push(stair_config);
                }
                self.locked.set(x, y, true);
            }
        }
    }

    pub fn apply_edge_constraints(&mut self) {
        // NORTH
        for (x, constraint) in self.ozone.constraints.north.0.iter().enumerate() {
            if let ZoneConstraintType::Road(_) = constraint {
                let y = 0;
                self.terrain.set(x, y, Terrain::Dirt);
                self.locked.set(x, y, true);
            }
        }

        // SOUTH
        for (x, constraint) in self.ozone.constraints.south.0.iter().enumerate() {
            if let ZoneConstraintType::Road(_) = constraint {
                let y = ZONE_SIZE.1 - 1;
                self.terrain.set(x, y, Terrain::Dirt);
                self.locked.set(x, y, true);
            }
        }

        // EAST
        for (y, constraint) in self.ozone.constraints.east.0.iter().enumerate() {
            if let ZoneConstraintType::Road(_) = constraint {
                let x = ZONE_SIZE.0 - 1;
                self.terrain.set(x, y, Terrain::Dirt);
                self.locked.set(x, y, true);
            }
        }

        // WEST
        for (y, constraint) in self.ozone.constraints.west.0.iter().enumerate() {
            if let ZoneConstraintType::Road(_) = constraint {
                let x = 0;
                self.terrain.set(x, y, Terrain::Dirt);
                self.locked.set(x, y, true);
            }
        }
    }

    pub fn connect_roads(&mut self) {
        // - all roads in the zone should connect
        // - add some curve to roads, instead of straight lines.
        // - connect major roads first
        // - smaller roads should connect with bigger roads
        // - lastly, connect stairs to nearest road with a footpath
    }
}
