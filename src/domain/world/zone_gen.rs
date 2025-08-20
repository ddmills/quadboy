use bevy_ecs::{hierarchy::ChildOf, world::World};

use crate::{
    cfg::ZONE_SIZE,
    common::{AStarSettings, Distance, Grid, Palette, Perlin, Rand, astar, bresenham_line},
    domain::{Map, Terrain, Zone, ZoneConstraintType, ZoneStatus},
    rendering::{Glyph, Position, RenderLayer, TrackZone, zone_local_to_world},
    states::CleanupStatePlay,
};

fn collect_constraint_positions(
    constraints: &crate::domain::ZoneConstraints,
) -> (Vec<(usize, usize)>, Vec<(usize, usize)>) {
    let mut river_positions = Vec::new();
    let mut path_positions = Vec::new();

    for (x, constraint_type) in constraints.south.iter().enumerate() {
        match constraint_type {
            ZoneConstraintType::River => river_positions.push((x, 0)),
            ZoneConstraintType::Path => path_positions.push((x, 0)),
            ZoneConstraintType::None => {}
        }
    }

    for (x, constraint_type) in constraints.north.iter().enumerate() {
        match constraint_type {
            ZoneConstraintType::River => river_positions.push((x, ZONE_SIZE.1 - 1)),
            ZoneConstraintType::Path => path_positions.push((x, ZONE_SIZE.1 - 1)),
            ZoneConstraintType::None => {}
        }
    }

    for (y, constraint_type) in constraints.west.iter().enumerate() {
        match constraint_type {
            ZoneConstraintType::River => river_positions.push((0, y)),
            ZoneConstraintType::Path => path_positions.push((0, y)),
            ZoneConstraintType::None => {}
        }
    }

    for (y, constraint_type) in constraints.east.iter().enumerate() {
        match constraint_type {
            ZoneConstraintType::River => river_positions.push((ZONE_SIZE.0 - 1, y)),
            ZoneConstraintType::Path => path_positions.push((ZONE_SIZE.0 - 1, y)),
            ZoneConstraintType::None => {}
        }
    }

    (river_positions, path_positions)
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
                        base_cost * noise_factor * diagonal_bonus
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
                };

                let result = astar(settings);
                if result.is_success && result.cost < best_cost {
                    best_path = Some(result.path);
                    best_cost = result.cost;
                    best_to = to_idx;
                } else if !result.is_success {
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

fn generate_paths(positions: &[(usize, usize)], terrain: &mut Grid<Terrain>) {
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
                        if let Some(existing_terrain) = terrain.get(to.0, to.1) {
                            if *existing_terrain == Terrain::River {
                                return f32::INFINITY;
                            }
                        }
                        1.0
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
                    max_depth: 1000,
                };

                let result = astar(settings);
                if result.is_success && result.cost < best_cost {
                    best_path = Some(result.path);
                    best_cost = result.cost;
                    best_to = to_idx;
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

pub fn gen_zone(world: &mut World, zone_idx: usize) {
    let mut rand = Rand::seed(zone_idx as u64);
    let map = world.resource::<Map>();
    let constraints = map.get_zone_constraints(zone_idx);
    let mut terrain = Grid::init_fill(ZONE_SIZE.0, ZONE_SIZE.1, |_x, _y| Terrain::Grass);
    let (river_positions, _path_positions) = collect_constraint_positions(&constraints);

    generate_rivers(&river_positions, &mut terrain, &mut rand, zone_idx);
    generate_paths(&_path_positions, &mut terrain);

    let zone_entity_id = world.spawn((ZoneStatus::Dormant, CleanupStatePlay)).id();

    Grid::init_fill(ZONE_SIZE.0, ZONE_SIZE.1, |x, y| {
        let wpos = zone_local_to_world(zone_idx, x, y);
        let terrain = terrain.get(x, y).unwrap_or(&Terrain::Dirt);

        let idx = terrain.tile();
        let (bg, fg) = terrain.colors();

        // trees
        if rand.bool(0.05) && *terrain == Terrain::Grass {
            world.spawn((
                Position::new(wpos.0, wpos.1, wpos.2),
                Glyph::new(64, Palette::DarkCyan, Palette::Orange).layer(RenderLayer::Actors),
                ChildOf(zone_entity_id),
                ZoneStatus::Dormant,
                TrackZone,
                CleanupStatePlay,
            ));
        }

        // Add terrain tiles
        world.spawn((
            Position::new(wpos.0, wpos.1, wpos.2),
            Glyph::idx(idx)
                .bg_opt(bg)
                .fg1_opt(fg)
                .layer(RenderLayer::Ground),
            ChildOf(zone_entity_id),
            ZoneStatus::Dormant,
            CleanupStatePlay,
        ));
    });

    world
        .entity_mut(zone_entity_id)
        .insert(Zone::new(zone_idx, terrain));
}
