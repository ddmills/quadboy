use crate::{
    cfg::ZONE_SIZE,
    common::{Grid, LootTable, Rand},
    domain::{Biome, LootTableId, Prefab, PrefabId, Terrain, ZoneFactory},
    rendering::zone_local_to_world,
};

use super::super::biome_helpers::*;

pub struct DesertBiome {
    loot_table: LootTable<PrefabId>,
    enemy_table: LootTable<PrefabId>,
}

impl DesertBiome {
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

impl Biome for DesertBiome {
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
        LootTableId::DesertGroundLoot
    }

    fn chest_loot_table_id(&self) -> LootTableId {
        LootTableId::DesertChestLoot
    }

    fn enemy_table_id(&self) -> LootTableId {
        LootTableId::DesertEnemies
    }

    fn generate(&self, zone: &mut ZoneFactory) {
        let mut rand = Rand::seed(zone.zone_idx as u32);

        apply_base_terrain(zone, self.base_terrain());

        let constraint_grid = collect_constraint_grid(zone);
        let boulder_grid = generate_desert_boulder_ca(&constraint_grid, &mut rand);

        place_feature_grid(zone, &boulder_grid, PrefabId::Boulder);

        generate_desert_cacti(zone, &mut rand, Some(&boulder_grid));

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

fn generate_desert_cacti(
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

            if rand.bool(0.02) {
                let wpos = zone_local_to_world(zone.zone_idx, x, y);
                zone.push_entity(x, y, Prefab::new(PrefabId::Cactus, wpos));
            }
        }
    }
}
