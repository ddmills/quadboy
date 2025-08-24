use bevy_ecs::resource::Resource;

use crate::{common::Perlin, rendering::zone_xyz};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ZoneType {
    OpenAir,
    Forest,
    Desert,
    Cavern,
}

pub struct OverworldZone {
    pub zone_idx: usize,
    pub zone_type: ZoneType,
}

#[derive(Resource)]
pub struct Overworld {
    perlin: Perlin,
}

impl Overworld {
    pub fn new(seed: u32) -> Self {
        Self {
            perlin: Perlin::new(seed, 0.15, 2, 2.0),
        }
    }

    pub fn get_overworld_zone(&mut self, zone_idx: usize) -> OverworldZone {
        let (x, y, z) = zone_xyz(zone_idx);

        let zone_type = if z > 3 {
            ZoneType::OpenAir
        } else if z < 3 {
            ZoneType::Cavern
        } else {
            let noise = self.perlin.get(x as f32, y as f32);

            if noise < 0.4 {
                ZoneType::Desert
            } else {
                ZoneType::Forest
            }
        };

        OverworldZone {
            zone_idx,
            zone_type,
        }
    }
}
