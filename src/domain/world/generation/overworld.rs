use std::collections::{HashMap, HashSet};

use bevy_ecs::resource::Resource;

use crate::{
    cfg::SURFACE_LEVEL_Z,
    common::Perlin,
    domain::{
        BiomeType, OverworldRoadGenerator, OverworldTownGenerator, ZoneContinuity,
        get_zone_constraints,
    },
    rendering::zone_xyz,
};

pub struct OverworldZone {
    pub zone_idx: usize,
    pub biome_type: BiomeType,
    pub constraints: ZoneContinuity,
    pub town: Option<OverworldTown>,
}

#[derive(Clone)]
pub struct OverworldTown {
    pub name: String,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum RoadType {
    Footpath,
    Road,
    RoyalHighway,
}

impl RoadType {
    pub fn width(self) -> usize {
        match self {
            RoadType::Footpath => 1,
            RoadType::Road => 2,
            RoadType::RoyalHighway => 3,
        }
    }
}

#[derive(Clone, Debug)]
pub struct RoadSegment {
    pub road_type: RoadType,
    pub length: f32,
}

#[derive(Default, Clone)]
pub struct RoadNetwork {
    pub edges: HashMap<(usize, usize), RoadSegment>,
    pub nodes: HashSet<usize>,
}

impl RoadNetwork {
    pub fn has_road(&self, zone_idx: usize) -> bool {
        self.nodes.contains(&zone_idx)
    }
}

#[derive(Resource)]
pub struct Overworld {
    perlin: Perlin,
    pub seed: u32,
    pub towns: HashMap<usize, HashMap<usize, OverworldTown>>,
    pub road_networks: HashMap<usize, RoadNetwork>,
}

impl Overworld {
    pub fn new(seed: u32) -> Self {
        let mut overworld = Self {
            seed,
            perlin: Perlin::new(seed, 0.15, 2, 2.0),
            towns: HashMap::new(),
            road_networks: HashMap::new(),
        };

        overworld.towns = OverworldTownGenerator::generate_towns(seed);
        overworld.generate_roads();
        overworld
    }

    pub fn get_overworld_zone(&mut self, zone_idx: usize) -> OverworldZone {
        let (_, _, z) = zone_xyz(zone_idx);
        OverworldZone {
            zone_idx,
            biome_type: self.get_zone_type(zone_idx),
            constraints: get_zone_constraints(self, zone_idx),
            town: self.towns.get(&z).and_then(|v| v.get(&zone_idx).cloned()),
        }
    }

    pub fn get_road_network(&self, z: usize) -> Option<&RoadNetwork> {
        self.road_networks.get(&z)
    }

    pub fn zone_has_road(&self, zone_idx: usize) -> bool {
        let (_, _, z) = zone_xyz(zone_idx);

        match self.road_networks.get(&z) {
            Some(rn) => rn.has_road(zone_idx),
            None => false,
        }
    }

    pub fn get_zone_type(&self, zone_idx: usize) -> BiomeType {
        let (x, y, z) = zone_xyz(zone_idx);

        if z < SURFACE_LEVEL_Z {
            return BiomeType::OpenAir;
        }

        if z > SURFACE_LEVEL_Z {
            return BiomeType::Cavern;
        }

        let noise = self.perlin.get(x as f32, y as f32);

        if noise < 0.5 {
            return BiomeType::Desert;
        }

        BiomeType::Forest
    }

    fn generate_roads(&mut self) {
        self.road_networks = OverworldRoadGenerator::generate_roads(&self.towns, self.seed);
    }
}
