use crate::{
    cfg::ZONE_SIZE,
    common::{
        Grid, LootTable, Rand,
        algorithm::{ca_rules::*, cellular_automata::*},
    },
    domain::{BiomeBuilder, Prefab, PrefabId, Terrain, ZoneFactory},
    rendering::zone_local_to_world,
};

pub struct DesertBiomeBuilder;

impl BiomeBuilder for DesertBiomeBuilder {
    fn build(zone: &mut ZoneFactory) {
        let mut rand = Rand::seed(zone.zone_idx as u32);

        // Set base terrain
        for x in 0..ZONE_SIZE.0 {
            for y in 0..ZONE_SIZE.1 {
                if !zone.is_locked_tile(x, y) {
                    zone.set_terrain(x, y, Terrain::Sand);
                }
            }
        }

        // Generate boulder clusters using CA
        let constraint_grid = collect_constraint_grid(zone);
        let boulder_grid = generate_desert_boulder_ca(&constraint_grid, &mut rand);

        // Create loot table for desert spawns
        let desert_loot = LootTable::builder()
            .add(Some(PrefabId::Cactus), 20.0)    // 0.02 probability -> 20 weight
            .add(Some(PrefabId::Bandit), 5.0)     // 0.005 probability -> 5 weight
            .add(Some(PrefabId::Lantern), 1.0)    // 0.001 probability -> 1 weight
            .add(Some(PrefabId::Pickaxe), 1.0)    // 0.001 probability -> 1 weight
            .add(Some(PrefabId::Hatchet), 1.0)    // 0.001 probability -> 1 weight
            .add(None, 972.0)                     // No spawn (remainder to make 1000 total)
            .build();

        // Place entities
        for x in 0..ZONE_SIZE.0 {
            for y in 0..ZONE_SIZE.1 {
                if zone.is_locked_tile(x, y) {
                    continue;
                }

                let wpos = zone_local_to_world(zone.zone_idx, x, y);

                if *boulder_grid.get(x, y).unwrap_or(&false) {
                    zone.push_entity(x, y, Prefab::new(PrefabId::Boulder, wpos));
                } else if let Some(prefab_id) = desert_loot.pick_cloned(&mut rand) {
                    zone.push_entity(x, y, Prefab::new(prefab_id, wpos));
                }
            }
        }
    }
}

fn collect_constraint_grid(zone: &mut ZoneFactory) -> Grid<bool> {
    Grid::init_fill(ZONE_SIZE.0, ZONE_SIZE.1, |x, y| zone.is_locked_tile(x, y))
}

fn generate_desert_boulder_ca(constraint_grid: &Grid<bool>, rand: &mut Rand) -> Grid<bool> {
    let initial_grid = Grid::init_fill(ZONE_SIZE.0, ZONE_SIZE.1, |x, y| {
        if *constraint_grid.get(x, y).unwrap_or(&true) {
            false
        } else {
            rand.bool(0.25)
        }
    });

    let mut ca = CellularAutomata::from_grid(initial_grid)
        .with_neighborhood(Neighborhood::Moore)
        .with_boundary(BoundaryBehavior::Constant(false))
        .with_constraints(constraint_grid.clone());

    let desert_rule = CaveRule::new(5, 3);
    ca.evolve_steps(&desert_rule, 2);

    let smoothing_rule = SmoothingRule::new(0.5);
    ca.evolve_steps(&smoothing_rule, 2);

    let erosion_rule = ErosionRule::new(3);
    ca.evolve_steps(&erosion_rule, 1);

    ca.grid().clone()
}
