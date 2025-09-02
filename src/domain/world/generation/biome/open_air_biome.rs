use crate::{
    common::LootTable,
    domain::{Biome, LootTableId, PrefabId, Terrain, ZoneFactory},
};

use super::super::biome_helpers::*;

pub struct OpenAirBiome {
    loot_table: LootTable<PrefabId>,
    enemy_table: LootTable<PrefabId>,
}

impl OpenAirBiome {
    pub fn new() -> Self {
        let loot_table = LootTable::builder().build();
        let enemy_table = LootTable::builder().build();

        Self {
            loot_table,
            enemy_table,
        }
    }
}

impl Biome for OpenAirBiome {
    fn base_terrain(&self) -> Terrain {
        Terrain::OpenAir
    }

    fn loot_table(&self) -> &LootTable<PrefabId> {
        &self.loot_table
    }

    fn enemy_table(&self) -> &LootTable<PrefabId> {
        &self.enemy_table
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

    fn generate(&self, zone: &mut ZoneFactory) {
        apply_base_terrain(zone, self.base_terrain());
    }
}
