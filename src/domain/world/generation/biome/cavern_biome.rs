use crate::{
    cfg::ZONE_SIZE,
    common::{Grid, LootTable, Rand},
    domain::{Biome, LootTableId, Prefab, PrefabId, Terrain, ZoneConstraintType, ZoneFactory},
    rendering::zone_local_to_world,
};

use super::super::biome_helpers::*;

pub struct CavernBiome {
    loot_table: LootTable<PrefabId>,
    enemy_table: LootTable<PrefabId>,
}

impl CavernBiome {
    pub fn new() -> Self {
        let loot_table = LootTable::builder()
            .add(PrefabId::Lantern, 1.0)
            .add(PrefabId::Pickaxe, 1.0)
            .add(PrefabId::Hatchet, 1.0)
            .build();

        let enemy_table = LootTable::builder().add(PrefabId::Bandit, 1.0).build();

        Self {
            loot_table,
            enemy_table,
        }
    }
}

impl Biome for CavernBiome {
    fn base_terrain(&self) -> Terrain {
        Terrain::Sand
    }

    fn loot_table(&self) -> &LootTable<PrefabId> {
        &self.loot_table
    }

    fn enemy_table(&self) -> &LootTable<PrefabId> {
        &self.enemy_table
    }

    fn road_terrain(&self) -> Terrain {
        Terrain::Dirt
    }

    fn ground_loot_table_id(&self) -> LootTableId {
        LootTableId::CavernGroundLoot
    }

    fn chest_loot_table_id(&self) -> LootTableId {
        LootTableId::CavernChestLoot
    }

    fn enemy_table_id(&self) -> LootTableId {
        LootTableId::CavernEnemies
    }

    fn generate(&self, zone: &mut ZoneFactory) {
        let mut rand = Rand::seed(zone.zone_idx as u32);

        apply_base_terrain(zone, self.base_terrain());

        let boulder_grid = generate_cavern_boulder_ca(zone, &mut rand);

        place_feature_grid(zone, &boulder_grid, PrefabId::Boulder);

        generate_giant_mushrooms(zone, &mut rand, Some(&boulder_grid));

        spawn_loot_and_enemies(
            zone,
            &self.loot_table,
            &self.enemy_table,
            &mut rand,
            Some(&boulder_grid),
        );

        // Spawn chests with biome-specific loot
        spawn_chests(
            zone,
            self.chest_loot_table_id(),
            &mut rand,
            Some(&boulder_grid),
        );
    }
}

fn generate_giant_mushrooms(
    zone: &mut ZoneFactory,
    rand: &mut Rand,
    exclude_grid: Option<&Grid<bool>>,
) {
    for x in 0..ZONE_SIZE.0 {
        for y in 0..ZONE_SIZE.1 {
            if zone.is_locked_tile(x, y) {
                continue;
            }

            if let Some(grid) = exclude_grid {
                if *grid.get(x, y).unwrap_or(&false) {
                    continue;
                }
            }

            let wpos = zone_local_to_world(zone.zone_idx, x, y);
            if rand.bool(0.0025 * wpos.2 as f32) {
                zone.push_entity(x, y, Prefab::new(PrefabId::GiantMushroom, wpos));
            }
        }
    }
}
