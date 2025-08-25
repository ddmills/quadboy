use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};

use bevy_ecs::resource::Resource;

use crate::{
    cfg::{MAP_SIZE, SURFACE_LEVEL_Z},
    common::{Perlin, PoissonDiscSampler, PoissonDiscSettings},
    domain::{OverworldRoadGenerator, OverworldTownGenerator, ZoneConstraints},
    rendering::{zone_idx, zone_xyz},
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ZoneType {
    OpenAir,
    Forest,
    Desert,
    Cavern,
}

impl Display for ZoneType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ZoneType::OpenAir => write!(f, "OpenAir"),
            ZoneType::Forest => write!(f, "Forest"),
            ZoneType::Desert => write!(f, "Desert"),
            ZoneType::Cavern => write!(f, "Cavern"),
        }
    }
}

pub struct OverworldZone {
    pub zone_idx: usize,
    pub zone_type: ZoneType,
    pub constraints: ZoneConstraints,
    pub town: Option<OverworldTown>,
}

#[derive(Clone)]
pub struct OverworldTown {
    pub name: String,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RoadType {
    DirtPath,
    StoneRoad,
    RoyalHighway,
}

#[derive(Clone, Debug)]
pub struct RoadSegment {
    pub road_type: RoadType,
    pub length: f32,
}

#[derive(Default)]
pub struct RoadNetwork {
    pub edges: HashMap<(usize, usize), RoadSegment>, // (from_zone, to_zone) -> segment
    pub nodes: HashSet<usize>,                       // All zones with roads
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
    pub towns: HashMap<usize, OverworldTown>,
    pub road_network: RoadNetwork,
}

impl Overworld {
    pub fn new(seed: u32) -> Self {
        let mut overworld = Self {
            seed,
            perlin: Perlin::new(seed, 0.15, 2, 2.0),
            towns: HashMap::new(),
            road_network: RoadNetwork::default(),
        };

        overworld.towns = OverworldTownGenerator::generate_towns(seed);
        overworld.generate_roads();
        overworld
    }

    pub fn get_overworld_zone(&mut self, zone_idx: usize) -> OverworldZone {
        OverworldZone {
            zone_idx,
            zone_type: self.get_zone_type(zone_idx),
            constraints: self.get_zone_constraints(zone_idx),
            town: self.towns.get(&zone_idx).cloned(),
        }
    }

    pub fn get_zone_type(&mut self, zone_idx: usize) -> ZoneType {
        let (x, y, z) = zone_xyz(zone_idx);

        if z > SURFACE_LEVEL_Z {
            return ZoneType::OpenAir;
        }

        if z < SURFACE_LEVEL_Z {
            return ZoneType::Cavern;
        }

        let noise = self.perlin.get(x as f32, y as f32);

        if noise < 0.4 {
            return ZoneType::Desert;
        }

        ZoneType::Forest
    }

    fn get_zone_constraints(&self, zone_idx: usize) -> ZoneConstraints {
        ZoneConstraints {
            idx: zone_idx,
            south: vec![],
            west: vec![],
            east: vec![],
            north: vec![],
            up: vec![],
            down: vec![],
        }
    }

    fn generate_roads(&mut self) {
        self.road_network = OverworldRoadGenerator::generate_roads(&self.towns, self.seed);
    }
}
