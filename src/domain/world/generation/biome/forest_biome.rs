use bevy_ecs::world::World;
use crate::{
    cfg::ZONE_SIZE,
    common::{Grid, Rand},
    domain::{Biome, LootTableId, PrefabId, Terrain, ZoneFactory},
};

use super::super::biome_helpers::*;
use crate::common::algorithm::{ca_rules::*, cellular_automata::*};

pub struct ForestBiome;

impl ForestBiome {
    pub fn new() -> Self {
        Self
    }
}

impl Biome for ForestBiome {
    fn base_terrain(&self) -> Terrain {
        Terrain::Grass
    }

    fn road_terrain(&self) -> Terrain {
        Terrain::Dirt
    }

    fn ground_loot_table_id(&self) -> LootTableId {
        LootTableId::ForestGroundLoot
    }

    fn chest_loot_table_id(&self) -> LootTableId {
        LootTableId::ForestChestLoot
    }

    fn enemy_table_id(&self) -> LootTableId {
        LootTableId::ForestEnemies
    }

    fn generate(&self, zone: &mut ZoneFactory, world: &World) {
        let mut rand = Rand::seed(zone.zone_idx as u32);

        // Apply base terrain
        apply_base_terrain(zone, self.base_terrain());

        // Generate boulders first
        let constraints = collect_constraint_grid(zone);
        let boulder_grid = generate_forest_boulder_ca(&constraints, &mut rand);

        // Create combined constraint grid that includes boulders
        let boulder_constraint_grid = Grid::init_fill(ZONE_SIZE.0, ZONE_SIZE.1, |x, y| {
            *constraints.get(x, y).unwrap_or(&true) || *boulder_grid.get(x, y).unwrap_or(&false)
        });

        // Generate trees using CA, avoiding boulders
        let tree_grid = generate_forest_tree_ca(&boulder_constraint_grid, &mut rand);

        // Place generated features
        place_feature_grid(zone, &boulder_grid, PrefabId::Boulder);
        place_feature_grid(zone, &tree_grid, PrefabId::PineTree);

        // Spawn loot and enemies with standard 1% chances
        let exclude = combine_grids(&boulder_grid, &tree_grid);
        spawn_loot_and_enemies(
            zone,
            self.ground_loot_table_id(),
            self.enemy_table_id(),
            self.chest_loot_table_id(),
            world,
            &mut rand,
            Some(&exclude),
        );
    }
}

fn generate_forest_boulder_ca(constraint_grid: &Grid<bool>, rand: &mut Rand) -> Grid<bool> {
    let initial_grid = Grid::init_fill(ZONE_SIZE.0, ZONE_SIZE.1, |x, y| {
        if *constraint_grid.get(x, y).unwrap_or(&true) {
            false
        } else {
            rand.bool(0.2)
        }
    });

    let mut ca = CellularAutomata::from_grid(initial_grid)
        .with_neighborhood(Neighborhood::Moore)
        .with_boundary(BoundaryBehavior::Constant(false))
        .with_constraints(constraint_grid.clone());

    let initial_rule = CaveRule::new(4, 2);
    ca.evolve_steps(&initial_rule, 3);

    let smoothing_rule = SmoothingRule::new(0.5);
    ca.evolve_steps(&smoothing_rule, 3);

    let erosion_rule = ErosionRule::new(2);
    ca.evolve_steps(&erosion_rule, 1);

    ca.grid().clone()
}

fn generate_forest_tree_ca(constraint_grid: &Grid<bool>, rand: &mut Rand) -> Grid<bool> {
    // Initialize with higher density for trees than boulders
    let initial_grid = Grid::init_fill(ZONE_SIZE.0, ZONE_SIZE.1, |x, y| {
        if *constraint_grid.get(x, y).unwrap_or(&true) {
            false
        } else {
            // Higher initial density for forest clusters
            rand.bool(0.35)
        }
    });

    let mut ca = CellularAutomata::from_grid(initial_grid)
        .with_neighborhood(Neighborhood::Moore)
        .with_boundary(BoundaryBehavior::Constant(false))
        .with_constraints(constraint_grid.clone());

    // Trees form in clusters - born if 4-6 neighbors, survive if 3-6 neighbors
    let forest_rule = CaveRule::new(4, 3);
    ca.evolve_steps(&forest_rule, 2);

    // Light smoothing to create more natural forest clusters
    let smoothing_rule = SmoothingRule::new(0.45);
    ca.evolve_steps(&smoothing_rule, 2);

    // Slight erosion to create clearings and paths
    let erosion_rule = ErosionRule::new(1);
    ca.evolve_steps(&erosion_rule, 1);

    ca.grid().clone()
}
