use bevy_ecs::world::World;
use crate::{
    domain::{Biome, LootTableId, Terrain, ZoneFactory},
};

use super::super::biome_helpers::*;

pub struct OpenAirBiome;

impl OpenAirBiome {
    pub fn new() -> Self {
        Self
    }
}

impl Biome for OpenAirBiome {
    fn base_terrain(&self) -> Terrain {
        Terrain::OpenAir
    }

    fn road_terrain(&self) -> Terrain {
        Terrain::OpenAir
    }

    fn ground_loot_table_id(&self) -> LootTableId {
        LootTableId::OpenAirGroundLoot
    }

    fn chest_loot_table_id(&self) -> LootTableId {
        LootTableId::CommonChestLoot
    }

    fn enemy_table_id(&self) -> LootTableId {
        LootTableId::OpenAirEnemies
    }

    fn generate(&self, zone: &mut ZoneFactory, _world: &World) {
        apply_base_terrain(zone, self.base_terrain());
    }
}
