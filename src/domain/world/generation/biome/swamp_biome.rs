use crate::{
    cfg::ZONE_SIZE,
    common::{Grid, Rand},
    domain::{Biome, LootTableId, PrefabId, Terrain, ZoneFactory},
};
use bevy_ecs::world::World;

use super::super::biome_helpers::*;
use crate::common::algorithm::{ca_rules::*, cellular_automata::*};

pub struct SwampBiome;

impl SwampBiome {
    pub fn new() -> Self {
        Self
    }
}

impl Biome for SwampBiome {
    fn base_terrain(&self) -> Terrain {
        Terrain::Swamp
    }

    fn road_terrain(&self) -> Terrain {
        Terrain::Dirt
    }

    fn ground_loot_table_id(&self) -> LootTableId {
        LootTableId::SwampGroundLoot
    }

    fn chest_loot_table_id(&self) -> LootTableId {
        LootTableId::SwampChestLoot
    }

    fn enemy_table_id(&self) -> LootTableId {
        LootTableId::SwampEnemies
    }

    fn generate(&self, zone: &mut ZoneFactory, world: &World) {
        let mut rand = Rand::seed(zone.zone_idx as u32);

        apply_base_terrain(zone, self.base_terrain());

        let constraints = collect_constraint_grid(zone);

        let water_grid = generate_swamp_water_ca(&constraints, &mut rand);
        place_water_terrain(zone, &water_grid);

        let tree_grid = generate_swamp_tree_ca(&constraints, &mut rand);
        place_feature_grid(zone, &tree_grid, PrefabId::BaldCypress);

        let exclude = tree_grid;
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

fn generate_swamp_water_ca(constraint_grid: &Grid<bool>, rand: &mut Rand) -> Grid<bool> {
    let initial_grid = Grid::init_fill(ZONE_SIZE.0, ZONE_SIZE.1, |x, y| {
        if *constraint_grid.get(x, y).unwrap_or(&true) {
            false
        } else {
            rand.bool(0.35)
        }
    });

    let mut ca = CellularAutomata::from_grid(initial_grid)
        .with_neighborhood(Neighborhood::Moore)
        .with_boundary(BoundaryBehavior::Constant(false))
        .with_constraints(constraint_grid.clone());

    let water_rule = CaveRule::new(5, 3);
    ca.evolve_steps(&water_rule, 3);

    let smoothing_rule = SmoothingRule::new(0.4);
    ca.evolve_steps(&smoothing_rule, 2);

    ca.grid().clone()
}

fn generate_swamp_tree_ca(constraint_grid: &Grid<bool>, rand: &mut Rand) -> Grid<bool> {
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

    let tree_rule = CaveRule::new(4, 2);
    ca.evolve_steps(&tree_rule, 2);

    let smoothing_rule = SmoothingRule::new(0.3);
    ca.evolve_steps(&smoothing_rule, 1);

    let erosion_rule = ErosionRule::new(2);
    ca.evolve_steps(&erosion_rule, 1);

    ca.grid().clone()
}

fn place_water_terrain(zone: &mut ZoneFactory, water_grid: &Grid<bool>) {
    for x in 0..ZONE_SIZE.0 {
        for y in 0..ZONE_SIZE.1 {
            if *water_grid.get(x, y).unwrap_or(&false) {
                zone.set_terrain(x, y, Terrain::River);
            }
        }
    }
}
