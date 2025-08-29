use crate::{
    cfg::ZONE_SIZE,
    common::{
        Grid, Rand,
        algorithm::{ca_rules::*, cellular_automata::*},
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

        let constraint_grid = collect_constraint_grid(zone);
        let boulder_grid = generate_boulder_cellular_automata(&constraint_grid, zone, &mut rand);

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

fn collect_constraint_grid(zone: &mut ZoneFactory) -> Grid<bool> {
    Grid::init_fill(ZONE_SIZE.0, ZONE_SIZE.1, |x, y| {
        if zone.is_locked_tile(x, y) {
            return true;
        }

        if y == 0
            && let Some(constraint) = zone.ozone.constraints.north.0.get(x)
            && *constraint == ZoneConstraintType::None
        {
            return true;
        }

        if y == ZONE_SIZE.1 - 1
            && let Some(constraint) = zone.ozone.constraints.south.0.get(x)
            && *constraint == ZoneConstraintType::None
        {
            return true;
        }

        if x == 0
            && let Some(constraint) = zone.ozone.constraints.west.0.get(y)
            && *constraint == ZoneConstraintType::None
        {
            return true;
        }

        if x == ZONE_SIZE.0 - 1
            && let Some(constraint) = zone.ozone.constraints.east.0.get(y)
            && *constraint == ZoneConstraintType::None
        {
            return true;
        }

        false
    })
}

fn generate_boulder_cellular_automata(
    constraint_grid: &Grid<bool>,
    zone: &ZoneFactory,
    rand: &mut Rand,
) -> Grid<bool> {
    let initial_grid = Grid::init_fill(ZONE_SIZE.0, ZONE_SIZE.1, |x, y| {
        if *constraint_grid.get(x, y).unwrap_or(&true) {
            false
        } else {
            let has_edge_rock = is_edge_rock_position(zone, x, y);
            if has_edge_rock { true } else { rand.bool(0.60) }
        }
    });

    let mut ca = CellularAutomata::from_grid(initial_grid)
        .with_neighborhood(Neighborhood::Moore)
        .with_boundary(BoundaryBehavior::Constant(true))
        .with_constraints(constraint_grid.clone());

    let rule = CaveRule::new(5, 4);
    ca.evolve_steps(&rule, 3);

    ca.grid().clone()
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
