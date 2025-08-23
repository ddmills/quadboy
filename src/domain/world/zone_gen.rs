use bevy_ecs::{hierarchy::ChildOf, world::World};

use crate::{
    cfg::ZONE_SIZE,
    common::{AStarSettings, Distance, Grid, Perlin, Rand, astar, bresenham_line},
    domain::{Map, PrefabId, Prefabs, SpawnConfig, Terrain, Zone, ZoneConstraintType, ZoneStatus},
    rendering::{Glyph, Layer, Position, zone_local_to_world},
    states::CleanupStatePlay,
};

fn calculate_edge_proximity_cost(pos: (usize, usize)) -> f32 {
    let (x, y) = pos;
    let distance_to_edge = [x, ZONE_SIZE.0 - 1 - x, y, ZONE_SIZE.1 - 1 - y]
        .into_iter()
        .min()
        .unwrap();

    match distance_to_edge {
        0 => 2.5,
        1 => 2.0,
        2 => 1.5,
        3 => 1.2,
        _ => 1.0,
    }
}

fn collect_constraint_positions(
    constraints: &crate::domain::ZoneConstraints,
) -> (
    Vec<(usize, usize)>,
    Vec<(usize, usize)>,
    Vec<(usize, usize)>,
    Vec<(usize, usize)>,
    Vec<(usize, usize)>,
) {
    let mut river_positions = Vec::new();
    let mut path_positions = Vec::new();
    let mut stair_down_positions = Vec::new();
    let mut stair_up_positions = Vec::new();
    let mut rock_wall_positions = Vec::new();

    for (x, constraint_type) in constraints.south.iter().enumerate() {
        match constraint_type {
            ZoneConstraintType::River => river_positions.push((x, 0)),
            ZoneConstraintType::Footpath => path_positions.push((x, 0)),
            ZoneConstraintType::StairDown => {}
            ZoneConstraintType::RockWall => rock_wall_positions.push((x, 0)),
            ZoneConstraintType::None => {}
        }
    }

    for (x, constraint_type) in constraints.north.iter().enumerate() {
        match constraint_type {
            ZoneConstraintType::River => river_positions.push((x, ZONE_SIZE.1 - 1)),
            ZoneConstraintType::Footpath => path_positions.push((x, ZONE_SIZE.1 - 1)),
            ZoneConstraintType::StairDown => {}
            ZoneConstraintType::RockWall => rock_wall_positions.push((x, ZONE_SIZE.1 - 1)),
            ZoneConstraintType::None => {}
        }
    }

    for (y, constraint_type) in constraints.west.iter().enumerate() {
        match constraint_type {
            ZoneConstraintType::River => river_positions.push((0, y)),
            ZoneConstraintType::Footpath => path_positions.push((0, y)),
            ZoneConstraintType::StairDown => {}
            ZoneConstraintType::RockWall => rock_wall_positions.push((0, y)),
            ZoneConstraintType::None => {}
        }
    }

    for (y, constraint_type) in constraints.east.iter().enumerate() {
        match constraint_type {
            ZoneConstraintType::River => river_positions.push((ZONE_SIZE.0 - 1, y)),
            ZoneConstraintType::Footpath => path_positions.push((ZONE_SIZE.0 - 1, y)),
            ZoneConstraintType::StairDown => {}
            ZoneConstraintType::RockWall => rock_wall_positions.push((ZONE_SIZE.0 - 1, y)),
            ZoneConstraintType::None => {}
        }
    }

    for (x, constraint_type) in constraints.up.iter().enumerate() {
        if constraint_type == &ZoneConstraintType::StairDown {
            let stair_pos = (x, ZONE_SIZE.1 / 2);
            stair_up_positions.push(stair_pos);
        }
    }

    for (x, constraint_type) in constraints.down.iter().enumerate() {
        if constraint_type == &ZoneConstraintType::StairDown {
            let stair_pos = (x, ZONE_SIZE.1 / 2);
            stair_down_positions.push(stair_pos);
        }
    }

    (
        river_positions,
        path_positions,
        stair_down_positions,
        stair_up_positions,
        rock_wall_positions,
    )
}

fn generate_rivers(
    positions: &[(usize, usize)],
    terrain: &mut Grid<Terrain>,
    _rand: &mut Rand,
    zone_idx: usize,
) {
    if positions.len() < 2 {
        return;
    }

    let mut perlin = Perlin::new(42, 0.05, 3, 2.0);

    let mut connected = vec![false; positions.len()];
    connected[0] = true;
    let mut connections_made = 1;

    while connections_made < positions.len() {
        let mut best_path = None;
        let mut best_cost = f32::INFINITY;
        let mut best_to = 0;

        for (from_idx, &from_pos) in positions.iter().enumerate() {
            if !connected[from_idx] {
                continue;
            }

            for (to_idx, &to_pos) in positions.iter().enumerate() {
                if connected[to_idx] {
                    continue;
                }

                let noise_cache = Grid::init_fill(ZONE_SIZE.0, ZONE_SIZE.1, |x, y| {
                    let (world_x, world_y, _world_z) = zone_local_to_world(zone_idx, x, y);
                    perlin.get(world_x as f32, world_y as f32)
                });

                let settings = AStarSettings {
                    start: from_pos,
                    is_goal: |pos| pos == to_pos,
                    cost: move |from, to| {
                        let base_cost = 1.0;
                        let noise_val = noise_cache.get(to.0, to.1).unwrap_or(&0.5);
                        let noise_factor = 0.8 + (noise_val * 0.4);
                        let dx = (from.0 as i32 - to.0 as i32).abs();
                        let dy = (from.1 as i32 - to.1 as i32).abs();
                        let diagonal_bonus = if dx > 0 && dy > 0 { 0.85 } else { 1.0 };
                        let edge_cost = calculate_edge_proximity_cost(to);
                        base_cost * noise_factor * diagonal_bonus * edge_cost
                    },
                    heuristic: |pos| {
                        Distance::manhattan(
                            [pos.0 as i32, pos.1 as i32, 0],
                            [to_pos.0 as i32, to_pos.1 as i32, 0],
                        ) * 0.7
                    },
                    neighbors: |pos| {
                        let mut neighbors = Vec::new();
                        let (x, y) = pos;

                        for dx in -1..=1 {
                            for dy in -1..=1 {
                                if dx == 0 && dy == 0 {
                                    continue;
                                }

                                let nx = x as i32 + dx;
                                let ny = y as i32 + dy;

                                if nx >= 0
                                    && nx < ZONE_SIZE.0 as i32
                                    && ny >= 0
                                    && ny < ZONE_SIZE.1 as i32
                                {
                                    neighbors.push((nx as usize, ny as usize));
                                }
                            }
                        }

                        neighbors
                    },
                    max_depth: 4000,
                    max_cost: Some(ZONE_SIZE.0 as f32 * 2.0),
                };

                let result = astar(settings);
                if result.is_success && result.cost < best_cost {
                    best_path = Some(result.path);
                    best_cost = result.cost;
                    best_to = to_idx;
                } else if !result.is_success {
                    use macroquad::prelude::warn;
                    warn!(
                        "A* failed for river from {:?} to {:?} in zone {}",
                        from_pos, to_pos, zone_idx
                    );
                    warn!(
                        "  Positions count: {}, Distance: {}",
                        positions.len(),
                        Distance::manhattan(
                            [from_pos.0 as i32, from_pos.1 as i32, 0],
                            [to_pos.0 as i32, to_pos.1 as i32, 0]
                        )
                    );

                    let fallback_path = bresenham_line(from_pos, to_pos);
                    let filtered_path: Vec<(usize, usize)> = fallback_path
                        .into_iter()
                        .filter(|(x, y)| *x < ZONE_SIZE.0 && *y < ZONE_SIZE.1)
                        .collect();

                    let fallback_cost = filtered_path.len() as f32;
                    if fallback_cost < best_cost {
                        best_path = Some(filtered_path);
                        best_cost = fallback_cost;
                        best_to = to_idx;
                    }
                }
            }
        }

        if let Some(path) = best_path {
            for &(x, y) in &path {
                terrain.insert(x, y, Terrain::River);
            }
            connected[best_to] = true;
            connections_made += 1;
        } else {
            break;
        }
    }

    ensure_edge_connections(positions, terrain);
}

fn ensure_edge_connections(positions: &[(usize, usize)], terrain: &mut Grid<Terrain>) {
    for &(x, y) in positions {
        terrain.insert(x, y, Terrain::River);
    }
}

fn generate_paths(positions: &[(usize, usize)], terrain: &mut Grid<Terrain>, zone_idx: usize) {
    if positions.len() < 2 {
        return;
    }

    let mut connected = vec![false; positions.len()];
    connected[0] = true;
    let mut connections_made = 1;

    while connections_made < positions.len() {
        let mut best_path = None;
        let mut best_cost = f32::INFINITY;
        let mut best_to = 0;

        for (from_idx, &from_pos) in positions.iter().enumerate() {
            if !connected[from_idx] {
                continue;
            }

            for (to_idx, &to_pos) in positions.iter().enumerate() {
                if connected[to_idx] {
                    continue;
                }

                let settings = AStarSettings {
                    start: from_pos,
                    is_goal: |pos| pos == to_pos,
                    cost: |_from, to| {
                        let mut base_cost = 1.0;
                        if let Some(existing_terrain) = terrain.get(to.0, to.1)
                            && *existing_terrain == Terrain::River
                        {
                            base_cost = 3.0;
                        }
                        let edge_cost = calculate_edge_proximity_cost(to);
                        base_cost * edge_cost
                    },
                    heuristic: |pos| {
                        Distance::manhattan(
                            [pos.0 as i32, pos.1 as i32, 0],
                            [to_pos.0 as i32, to_pos.1 as i32, 0],
                        )
                    },
                    neighbors: |pos| {
                        let mut neighbors = Vec::new();
                        let (x, y) = pos;

                        if x > 0 {
                            neighbors.push((x - 1, y));
                        }
                        if x < ZONE_SIZE.0 - 1 {
                            neighbors.push((x + 1, y));
                        }
                        if y > 0 {
                            neighbors.push((x, y - 1));
                        }
                        if y < ZONE_SIZE.1 - 1 {
                            neighbors.push((x, y + 1));
                        }

                        neighbors
                    },
                    max_depth: 2000,
                    max_cost: Some(ZONE_SIZE.0 as f32 * 1.5),
                };

                let result = astar(settings);
                if result.is_success && result.cost < best_cost {
                    best_path = Some(result.path);
                    best_cost = result.cost;
                    best_to = to_idx;
                } else if !result.is_success {
                    use macroquad::prelude::warn;
                    warn!(
                        "A* failed for footpath from {:?} to {:?} in zone {}",
                        from_pos, to_pos, zone_idx
                    );
                    warn!(
                        "  Positions count: {}, Distance: {}",
                        positions.len(),
                        Distance::manhattan(
                            [from_pos.0 as i32, from_pos.1 as i32, 0],
                            [to_pos.0 as i32, to_pos.1 as i32, 0]
                        )
                    );
                }
            }
        }

        if let Some(path) = best_path {
            for &(x, y) in &path {
                if terrain.get(x, y) != Some(&Terrain::River) {
                    terrain.insert(x, y, Terrain::Dirt);
                }
            }
            connected[best_to] = true;
            connections_made += 1;
        } else {
            break;
        }
    }
}

fn generate_stairs(positions: &[(usize, usize)], terrain: &mut Grid<Terrain>) {
    for &(x, y) in positions {
        terrain.insert(x, y, Terrain::Dirt);
    }
}

fn connect_stairs_to_footpaths(stair_positions: &[(usize, usize)], terrain: &mut Grid<Terrain>) {
    for &stair_pos in stair_positions {
        let mut nearest_footpath = None;
        let mut shortest_distance = f32::INFINITY;

        for x in 0..ZONE_SIZE.0 {
            for y in 0..ZONE_SIZE.1 {
                if let Some(existing_terrain) = terrain.get(x, y)
                    && *existing_terrain == Terrain::Dirt
                    && x != stair_pos.0
                    && y != stair_pos.1
                {
                    let distance = Distance::manhattan(
                        [stair_pos.0 as i32, stair_pos.1 as i32, 0],
                        [x as i32, y as i32, 0],
                    );
                    if distance < shortest_distance {
                        shortest_distance = distance;
                        nearest_footpath = Some((x, y));
                    }
                }
            }
        }

        if let Some(footpath_pos) = nearest_footpath {
            let settings = AStarSettings {
                start: stair_pos,
                is_goal: |pos| pos == footpath_pos,
                cost: |_from, to| {
                    let mut base_cost = 1.0;
                    if let Some(existing_terrain) = terrain.get(to.0, to.1)
                        && *existing_terrain == Terrain::River
                    {
                        base_cost = 3.0;
                    }
                    let edge_cost = calculate_edge_proximity_cost(to);
                    base_cost * edge_cost
                },
                heuristic: |pos| {
                    Distance::manhattan(
                        [pos.0 as i32, pos.1 as i32, 0],
                        [footpath_pos.0 as i32, footpath_pos.1 as i32, 0],
                    )
                },
                neighbors: |pos| {
                    let mut neighbors = Vec::new();
                    let (x, y) = pos;

                    if x > 0 {
                        neighbors.push((x - 1, y));
                    }
                    if x < ZONE_SIZE.0 - 1 {
                        neighbors.push((x + 1, y));
                    }
                    if y > 0 {
                        neighbors.push((x, y - 1));
                    }
                    if y < ZONE_SIZE.1 - 1 {
                        neighbors.push((x, y + 1));
                    }

                    neighbors
                },
                max_depth: 1000,
                max_cost: Some(ZONE_SIZE.0 as f32 * 1.5),
            };

            let result = astar(settings);
            if result.is_success {
                for &(x, y) in &result.path {
                    if terrain.get(x, y) != Some(&Terrain::River) {
                        terrain.insert(x, y, Terrain::Dirt);
                    }
                }
            } else {
                let fallback_path = bresenham_line(stair_pos, footpath_pos);
                for &(x, y) in &fallback_path {
                    if x < ZONE_SIZE.0
                        && y < ZONE_SIZE.1
                        && terrain.get(x, y) != Some(&Terrain::River)
                    {
                        terrain.insert(x, y, Terrain::Dirt);
                    }
                }
            }
        }
    }
}

pub fn gen_zone(world: &mut World, zone_idx: usize) {
    let mut rand = Rand::seed(zone_idx as u64);
    let map = world.resource::<Map>();
    let constraints = map.get_zone_constraints(zone_idx);
    let mut terrain = Grid::init_fill(ZONE_SIZE.0, ZONE_SIZE.1, |_x, _y| Terrain::Grass);
    let (
        river_positions,
        path_positions,
        stair_down_positions,
        stair_up_positions,
        rock_wall_positions,
    ) = collect_constraint_positions(&constraints);

    generate_rivers(&river_positions, &mut terrain, &mut rand, zone_idx);
    generate_paths(&path_positions, &mut terrain, zone_idx);
    generate_stairs(&stair_down_positions, &mut terrain);
    generate_stairs(&stair_up_positions, &mut terrain);

    connect_stairs_to_footpaths(&stair_down_positions, &mut terrain);
    connect_stairs_to_footpaths(&stair_up_positions, &mut terrain);
    let rock_positions =
        generate_cellular_rock_clusters(&terrain, zone_idx, &rock_wall_positions, &constraints);

    let zone_entity_id = world.spawn((ZoneStatus::Dormant, CleanupStatePlay)).id();

    let mut occupied_positions = Grid::init_fill(ZONE_SIZE.0, ZONE_SIZE.1, |_, _| false);
    for &(x, y) in &rock_positions {
        occupied_positions.insert(x, y, true);
    }

    for x in 0..ZONE_SIZE.0 {
        for y in 0..ZONE_SIZE.1 {
            let wpos = zone_local_to_world(zone_idx, x, y);
            let terrain_type = terrain.get(x, y).unwrap_or(&Terrain::Dirt);

            let idx = terrain_type.tile();
            let (bg, fg) = terrain_type.colors();

            if rand.bool(0.05)
                && *terrain_type == Terrain::Grass
                && occupied_positions.get(x, y) == Some(&false)
            {
                let config = SpawnConfig::new((wpos.0, wpos.1, wpos.2), zone_entity_id);
                Prefabs::spawn_world(world, PrefabId::PineTree, config);
                occupied_positions.insert(x, y, true);
            } else if rand.bool(0.002) && occupied_positions.get(x, y) == Some(&false) {
                let config = SpawnConfig::new((wpos.0, wpos.1, wpos.2), zone_entity_id);
                Prefabs::spawn_world(world, PrefabId::Bandit, config);
                occupied_positions.insert(x, y, true);
            }

            world.spawn((
                Position::new(wpos.0, wpos.1, wpos.2),
                Glyph::idx(idx).bg_opt(bg).fg1_opt(fg).layer(Layer::Terrain),
                ChildOf(zone_entity_id),
                ZoneStatus::Dormant,
                CleanupStatePlay,
            ));
        }
    }

    for &(x, y) in &stair_down_positions {
        let wpos = zone_local_to_world(zone_idx, x, y);
        let config = SpawnConfig::new((wpos.0, wpos.1, wpos.2), zone_entity_id);
        let _ = Prefabs::spawn_world(world, PrefabId::StairDown, config);
    }

    for &(x, y) in &stair_up_positions {
        let wpos = zone_local_to_world(zone_idx, x, y);
        let config = SpawnConfig::new((wpos.0, wpos.1, wpos.2), zone_entity_id);
        let _ = Prefabs::spawn_world(world, PrefabId::StairUp, config);
    }

    for &(x, y) in &rock_positions {
        let wpos = zone_local_to_world(zone_idx, x, y);
        let config = SpawnConfig::new((wpos.0, wpos.1, wpos.2), zone_entity_id);
        let _ = Prefabs::spawn_world(world, PrefabId::Boulder, config);
    }

    world
        .entity_mut(zone_entity_id)
        .insert(Zone::new(zone_idx, terrain));
}

fn generate_cellular_rock_clusters(
    terrain: &Grid<Terrain>,
    zone_idx: usize,
    rock_wall_positions: &[(usize, usize)],
    constraints: &crate::domain::ZoneConstraints,
) -> Vec<(usize, usize)> {
    let mut rand = Rand::seed(zone_idx as u64 + 3000);

    let mut rock_grid = Grid::init_fill(ZONE_SIZE.0, ZONE_SIZE.1, |x, y| {
        if rock_wall_positions.contains(&(x, y)) {
            return true;
        }
        if is_none_constraint_edge(x, y, constraints) {
            return false;
        }

        if is_protected_terrain(terrain, x, y) {
            return false;
        }

        let wall_proximity_factor = get_wall_proximity_factor(x, y, rock_wall_positions);
        let edge_factor = get_edge_proximity_factor(x, y);
        let base_probability = 0.35;
        let adjusted_probability =
            base_probability + (edge_factor * 0.05) + (wall_proximity_factor * 0.2);

        rand.random() < adjusted_probability
    });

    for iteration in 0..5 {
        rock_grid = apply_cellular_automata_rules(&rock_grid, terrain, constraints, iteration);
    }

    rock_grid = remove_small_clusters(&rock_grid, constraints, 3);
    let mut rock_positions = Vec::new();
    for x in 0..ZONE_SIZE.0 {
        for y in 0..ZONE_SIZE.1 {
            if rock_grid.get(x, y) == Some(&true) {
                rock_positions.push((x, y));
            }
        }
    }

    rock_positions
}

fn is_protected_terrain(terrain: &Grid<Terrain>, x: usize, y: usize) -> bool {
    matches!(terrain.get(x, y), Some(Terrain::River | Terrain::Dirt))
}

fn is_rock_wall_constraint_edge(
    x: usize,
    y: usize,
    constraints: &crate::domain::ZoneConstraints,
) -> bool {
    if y == 0
        && x < constraints.south.len()
        && constraints.south[x] == crate::domain::ZoneConstraintType::RockWall
    {
        return true;
    }

    if y == ZONE_SIZE.1 - 1
        && x < constraints.north.len()
        && constraints.north[x] == crate::domain::ZoneConstraintType::RockWall
    {
        return true;
    }

    if x == 0
        && y < constraints.west.len()
        && constraints.west[y] == crate::domain::ZoneConstraintType::RockWall
    {
        return true;
    }

    if x == ZONE_SIZE.0 - 1
        && y < constraints.east.len()
        && constraints.east[y] == crate::domain::ZoneConstraintType::RockWall
    {
        return true;
    }

    false
}

fn is_none_constraint_edge(
    x: usize,
    y: usize,
    constraints: &crate::domain::ZoneConstraints,
) -> bool {
    if y == 0
        && x < constraints.south.len()
        && constraints.south[x] == crate::domain::ZoneConstraintType::None
    {
        return true;
    }

    if y == ZONE_SIZE.1 - 1
        && x < constraints.north.len()
        && constraints.north[x] == crate::domain::ZoneConstraintType::None
    {
        return true;
    }

    if x == 0
        && y < constraints.west.len()
        && constraints.west[y] == crate::domain::ZoneConstraintType::None
    {
        return true;
    }

    if x == ZONE_SIZE.0 - 1
        && y < constraints.east.len()
        && constraints.east[y] == crate::domain::ZoneConstraintType::None
    {
        return true;
    }

    false
}

fn get_edge_proximity_factor(x: usize, y: usize) -> f32 {
    let distances = [x, y, ZONE_SIZE.0 - x - 1, ZONE_SIZE.1 - y - 1];
    let dist_to_edge = *distances.iter().min().unwrap();
    let normalized = dist_to_edge as f32 / (ZONE_SIZE.0.min(ZONE_SIZE.1) as f32 / 2.0);
    (1.0 - normalized.min(1.0)) * 0.5
}

fn get_wall_proximity_factor(x: usize, y: usize, rock_wall_positions: &[(usize, usize)]) -> f32 {
    if rock_wall_positions.is_empty() {
        return 0.0;
    }

    let min_distance = rock_wall_positions
        .iter()
        .map(|(wx, wy)| {
            let dx = (x as f32 - *wx as f32).abs();
            let dy = (y as f32 - *wy as f32).abs();
            (dx * dx + dy * dy).sqrt()
        })
        .fold(f32::INFINITY, f32::min);

    let max_influence_distance = 4.0;
    if min_distance > max_influence_distance {
        0.0
    } else {
        (1.0 - (min_distance / max_influence_distance)).max(0.0)
    }
}

fn apply_cellular_automata_rules(
    rock_grid: &Grid<bool>,
    terrain: &Grid<Terrain>,
    constraints: &crate::domain::ZoneConstraints,
    iteration: usize,
) -> Grid<bool> {
    let mut new_grid = rock_grid.clone();

    let (survival_threshold, birth_threshold) = match iteration {
        0..=1 => (4, 5),
        2..=3 => (3, 4),
        _ => (2, 3),
    };

    for x in 0..ZONE_SIZE.0 {
        for y in 0..ZONE_SIZE.1 {
            if is_rock_wall_constraint_edge(x, y, constraints) {
                new_grid.insert(x, y, true);
                continue;
            }

            if is_none_constraint_edge(x, y, constraints) {
                new_grid.insert(x, y, false);
                continue;
            }

            if is_protected_terrain(terrain, x, y) {
                new_grid.insert(x, y, false);
                continue;
            }

            if x == 0 || x == ZONE_SIZE.0 - 1 || y == 0 || y == ZONE_SIZE.1 - 1 {
                continue;
            }

            let neighbors = count_rock_neighbors(rock_grid, x, y);
            let current = rock_grid.get(x, y) == Some(&true);

            let new_state = if current {
                neighbors >= survival_threshold
            } else {
                neighbors >= birth_threshold
            };

            new_grid.insert(x, y, new_state);
        }
    }

    new_grid
}

fn count_rock_neighbors(rock_grid: &Grid<bool>, x: usize, y: usize) -> usize {
    let mut count = 0;

    for dx in -1i32..=1i32 {
        for dy in -1i32..=1i32 {
            if dx == 0 && dy == 0 {
                continue;
            }

            let nx = (x as i32 + dx) as usize;
            let ny = (y as i32 + dy) as usize;

            if nx < ZONE_SIZE.0 && ny < ZONE_SIZE.1 {
                if rock_grid.get(nx, ny) == Some(&true) {
                    count += 1;
                }
            } else {
                count += 1;
            }
        }
    }

    count
}

fn remove_small_clusters(
    rock_grid: &Grid<bool>,
    constraints: &crate::domain::ZoneConstraints,
    min_cluster_size: usize,
) -> Grid<bool> {
    let mut new_grid = Grid::init_fill(ZONE_SIZE.0, ZONE_SIZE.1, |_, _| false);
    let mut visited = Grid::init_fill(ZONE_SIZE.0, ZONE_SIZE.1, |_, _| false);

    for x in 0..ZONE_SIZE.0 {
        for y in 0..ZONE_SIZE.1 {
            if rock_grid.get(x, y) == Some(&true) && visited.get(x, y) == Some(&false) {
                let cluster = flood_fill_cluster(rock_grid, &mut visited, x, y);

                let contains_rock_wall = cluster
                    .iter()
                    .any(|&(cx, cy)| is_rock_wall_constraint_edge(cx, cy, constraints));

                if cluster.len() >= min_cluster_size || contains_rock_wall {
                    for &(cx, cy) in &cluster {
                        new_grid.insert(cx, cy, true);
                    }
                }
            }
        }
    }

    new_grid
}

fn flood_fill_cluster(
    rock_grid: &Grid<bool>,
    visited: &mut Grid<bool>,
    start_x: usize,
    start_y: usize,
) -> Vec<(usize, usize)> {
    let mut cluster = Vec::new();
    let mut stack = vec![(start_x, start_y)];

    while let Some((x, y)) = stack.pop() {
        if visited.get(x, y) == Some(&true) || rock_grid.get(x, y) != Some(&true) {
            continue;
        }

        visited.insert(x, y, true);
        cluster.push((x, y));

        for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
            let nx = (x as i32 + dx) as usize;
            let ny = (y as i32 + dy) as usize;

            if nx < ZONE_SIZE.0 && ny < ZONE_SIZE.1 {
                stack.push((nx, ny));
            }
        }
    }

    cluster
}
