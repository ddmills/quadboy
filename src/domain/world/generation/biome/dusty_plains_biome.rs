use crate::{
    cfg::ZONE_SIZE,
    common::{Grid, Palette, Rand},
    domain::{Biome, LootTableId, Prefab, PrefabId, SpawnValue, Terrain, ZoneFactory},
    rendering::zone_local_to_world,
};
use bevy_ecs::world::World;

use super::super::biome_helpers::*;

pub struct DustyPlainsBiome;

impl DustyPlainsBiome {
    pub fn new() -> Self {
        Self
    }
}

impl Biome for DustyPlainsBiome {
    fn base_terrain(&self) -> Terrain {
        Terrain::DyingGrass
    }

    fn road_terrain(&self) -> Terrain {
        Terrain::Dirt
    }

    fn ground_loot_table_id(&self) -> LootTableId {
        LootTableId::DustyPlainsGroundLoot
    }

    fn chest_loot_table_id(&self) -> LootTableId {
        LootTableId::DustyPlainsChestLoot
    }

    fn enemy_table_id(&self) -> LootTableId {
        LootTableId::DustyPlainsEnemies
    }

    fn generate(&self, zone: &mut ZoneFactory, world: &World) {
        let mut rand = Rand::seed(zone.zone_idx as u32);

        apply_base_terrain(zone, self.base_terrain());

        let constraint_grid = collect_constraint_grid(zone);

        generate_dusty_plains_cacti(zone, &mut rand, Some(&constraint_grid));
        generate_dusty_plains_pine_trees(zone, &mut rand, Some(&constraint_grid));

        spawn_loot_and_enemies(
            zone,
            self.ground_loot_table_id(),
            self.enemy_table_id(),
            self.chest_loot_table_id(),
            world,
            &mut rand,
            Some(&constraint_grid),
        );
    }
}

fn generate_dusty_plains_cacti(
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

            if rand.bool(0.015) {
                let wpos = zone_local_to_world(zone.zone_idx, x, y);
                zone.push_entity(x, y, Prefab::new(PrefabId::Cactus, wpos));
            }
        }
    }
}

fn generate_dusty_plains_pine_trees(
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

            if rand.bool(0.005) {
                let wpos = zone_local_to_world(zone.zone_idx, x, y);
                zone.push_entity(x, y, Prefab::new(PrefabId::Tree, wpos).with_metadata("fg1".to_owned(), SpawnValue::Palette(Palette::Green)));
            }
        }
    }
}
