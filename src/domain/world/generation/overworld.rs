use std::{collections::HashMap, fmt::Display};

use bevy_ecs::resource::Resource;

use crate::{
    cfg::{MAP_SIZE, SURFACE_LEVEL_Z},
    common::{Perlin, PoissonDiscSampler, PoissonDiscSettings},
    domain::ZoneConstraints,
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

#[derive(Resource)]
pub struct Overworld {
    perlin: Perlin,
    pub seed: u32,
    pub towns: HashMap<usize, OverworldTown>,
}

impl Overworld {
    pub fn new(seed: u32) -> Self {
        let mut overworld = Self {
            seed,
            perlin: Perlin::new(seed, 0.15, 2, 2.0),
            towns: HashMap::new(),
        };

        overworld.generate_towns();
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

    fn generate_towns(&mut self) {
        let settings = PoissonDiscSettings {
            width: MAP_SIZE.0,
            height: MAP_SIZE.1,
            radius: 6.0,
            seed: self.seed + 1000,
        };

        let mut sampler = PoissonDiscSampler::new(settings);
        let candidates = sampler.all();

        for (x, y) in candidates {
            let idx = zone_idx(x, y, SURFACE_LEVEL_Z);
            let zone_type = self.get_zone_type(idx);

            if matches!(zone_type, ZoneType::Forest | ZoneType::Desert) {
                let town = OverworldTown {
                    name: self.generate_town_name(idx),
                };
                self.towns.insert(idx, town);
            }
        }
    }

    fn generate_town_name(&self, zone_idx: usize) -> String {
        let names = [
            "Millbrook",
            "Stonehaven",
            "Greenfield",
            "Riverside",
            "Oakenford",
            "Thornhill",
            "Redrock",
            "Goldleaf",
            "Ironhold",
            "Windhaven",
            "Sundale",
            "Moonshire",
            "Starfall",
            "Drakemoor",
            "Wolfsburg",
            "Eaglerest",
            "Lionheart",
            "Bearwood",
            "Foxhollow",
            "Ravencliff",
        ];

        let index = (zone_idx + self.seed as usize) % names.len();
        names[index].to_string()
    }
}
