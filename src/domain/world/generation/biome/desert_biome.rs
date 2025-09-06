use bevy_ecs::world::World;
use crate::{
    cfg::ZONE_SIZE,
    common::{Grid, Rand},
    domain::{Biome, LootTableId, Prefab, PrefabId, Terrain, ZoneFactory},
    rendering::zone_local_to_world,
};

use super::super::biome_helpers::*;

pub struct DesertBiome;

impl DesertBiome {
    pub fn new() -> Self {
        Self
    }
}

impl Biome for DesertBiome {
    fn base_terrain(&self) -> Terrain {
        Terrain::Sand
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

    fn generate(&self, zone: &mut ZoneFactory, world: &World) {
        let mut rand = Rand::seed(zone.zone_idx as u32);

        apply_base_terrain(zone, self.base_terrain());

        let constraint_grid = collect_constraint_grid(zone);
        let boulder_grid = generate_desert_boulder_ca(&constraint_grid, &mut rand);

        place_feature_grid(zone, &boulder_grid, PrefabId::Boulder);

        generate_desert_cacti(zone, &mut rand, Some(&boulder_grid));

        spawn_loot_and_enemies(
            zone,
            self.ground_loot_table_id(),
            self.enemy_table_id(),
            self.chest_loot_table_id(),
            world,
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

            if let Some(grid) = exclude_grid
                && *grid.get(x, y).unwrap_or(&false)
            {
                continue;
            }

            if rand.bool(0.02) {
                let wpos = zone_local_to_world(zone.zone_idx, x, y);
                zone.push_entity(x, y, Prefab::new(PrefabId::Cactus, wpos));
            }
        }
    }
}
