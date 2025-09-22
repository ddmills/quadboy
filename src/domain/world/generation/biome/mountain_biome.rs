use crate::{
    cfg::ZONE_SIZE,
    common::algorithm::{ca_rules::*, cellular_automata::*},
    common::{Grid, Rand},
    domain::{Biome, LootTableId, PrefabId, Terrain, ZoneFactory},
};
use bevy_ecs::world::World;

use super::super::biome_helpers::*;

pub struct MountainBiome;

impl MountainBiome {
    pub fn new() -> Self {
        Self
    }
}

impl Biome for MountainBiome {
    fn base_terrain(&self) -> Terrain {
        Terrain::Gravel
    }

    fn road_terrain(&self) -> Terrain {
        Terrain::Dirt
    }

    fn ground_loot_table_id(&self) -> LootTableId {
        LootTableId::MountainGroundLoot
    }

    fn chest_loot_table_id(&self) -> LootTableId {
        LootTableId::MountainChestLoot
    }

    fn enemy_table_id(&self) -> LootTableId {
        LootTableId::MountainEnemies
    }

    fn generate(&self, zone: &mut ZoneFactory, world: &World) {
        let mut rand = Rand::seed(zone.zone_idx as u32);

        // Apply base terrain (grass)
        apply_base_terrain(zone, self.base_terrain());

        // Generate constraint grid first
        let constraints = collect_constraint_grid(zone);

        // Generate boulders using custom mountain generation (large formations)
        let boulder_grid = generate_mountain_boulder_ca(&constraints, &mut rand);

        // Generate sparse pine trees
        let boulder_constraint_grid = Grid::init_fill(ZONE_SIZE.0, ZONE_SIZE.1, |x, y| {
            *constraints.get(x, y).unwrap_or(&true) || *boulder_grid.get(x, y).unwrap_or(&false)
        });

        let pine_grid = generate_mountain_pine_trees(&boulder_constraint_grid, &mut rand);

        // Place generated features
        place_feature_grid(zone, &boulder_grid, PrefabId::Boulder);
        place_feature_grid(zone, &pine_grid, PrefabId::PineTree);

        // Spawn loot and enemies
        let exclude = combine_grids(&boulder_grid, &pine_grid);
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

fn generate_mountain_pine_trees(constraint_grid: &Grid<bool>, rand: &mut Rand) -> Grid<bool> {
    // Mountains have moderate pine tree coverage in small pockets
    let initial_grid = Grid::init_fill(ZONE_SIZE.0, ZONE_SIZE.1, |x, y| {
        if *constraint_grid.get(x, y).unwrap_or(&true) {
            false
        } else {
            // Increased density for more tree pockets
            rand.bool(0.15)
        }
    });

    let mut ca = CellularAutomata::from_grid(initial_grid)
        .with_neighborhood(Neighborhood::Moore)
        .with_boundary(BoundaryBehavior::Constant(false))
        .with_constraints(constraint_grid.clone());

    // Trees form in small, tight clusters with moderate birth requirements
    let mountain_rule = CaveRule::new(3, 2);
    ca.evolve_steps(&mountain_rule, 2);

    // Light smoothing to create small natural groves
    let smoothing_rule = SmoothingRule::new(0.3);
    ca.evolve_steps(&smoothing_rule, 1);

    // Moderate erosion to create separated pockets
    let erosion_rule = ErosionRule::new(2);
    ca.evolve_steps(&erosion_rule, 2);

    ca.grid().clone()
}
