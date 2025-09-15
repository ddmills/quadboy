use bevy_ecs::world::World;
use std::fmt::Display;

use crate::domain::{BiomeRegistry, LootTableId, Terrain, ZoneFactory};

pub trait Biome: Send + Sync {
    fn base_terrain(&self) -> Terrain;
    fn road_terrain(&self) -> Terrain;
    fn ground_loot_table_id(&self) -> LootTableId;
    fn chest_loot_table_id(&self) -> LootTableId;
    fn enemy_table_id(&self) -> LootTableId;
    fn generate(&self, zone: &mut ZoneFactory, world: &World);
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BiomeType {
    OpenAir,
    Forest,
    Desert,
    Cavern,
    Mountain,
}

impl BiomeType {
    pub fn get_ambient_color(&self) -> u32 {
        match self {
            BiomeType::OpenAir => 0xFFFFFF,
            BiomeType::Forest => 0x1A201D,
            BiomeType::Desert => 0x1D1A19,
            BiomeType::Cavern => 0x171518,
            BiomeType::Mountain => 0x151718,
        }
    }

    pub fn get_ambient_intensity(&self) -> f32 {
        match self {
            BiomeType::OpenAir => 1.0,
            BiomeType::Forest => 0.9,
            BiomeType::Desert => 1.0,
            BiomeType::Cavern => 0.3,
            BiomeType::Mountain => 0.85,
        }
    }

    pub fn uses_daylight_cycle(&self) -> bool {
        match self {
            BiomeType::OpenAir | BiomeType::Forest | BiomeType::Desert | BiomeType::Mountain => {
                true
            }
            BiomeType::Cavern => false,
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
                BiomeType::Mountain => Terrain::Dirt,
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
                BiomeType::Mountain => Terrain::Grass,
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
            BiomeType::Mountain => write!(f, "Mountain"),
        }
    }
}
