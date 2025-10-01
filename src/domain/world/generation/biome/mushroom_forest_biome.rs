use crate::{
    cfg::ZONE_SIZE,
    common::{
        Grid, Rand,
        algorithm::{ca_rules::*, cellular_automata::*},
    },
    domain::{Biome, LootTableId, Prefab, PrefabId, Terrain, ZoneFactory},
    rendering::zone_local_to_world,
};
use bevy_ecs::world::World;

use super::super::biome_helpers::*;

pub struct MushroomForestBiome;

impl MushroomForestBiome {
    pub fn new() -> Self {
        Self
    }
}

impl Biome for MushroomForestBiome {
    fn base_terrain(&self) -> Terrain {
        Terrain::Dirt
    }

    fn road_terrain(&self) -> Terrain {
        Terrain::Sand
    }

    fn ground_loot_table_id(&self) -> LootTableId {
        LootTableId::MushroomForestGroundLoot
    }

    fn chest_loot_table_id(&self) -> LootTableId {
        LootTableId::MushroomForestChestLoot
    }

    fn enemy_table_id(&self) -> LootTableId {
        LootTableId::MushroomForestEnemies
    }

    fn generate(&self, zone: &mut ZoneFactory, world: &World) {
        let mut rand = Rand::seed(zone.zone_idx as u32);

        apply_base_terrain(zone, self.base_terrain());

        let boulder_grid = generate_mushroom_forest_boulder_ca(zone, &mut rand);

        place_feature_grid(zone, &boulder_grid, PrefabId::Boulder);

        generate_mushroom_forest_mushrooms(zone, &mut rand, Some(&boulder_grid));

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

fn generate_mushroom_forest_boulder_ca(zone: &ZoneFactory, rand: &mut Rand) -> Grid<bool> {
    let initial_grid = Grid::init_fill(ZONE_SIZE.0, ZONE_SIZE.1, |x, y| {
        if should_keep_clear_cavern(zone, x, y) {
            return false;
        }

        if zone.grid_data.is_locked_tile(x, y)
            && let Some(terrain) = zone.grid_data.terrain.get(x, y)
        {
            if matches!(terrain, Terrain::River | Terrain::Shallows) {
                return false;
            }
            if matches!(terrain, Terrain::Dirt) {
                return false;
            }
        }

        if is_edge_rock_position_cavern(zone, x, y) {
            return true;
        }

        // More open than caverns - 40% density instead of 50%
        rand.bool(0.3)
    });

    let constraints = Grid::init_fill(ZONE_SIZE.0, ZONE_SIZE.1, |x, y| {
        if should_keep_clear_cavern(zone, x, y) || is_edge_rock_position_cavern(zone, x, y) {
            return true;
        }

        if zone.grid_data.is_locked_tile(x, y)
            && let Some(terrain) = zone.grid_data.terrain.get(x, y)
            && matches!(terrain, Terrain::River | Terrain::Shallows | Terrain::Dirt)
        {
            return true;
        }

        false
    });

    let mut ca = CellularAutomata::from_grid(initial_grid)
        .with_neighborhood(Neighborhood::Moore)
        .with_boundary(BoundaryBehavior::Constant(true))
        .with_constraints(constraints);

    // Slightly different CA rules for more open spaces
    let rule = CaveRule::new(4, 3);
    ca.evolve_steps(&rule, 4);

    ca.grid().clone()
}

fn generate_mushroom_forest_mushrooms(
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

            // Much higher chance for giant mushrooms in mushroom forests
            if rand.bool(0.025) {
                zone.push_entity(x, y, Prefab::new(PrefabId::GiantMushroom, wpos));
            }
        }
    }
}
