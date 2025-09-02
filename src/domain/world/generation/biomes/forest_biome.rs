use crate::{
    cfg::ZONE_SIZE,
    common::{
        Grid, LootTable, Rand,
        algorithm::{ca_rules::*, cellular_automata::*},
    },
    domain::{BiomeBuilder, Prefab, PrefabId, Terrain, ZoneFactory},
    rendering::zone_local_to_world,
};

pub struct ForestBiomeBuilder;

impl BiomeBuilder for ForestBiomeBuilder {
    fn build(zone: &mut ZoneFactory) {
        let mut rand = Rand::seed(zone.zone_idx as u32);

        // Set base terrain
        for x in 0..ZONE_SIZE.0 {
            for y in 0..ZONE_SIZE.1 {
                if !zone.is_locked_tile(x, y) {
                    zone.set_terrain(x, y, Terrain::Grass);
                }
            }
        }

        // Generate boulders first
        let constraint_grid = collect_constraint_grid(zone);
        let boulder_grid = generate_forest_boulder_ca(&constraint_grid, &mut rand);

        // Create combined constraint grid that includes boulders
        let boulder_constraint_grid = Grid::init_fill(ZONE_SIZE.0, ZONE_SIZE.1, |x, y| {
            *constraint_grid.get(x, y).unwrap_or(&true) || *boulder_grid.get(x, y).unwrap_or(&false)
        });

        // Generate trees using CA, avoiding boulders
        let tree_grid = generate_forest_tree_ca(&boulder_constraint_grid, &mut rand);

        // Create loot table for random spawns
        let forest_loot = LootTable::builder()
            .add(Some(PrefabId::Bandit), 5.0)     // 0.005 probability -> 5 weight
            .add(Some(PrefabId::Lantern), 3.0)    // 0.003 probability -> 3 weight  
            .add(Some(PrefabId::Pickaxe), 3.0)    // 0.003 probability -> 3 weight
            .add(Some(PrefabId::Hatchet), 1.0)    // 0.001 probability -> 1 weight
            .add(None, 988.0)                     // No spawn (remainder to make ~1000 total)
            .build();

        // Place entities
        for x in 0..ZONE_SIZE.0 {
            for y in 0..ZONE_SIZE.1 {
                if zone.is_locked_tile(x, y) {
                    continue;
                }

                let wpos = zone_local_to_world(zone.zone_idx, x, y);

                // Boulders take precedence
                if *boulder_grid.get(x, y).unwrap_or(&false) {
                    zone.push_entity(x, y, Prefab::new(PrefabId::Boulder, wpos));
                } else if *tree_grid.get(x, y).unwrap_or(&false) {
                    zone.push_entity(x, y, Prefab::new(PrefabId::PineTree, wpos));
                } else if let Some(prefab_id) = forest_loot.pick_cloned(&mut rand) {
                    zone.push_entity(x, y, Prefab::new(prefab_id, wpos));
                }
            }
        }
    }
}

fn collect_constraint_grid(zone: &mut ZoneFactory) -> Grid<bool> {
    Grid::init_fill(ZONE_SIZE.0, ZONE_SIZE.1, |x, y| zone.is_locked_tile(x, y))
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
