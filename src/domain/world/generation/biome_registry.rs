use std::sync::Arc;

use crate::domain::{
    Biome, BiomeType, CavernBiome, DesertBiome, DustyPlainsBiome, ForestBiome, MountainBiome, OpenAirBiome,
};

pub struct BiomeRegistry {
    forest: Arc<ForestBiome>,
    desert: Arc<DesertBiome>,
    dusty_plains: Arc<DustyPlainsBiome>,
    cavern: Arc<CavernBiome>,
    open_air: Arc<OpenAirBiome>,
    mountain: Arc<MountainBiome>,
}

impl BiomeRegistry {
    pub fn new() -> Self {
        Self {
            forest: Arc::new(ForestBiome::new()),
            desert: Arc::new(DesertBiome::new()),
            dusty_plains: Arc::new(DustyPlainsBiome::new()),
            cavern: Arc::new(CavernBiome::new()),
            open_air: Arc::new(OpenAirBiome::new()),
            mountain: Arc::new(MountainBiome::new()),
        }
    }

    pub fn get(&self, biome_type: BiomeType) -> Option<Arc<dyn Biome>> {
        match biome_type {
            BiomeType::Forest => Some(self.forest.clone()),
            BiomeType::Desert => Some(self.desert.clone()),
            BiomeType::DustyPlains => Some(self.dusty_plains.clone()),
            BiomeType::Cavern => Some(self.cavern.clone()),
            BiomeType::OpenAir => Some(self.open_air.clone()),
            BiomeType::Mountain => Some(self.mountain.clone()),
        }
    }
}

impl Default for BiomeRegistry {
    fn default() -> Self {
        Self::new()
    }
}
