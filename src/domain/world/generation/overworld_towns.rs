use std::collections::HashMap;

use crate::{
    cfg::{MAP_SIZE, SURFACE_LEVEL_Z},
    common::{PoissonDiscSampler, PoissonDiscSettings},
    domain::world::generation::OverworldTown,
    rendering::zone_idx,
};

pub struct OverworldTownGenerator;

impl OverworldTownGenerator {
    pub fn generate_towns(seed: u32) -> HashMap<usize, HashMap<usize, OverworldTown>> {
        let mut layers = HashMap::new();

        for z in SURFACE_LEVEL_Z..MAP_SIZE.2 {
            let mut towns = HashMap::new();
            let mut sampler = PoissonDiscSampler::new(PoissonDiscSettings {
                width: MAP_SIZE.0,
                height: MAP_SIZE.1,
                radius: 6.0 + z as f32,
                seed: seed + 1000 + z as u32,
            });
            let candidates = sampler.all();

            for (x, y) in candidates {
                let idx = zone_idx(x, y, z);
                let town = OverworldTown {
                    name: Self::generate_town_name(idx, seed),
                };
                towns.insert(idx, town);
            }

            layers.insert(z, towns);
        }

        layers
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
