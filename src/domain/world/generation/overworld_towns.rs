use std::collections::HashMap;

use crate::{
    cfg::{MAP_SIZE, SURFACE_LEVEL_Z},
    common::{PoissonDiscSampler, PoissonDiscSettings},
    domain::{biome::BiomeType, world::generation::OverworldTown},
    rendering::{zone_idx, zone_xyz},
};

pub struct OverworldTownGenerator;

impl OverworldTownGenerator {
    pub fn generate_towns(seed: u32) -> HashMap<usize, OverworldTown> {
        let mut towns = HashMap::new();

        let settings = PoissonDiscSettings {
            width: MAP_SIZE.0,
            height: MAP_SIZE.1,
            radius: 6.0,
            seed: seed + 1000,
        };

        let mut sampler = PoissonDiscSampler::new(settings);
        let candidates = sampler.all();

        for (x, y) in candidates {
            let idx = zone_idx(x, y, SURFACE_LEVEL_Z);
            let zone_type = Self::get_zone_type_at(idx, seed);

            if matches!(zone_type, BiomeType::Forest | BiomeType::Desert) {
                let town = OverworldTown {
                    name: Self::generate_town_name(idx, seed),
                };
                towns.insert(idx, town);
            }
        }

        towns
    }

    fn get_zone_type_at(zone_idx: usize, seed: u32) -> BiomeType {
        // This duplicates the logic from Overworld::get_zone_type but with seed parameter
        use crate::common::Perlin;

        let (x, y, z) = zone_xyz(zone_idx);
        let mut perlin = Perlin::new(seed, 0.15, 2, 2.0);

        if z > SURFACE_LEVEL_Z {
            return BiomeType::OpenAir;
        }

        if z < SURFACE_LEVEL_Z {
            return BiomeType::Cavern;
        }

        let noise = perlin.get(x as f32, y as f32);

        if noise < 0.4 {
            return BiomeType::Desert;
        }

        BiomeType::Forest
    }

    fn generate_town_name(zone_idx: usize, seed: u32) -> String {
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

        let index = (zone_idx + seed as usize) % names.len();
        names[index].to_string()
    }
}
