use crate::{
    cfg::ZONE_SIZE,
    common::{
        Grid, Rand,
        algorithm::{ca_rules::CaveRule, cellular_automata::*},
    },
    domain::{BiomeBuilder, PrefabId, SpawnConfig, Terrain, ZoneConstraintType, ZoneFactory},
    rendering::zone_local_to_world,
};

pub struct CavernBiomeBuilder;

impl BiomeBuilder for CavernBiomeBuilder {
    fn build(zone: &mut ZoneFactory) {
        let mut rand = Rand::seed(zone.zone_idx as u32);

        for x in 0..ZONE_SIZE.0 {
            for y in 0..ZONE_SIZE.1 {
                if !zone.is_locked_tile(x, y) {
                    zone.set_terrain(x, y, Terrain::Sand);
                }
            }
        }

        let boulder_grid = generate_cave_ca(zone, &mut rand);

        // Place boulders based on CA result
        for x in 0..ZONE_SIZE.0 {
            for y in 0..ZONE_SIZE.1 {
                if !zone.is_locked_tile(x, y) {
                    let wpos = zone_local_to_world(zone.zone_idx, x, y);

                    if *boulder_grid.get(x, y).unwrap_or(&false) {
                        zone.push_entity(x, y, SpawnConfig::new(PrefabId::Boulder, wpos));
                    } else if rand.bool(0.005) {
                        zone.push_entity(x, y, SpawnConfig::new(PrefabId::Bandit, wpos));
                    }
                }
            }
        }
    }
}

fn generate_cave_ca(zone: &ZoneFactory, rand: &mut Rand) -> Grid<bool> {
    // Create initial random grid
    let initial_grid = Grid::init_fill(ZONE_SIZE.0, ZONE_SIZE.1, |x, y| {
        // Check edge constraints - don't place rocks where passages should be
        if should_keep_clear(zone, x, y) {
            return false;
        }

        // Place rocks at edge rock positions
        if is_edge_rock_position(zone, x, y) {
            return true;
        }

        // Random initial state with 45% density
        rand.bool(0.5)
    });

    // Create constraints grid to preserve edge passages
    let constraints = Grid::init_fill(ZONE_SIZE.0, ZONE_SIZE.1, |x, y| {
        should_keep_clear(zone, x, y) || is_edge_rock_position(zone, x, y)
    });

    // Run cellular automata
    let mut ca = CellularAutomata::from_grid(initial_grid)
        .with_neighborhood(Neighborhood::Moore)
        .with_boundary(BoundaryBehavior::Constant(true))
        .with_constraints(constraints);

    // Simple cave rule: B5678/S5678 (born if 5-8 neighbors, survive if 5-8 neighbors)
    let rule = CaveRule::new(5, 4);
    ca.evolve_steps(&rule, 5);

    ca.grid().clone()
}

fn should_keep_clear(zone: &ZoneFactory, x: usize, y: usize) -> bool {
    // Check if this position should be kept clear for passages, rivers, or foliage
    if y == 0 {
        if let Some(constraint) = zone.ozone.constraints.north.0.get(x) {
            return matches!(
                constraint,
                ZoneConstraintType::None
                    | ZoneConstraintType::River(_)
                    | ZoneConstraintType::Road(_)
                    | ZoneConstraintType::Foliage
            );
        }
    }

    if y == ZONE_SIZE.1 - 1 {
        if let Some(constraint) = zone.ozone.constraints.south.0.get(x) {
            return matches!(
                constraint,
                ZoneConstraintType::None
                    | ZoneConstraintType::River(_)
                    | ZoneConstraintType::Road(_)
                    | ZoneConstraintType::Foliage
            );
        }
    }

    if x == 0 {
        if let Some(constraint) = zone.ozone.constraints.west.0.get(y) {
            return matches!(
                constraint,
                ZoneConstraintType::None
                    | ZoneConstraintType::River(_)
                    | ZoneConstraintType::Road(_)
                    | ZoneConstraintType::Foliage
            );
        }
    }

    if x == ZONE_SIZE.0 - 1 {
        if let Some(constraint) = zone.ozone.constraints.east.0.get(y) {
            return matches!(
                constraint,
                ZoneConstraintType::None
                    | ZoneConstraintType::River(_)
                    | ZoneConstraintType::Road(_)
                    | ZoneConstraintType::Foliage
            );
        }
    }

    false
}

fn is_edge_rock_position(zone: &ZoneFactory, x: usize, y: usize) -> bool {
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
