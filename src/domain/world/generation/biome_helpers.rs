use crate::{
    cfg::ZONE_SIZE,
    common::{
        Grid, LootTable, Rand,
        algorithm::{ca_rules::*, cellular_automata::*},
    },
    domain::{Prefab, PrefabId, Terrain, ZoneConstraintType, ZoneFactory},
    rendering::zone_local_to_world,
};

const LOOT_SPAWN_CHANCE: f32 = 0.01; // 1% chance for loot
const ENEMY_SPAWN_CHANCE: f32 = 0.01; // 1% chance for enemies

pub fn apply_base_terrain(zone: &mut ZoneFactory, terrain: Terrain) {
    for x in 0..ZONE_SIZE.0 {
        for y in 0..ZONE_SIZE.1 {
            if !zone.is_locked_tile(x, y) {
                zone.set_terrain(x, y, terrain);
            }
        }
    }
}

pub fn collect_constraint_grid(zone: &mut ZoneFactory) -> Grid<bool> {
    Grid::init_fill(ZONE_SIZE.0, ZONE_SIZE.1, |x, y| zone.is_locked_tile(x, y))
}

pub fn place_feature_grid(zone: &mut ZoneFactory, feature_grid: &Grid<bool>, prefab_id: PrefabId) {
    for x in 0..ZONE_SIZE.0 {
        for y in 0..ZONE_SIZE.1 {
            if *feature_grid.get(x, y).unwrap_or(&false) {
                let wpos = zone_local_to_world(zone.zone_idx, x, y);
                zone.push_entity(x, y, Prefab::new(prefab_id.clone(), wpos));
            }
        }
    }
}

pub fn spawn_loot_and_enemies(
    zone: &mut ZoneFactory,
    loot_table: &LootTable<PrefabId>,
    enemy_table: &LootTable<PrefabId>,
    rand: &mut Rand,
    exclude_grid: Option<&Grid<bool>>,
) {
    for x in 0..ZONE_SIZE.0 {
        for y in 0..ZONE_SIZE.1 {
            if zone.is_locked_tile(x, y) {
                continue;
            }

            if let Some(grid) = exclude_grid {
                if *grid.get(x, y).unwrap_or(&false) {
                    continue;
                }
            }

            let wpos = zone_local_to_world(zone.zone_idx, x, y);

            // Check for enemy spawn (1% chance)
            if rand.bool(ENEMY_SPAWN_CHANCE) && !enemy_table.is_empty() {
                let enemy = enemy_table.pick_guaranteed_cloned(rand);
                zone.push_entity(x, y, Prefab::new(enemy, wpos));
            }
            // Check for loot spawn (1% chance, only if no enemy)
            else if rand.bool(LOOT_SPAWN_CHANCE) && !loot_table.is_empty() {
                let loot = loot_table.pick_guaranteed_cloned(rand);
                zone.push_entity(x, y, Prefab::new(loot, wpos));
            }
        }
    }
}

pub fn combine_grids(grid1: &Grid<bool>, grid2: &Grid<bool>) -> Grid<bool> {
    Grid::init_fill(ZONE_SIZE.0, ZONE_SIZE.1, |x, y| {
        *grid1.get(x, y).unwrap_or(&false) || *grid2.get(x, y).unwrap_or(&false)
    })
}

pub fn generate_desert_boulder_ca(constraint_grid: &Grid<bool>, rand: &mut Rand) -> Grid<bool> {
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

pub fn generate_cavern_boulder_ca(zone: &ZoneFactory, rand: &mut Rand) -> Grid<bool> {
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

        rand.bool(0.5)
    });

    let constraints = Grid::init_fill(ZONE_SIZE.0, ZONE_SIZE.1, |x, y| {
        if should_keep_clear_cavern(zone, x, y) || is_edge_rock_position_cavern(zone, x, y) {
            return true;
        }

        if zone.grid_data.is_locked_tile(x, y)
            && let Some(terrain) = zone.grid_data.terrain.get(x, y)
        {
            if matches!(terrain, Terrain::River | Terrain::Shallows | Terrain::Dirt) {
                return true;
            }
        }

        false
    });

    let mut ca = CellularAutomata::from_grid(initial_grid)
        .with_neighborhood(Neighborhood::Moore)
        .with_boundary(BoundaryBehavior::Constant(true))
        .with_constraints(constraints);

    let rule = CaveRule::new(5, 4);
    ca.evolve_steps(&rule, 5);

    ca.grid().clone()
}

pub fn should_keep_clear_cavern(zone: &ZoneFactory, x: usize, y: usize) -> bool {
    if y == 0
        && let Some(constraint) = zone.ozone.constraints.north.0.get(x)
    {
        return matches!(
            constraint,
            ZoneConstraintType::None
                | ZoneConstraintType::River(_)
                | ZoneConstraintType::Road(_)
                | ZoneConstraintType::Foliage
        );
    }

    if y == ZONE_SIZE.1 - 1
        && let Some(constraint) = zone.ozone.constraints.south.0.get(x)
    {
        return matches!(
            constraint,
            ZoneConstraintType::None
                | ZoneConstraintType::River(_)
                | ZoneConstraintType::Road(_)
                | ZoneConstraintType::Foliage
        );
    }

    if x == 0
        && let Some(constraint) = zone.ozone.constraints.west.0.get(y)
    {
        return matches!(
            constraint,
            ZoneConstraintType::None
                | ZoneConstraintType::River(_)
                | ZoneConstraintType::Road(_)
                | ZoneConstraintType::Foliage
        );
    }

    if x == ZONE_SIZE.0 - 1
        && let Some(constraint) = zone.ozone.constraints.east.0.get(y)
    {
        return matches!(
            constraint,
            ZoneConstraintType::None
                | ZoneConstraintType::River(_)
                | ZoneConstraintType::Road(_)
                | ZoneConstraintType::Foliage
        );
    }

    false
}

pub fn is_edge_rock_position_cavern(zone: &ZoneFactory, x: usize, y: usize) -> bool {
    if y == 0
        && let Some(constraint) = zone.ozone.constraints.north.0.get(x)
    {
        return *constraint == ZoneConstraintType::Rock;
    }

    if y == ZONE_SIZE.1 - 1
        && let Some(constraint) = zone.ozone.constraints.south.0.get(x)
    {
        return *constraint == ZoneConstraintType::Rock;
    }

    if x == 0
        && let Some(constraint) = zone.ozone.constraints.west.0.get(y)
    {
        return *constraint == ZoneConstraintType::Rock;
    }

    if x == ZONE_SIZE.0 - 1
        && let Some(constraint) = zone.ozone.constraints.east.0.get(y)
    {
        return *constraint == ZoneConstraintType::Rock;
    }

    false
}
