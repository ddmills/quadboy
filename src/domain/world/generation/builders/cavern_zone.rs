use crate::{
    cfg::ZONE_SIZE,
    common::{Grid, Rand},
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
                    }

                    if rand.bool(0.005) {
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
    // Initial seeding using edge rock constraints + random interior
    let mut grid = Grid::init_fill(ZONE_SIZE.0, ZONE_SIZE.1, |x, y| {
        if *constraint_grid.get(x, y).unwrap_or(&true) {
            false
        } else {
            let has_edge_rock = is_edge_rock_position(zone, x, y);

            if has_edge_rock { true } else { rand.bool(0.60) }
        }
    });

    for _ in 0..3 {
        grid = cellular_automata_step(&grid, constraint_grid);
    }

    grid
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

fn cellular_automata_step(grid: &Grid<bool>, constraint_grid: &Grid<bool>) -> Grid<bool> {
    grid.map(|x, y, &current| {
        if *constraint_grid.get(x, y).unwrap_or(&true) {
            return false;
        }

        let neighbor_count = count_neighbors(grid, x, y);

        if current {
            neighbor_count >= 4 // Back to 4 for moderate boulder survival
        } else {
            neighbor_count >= 5 // Back to 5 for moderate boulder birth
        }
    })
}

fn count_neighbors(grid: &Grid<bool>, x: usize, y: usize) -> usize {
    let mut count = 0;

    for dx in -1..=1 {
        for dy in -1..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }

            let nx = x as i32 + dx;
            let ny = y as i32 + dy;

            // Treat out-of-bounds as walls (boulders) for edge behavior
            if nx < 0 || ny < 0 || nx >= ZONE_SIZE.0 as i32 || ny >= ZONE_SIZE.1 as i32 {
                count += 1;
            } else if let Some(&is_boulder) = grid.get(nx as usize, ny as usize)
                && is_boulder
            {
                count += 1;
            }
        }
    }

    count
}
