use std::collections::{HashMap, HashSet};

use bevy_ecs::resource::Resource;

use crate::{
    cfg::SURFACE_LEVEL_Z,
    common::{Direction, Perlin},
    domain::{
        BiomeType, OverworldRiverGenerator, OverworldRoadGenerator, OverworldTownGenerator,
        ZoneContinuity, get_zone_constraints,
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

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum RiverType {
    Creek,
    Stream,
    River,
    MightyRiver,
}

impl RiverType {
    pub fn width(self) -> usize {
        match self {
            RiverType::Creek => 1,
            RiverType::Stream => 2,
            RiverType::River => 3,
            RiverType::MightyRiver => 4,
        }
    }

    pub fn upgrade(self) -> Self {
        match self {
            RiverType::Creek => RiverType::Stream,
            RiverType::Stream => RiverType::River,
            RiverType::River => RiverType::MightyRiver,
            RiverType::MightyRiver => RiverType::MightyRiver,
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

#[derive(Clone, Debug)]
pub struct RiverSegment {
    pub river_type: RiverType,
    pub flow_direction: Direction,
    pub depth: f32,
}

#[derive(Default, Clone)]
pub struct RiverNetwork {
    pub edges: HashMap<(usize, usize), RiverSegment>,
    pub nodes: HashSet<usize>,
    pub confluences: Vec<usize>,
}

impl RiverNetwork {
    pub fn has_river(&self, zone_idx: usize) -> bool {
        self.nodes.contains(&zone_idx)
    }

    pub fn get_river_at_edge(&self, from_zone: usize, to_zone: usize) -> Option<&RiverSegment> {
        self.edges
            .get(&(from_zone, to_zone))
            .or_else(|| self.edges.get(&(to_zone, from_zone)))
    }
}

#[derive(Resource)]
pub struct Overworld {
    perlin: Perlin,
    pub seed: u32,
    pub towns: HashMap<usize, HashMap<usize, OverworldTown>>,
    pub road_networks: HashMap<usize, RoadNetwork>,
    pub river_networks: HashMap<usize, RiverNetwork>,
}

impl Overworld {
    pub fn new(seed: u32) -> Self {
        let mut overworld = Self {
            seed,
            perlin: Perlin::new(seed, 0.15, 2, 2.0),
            towns: HashMap::new(),
            road_networks: HashMap::new(),
            river_networks: HashMap::new(),
        };

        // Generate rivers first (natural features)
        overworld.river_networks = OverworldRiverGenerator::generate_rivers(seed);

        // Then generate towns (near rivers for water access)
        overworld.towns = OverworldTownGenerator::generate_towns(seed);

        // Finally generate roads (connecting towns, bridging rivers)
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

        if noise < 0.33 {
            return BiomeType::Desert;
        } else if noise > 0.75 {
            return BiomeType::Mountain;
        }

        BiomeType::Forest
    }

    pub fn zone_has_river(&self, zone_idx: usize) -> bool {
        let (_, _, z) = zone_xyz(zone_idx);
        if let Some(network) = self.river_networks.get(&z) {
            network.has_river(zone_idx)
        } else {
            false
        }
    }

    pub fn get_river_network(&self, z: usize) -> Option<&RiverNetwork> {
        self.river_networks.get(&z)
    }

    fn generate_roads(&mut self) {
        self.road_networks = OverworldRoadGenerator::generate_roads(&self.towns, self.seed);
    }
}
