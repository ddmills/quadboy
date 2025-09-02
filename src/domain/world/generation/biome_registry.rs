use std::sync::Arc;

use crate::domain::{Biome, BiomeType, CavernBiome, DesertBiome, ForestBiome, OpenAirBiome};

pub struct BiomeRegistry {
    forest: Arc<ForestBiome>,
    desert: Arc<DesertBiome>,
    cavern: Arc<CavernBiome>,
    open_air: Arc<OpenAirBiome>,
}

impl BiomeRegistry {
    pub fn new() -> Self {
        Self {
            forest: Arc::new(ForestBiome::new()),
            desert: Arc::new(DesertBiome::new()),
            cavern: Arc::new(CavernBiome::new()),
            open_air: Arc::new(OpenAirBiome::new()),
        }
    }

    pub fn get(&self, biome_type: BiomeType) -> Option<Arc<dyn Biome>> {
        match biome_type {
            BiomeType::Forest => Some(self.forest.clone()),
            BiomeType::Desert => Some(self.desert.clone()),
            BiomeType::Cavern => Some(self.cavern.clone()),
            BiomeType::OpenAir => Some(self.open_air.clone()),
        }
    }
}

impl Default for BiomeRegistry {
    fn default() -> Self {
        Self::new()
    }
}
