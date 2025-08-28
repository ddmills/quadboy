use crate::{
    cfg::ZONE_SIZE,
    common::{Grid, Rand, algorithm::{cellular_automata::*, ca_rules::*}},
    domain::{BiomeBuilder, PrefabId, SpawnConfig, Terrain, ZoneFactory},
    rendering::zone_local_to_world,
};

pub struct ForestBiomeBuilder;

impl BiomeBuilder for ForestBiomeBuilder {
    fn build(zone: &mut ZoneFactory) {
        let mut rand = Rand::seed(zone.zone_idx as u32);

        for x in 0..ZONE_SIZE.0 {
            for y in 0..ZONE_SIZE.1 {
                if !zone.is_locked_tile(x, y) {
                    zone.set_terrain(x, y, Terrain::Grass);
                }
            }
        }

        let constraint_grid = collect_constraint_grid(zone);
        let boulder_grid = generate_forest_boulder_ca(&constraint_grid, &mut rand);

        for x in 0..ZONE_SIZE.0 {
            for y in 0..ZONE_SIZE.1 {
                if zone.is_locked_tile(x, y) {
                    continue;
                }

                let wpos = zone_local_to_world(zone.zone_idx, x, y);

                if *boulder_grid.get(x, y).unwrap_or(&false) {
                    zone.push_entity(x, y, SpawnConfig::new(PrefabId::Boulder, wpos));
                } else if rand.bool(0.01) {
                    zone.push_entity(x, y, SpawnConfig::new(PrefabId::PineTree, wpos));
                } else if rand.bool(0.005) {
                    zone.push_entity(x, y, SpawnConfig::new(PrefabId::Bandit, wpos));
                }
            }
        }
    }
}

fn collect_constraint_grid(zone: &mut ZoneFactory) -> Grid<bool> {
    Grid::init_fill(ZONE_SIZE.0, ZONE_SIZE.1, |x, y| {
        zone.is_locked_tile(x, y)
    })
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
