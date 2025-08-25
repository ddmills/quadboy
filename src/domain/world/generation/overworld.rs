use std::fmt::Display;

use bevy_ecs::resource::Resource;

use crate::{cfg::SURFACE_LEVEL_Z, common::Perlin, domain::ZoneConstraints, rendering::zone_xyz};

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
}

#[derive(Resource)]
pub struct Overworld {
    perlin: Perlin,
    pub seed: u32,
}

impl Overworld {
    pub fn new(seed: u32) -> Self {
        Self {
            seed,
            perlin: Perlin::new(seed, 0.15, 2, 2.0),
        }
    }

    pub fn get_overworld_zone(&mut self, zone_idx: usize) -> OverworldZone {
        OverworldZone {
            zone_idx,
            zone_type: self.get_zone_type(zone_idx),
            constraints: self.get_zone_constraints(zone_idx),
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
}
