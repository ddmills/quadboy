use std::fmt::Display;

use crate::{
    common::LootTable,
    domain::{BiomeRegistry, PrefabId, Terrain, ZoneFactory},
};

pub trait Biome: Send + Sync {
    fn base_terrain(&self) -> Terrain;
    fn loot_table(&self) -> &LootTable<PrefabId>;
    fn enemy_table(&self) -> &LootTable<PrefabId>;
    fn road_terrain(&self) -> Terrain;
    fn generate(&self, zone: &mut ZoneFactory);
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BiomeType {
    OpenAir,
    Forest,
    Desert,
    Cavern,
}

impl BiomeType {
    pub fn apply_to_zone(&self, zone_factory: &mut ZoneFactory, registry: &BiomeRegistry) {
        if let Some(biome) = registry.get(*self) {
            biome.generate(zone_factory);
        }
    }

    pub fn get_road_terrain(&self, registry: &BiomeRegistry) -> Terrain {
        if let Some(biome) = registry.get(*self) {
            biome.road_terrain()
        } else {
            // Fallback for compatibility
            match self {
                BiomeType::OpenAir => Terrain::OpenAir,
                BiomeType::Forest => Terrain::Dirt,
                BiomeType::Desert => Terrain::Dirt,
                BiomeType::Cavern => Terrain::Dirt,
            }
        }
    }

    pub fn get_primary_terrain(&self, registry: &BiomeRegistry) -> Terrain {
        if let Some(biome) = registry.get(*self) {
            biome.base_terrain()
        } else {
            // Fallback for compatibility
            match self {
                BiomeType::OpenAir => Terrain::OpenAir,
                BiomeType::Forest => Terrain::Grass,
                BiomeType::Desert => Terrain::Sand,
                BiomeType::Cavern => Terrain::Sand,
            }
        }
    }
}

impl Display for BiomeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BiomeType::OpenAir => write!(f, "OpenAir"),
            BiomeType::Forest => write!(f, "Forest"),
            BiomeType::Desert => write!(f, "Desert"),
            BiomeType::Cavern => write!(f, "Cavern"),
        }
    }
}
