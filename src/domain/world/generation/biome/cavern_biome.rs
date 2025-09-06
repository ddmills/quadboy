use bevy_ecs::world::World;
use crate::{
    cfg::ZONE_SIZE,
    common::{Grid, Rand},
    domain::{Biome, LootTableId, Prefab, PrefabId, Terrain, ZoneFactory},
    rendering::zone_local_to_world,
};

use super::super::biome_helpers::*;

pub struct CavernBiome;

impl CavernBiome {
    pub fn new() -> Self {
        Self
    }
}

impl Biome for CavernBiome {
    fn base_terrain(&self) -> Terrain {
        Terrain::Sand
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

    fn generate(&self, zone: &mut ZoneFactory, world: &World) {
        let mut rand = Rand::seed(zone.zone_idx as u32);

        apply_base_terrain(zone, self.base_terrain());

        let boulder_grid = generate_cavern_boulder_ca(zone, &mut rand);

        place_feature_grid(zone, &boulder_grid, PrefabId::Boulder);

        generate_giant_mushrooms(zone, &mut rand, Some(&boulder_grid));

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

            if let Some(grid) = exclude_grid
                && *grid.get(x, y).unwrap_or(&false)
            {
                continue;
            }

            let wpos = zone_local_to_world(zone.zone_idx, x, y);
            if rand.bool(0.0025 * wpos.2 as f32) {
                zone.push_entity(x, y, Prefab::new(PrefabId::GiantMushroom, wpos));
            }
        }
    }
}
