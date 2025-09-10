use std::collections::{HashMap, HashSet};

use crate::{
    cfg::{MAP_SIZE, SURFACE_LEVEL_Z},
    common::{Direction, Perlin, Rand},
    domain::world::generation::{RiverNetwork, RiverSegment, RiverType},
    rendering::zone_idx,
};

pub struct OverworldRiverGenerator;

impl OverworldRiverGenerator {
    pub fn generate_rivers(seed: u32) -> HashMap<usize, RiverNetwork> {
        let mut networks = HashMap::new();

        // Generate rivers at surface level
        let network = Self::generate_rivers_for_layer(SURFACE_LEVEL_Z, seed);
        networks.insert(SURFACE_LEVEL_Z, network);

        // Generate underground rivers for cavern levels
        for z in (SURFACE_LEVEL_Z + 1)..MAP_SIZE.2 {
            let cavern_network = Self::generate_cavern_rivers(z, seed);
            networks.insert(z, cavern_network);
        }

        networks
    }

    fn generate_cavern_rivers(z: usize, seed: u32) -> RiverNetwork {
        let mut network = RiverNetwork::default();
        let mut rand = Rand::seed(seed + z as u32 + 5000); // Different seed offset for caverns

        // Cavern rivers use different perlin noise for underground water flow patterns
        let cave_perlin = Perlin::new(seed + 2000 + z as u32, 0.08, 3, 2.0);

        // More underground rivers for better coverage
        let num_underground_rivers = rand.range_n(4, 7);
        let mut river_sources = Vec::new();

        // Create underground river sources at random positions
        for _ in 0..num_underground_rivers {
            let x = rand.range_n(0, MAP_SIZE.0 as i32);
            let y = rand.range_n(0, MAP_SIZE.1 as i32);
            river_sources.push((x, y));
        }

        // Create more underground lakes/pools that rivers flow between
        let num_pools = rand.range_n(5, 9);
        let mut pools = Vec::new();
        for _ in 0..num_pools {
            let x = rand.range_n(2, MAP_SIZE.0 as i32 - 2);
            let y = rand.range_n(2, MAP_SIZE.1 as i32 - 2);
            pools.push((x as usize, y as usize));
        }

        // Flow rivers between sources and pools
        for &(start_x, start_y) in &river_sources {
            // Find nearest pool to flow toward
            if let Some(&(pool_x, pool_y)) = pools.iter().min_by_key(|&&(px, py)| {
                let dx = (start_x - px as i32).abs();
                let dy = (start_y - py as i32).abs();
                dx + dy
            }) {
                // Flow underground river toward pool
                Self::flow_cavern_river(
                    start_x,
                    start_y,
                    pool_x as i32,
                    pool_y as i32,
                    RiverType::Stream, // Underground rivers are typically streams
                    &mut network,
                    &cave_perlin,
                    &mut rand,
                    z,
                );
            }
        }

        // Connect pools to each other with more connections
        for i in 0..pools.len() {
            // Connect to next pool in sequence
            if i + 1 < pools.len() {
                let (x1, y1) = pools[i];
                let (x2, y2) = pools[i + 1];
                Self::flow_cavern_river(
                    x1 as i32,
                    y1 as i32,
                    x2 as i32,
                    y2 as i32,
                    RiverType::Creek,
                    &mut network,
                    &cave_perlin,
                    &mut rand,
                    z,
                );
            }

            // Add some cross-connections for more interconnected network
            if pools.len() > 4 && rand.bool(0.4) {
                let other_idx = rand.range_n(0, pools.len() as i32) as usize;
                if other_idx != i && other_idx != i + 1 && i != 0 {
                    let (x1, y1) = pools[i];
                    let (x2, y2) = pools[other_idx];
                    Self::flow_cavern_river(
                        x1 as i32,
                        y1 as i32,
                        x2 as i32,
                        y2 as i32,
                        RiverType::Creek,
                        &mut network,
                        &cave_perlin,
                        &mut rand,
                        z,
                    );
                }
            }
        }

        network
    }

    fn generate_rivers_for_layer(z: usize, seed: u32) -> RiverNetwork {
        let mut network = RiverNetwork::default();
        let mut rand = Rand::seed(seed + z as u32);
        let elevation_perlin = Perlin::new(seed + 1000 + z as u32, 0.05, 3, 2.0); // Lower frequency for larger features

        // Phase 1: Create a few major rivers that can start outside the map
        let major_rivers = Self::create_major_rivers(z, &mut rand, &elevation_perlin);

        // Phase 2: Add many small tributaries that flow into the major rivers
        let tributaries = Self::create_tributaries(z, &mut rand, &elevation_perlin, &major_rivers);

        // Phase 3: Flow all rivers with proper physics
        Self::flow_all_rivers(
            &major_rivers,
            &tributaries,
            &mut network,
            &elevation_perlin,
            &mut rand,
            z,
        );

        network
    }

    fn create_major_rivers(
        _z: usize,
        rand: &mut Rand,
        _elevation_perlin: &Perlin,
    ) -> Vec<(i32, i32)> {
        let mut major_rivers = Vec::new();
        let num_major_rivers = rand.range_n(3, 6); // 3-5 major rivers for better coverage

        for _ in 0..num_major_rivers {
            // Major rivers can start outside the map for more realistic flow
            let start_x = rand.range_n(-5, MAP_SIZE.0 as i32 + 5);
            let start_y = rand.range_n(-5, MAP_SIZE.1 as i32 + 5);

            // Always add the river source
            major_rivers.push((start_x, start_y));
        }

        major_rivers
    }

    fn create_tributaries(
        _z: usize,
        rand: &mut Rand,
        elevation_perlin: &Perlin,
        _major_rivers: &[(i32, i32)],
    ) -> Vec<(usize, usize)> {
        let mut tributaries = Vec::new();
        let min_distance_between_tributaries = 4; // Smaller tributaries can be closer

        // Generate many small tributaries across the map
        for x in 0..MAP_SIZE.0 {
            for y in 0..MAP_SIZE.1 {
                let elevation = elevation_perlin.get(x as f32, y as f32);

                // Higher elevation = more likely to spawn a tributary
                let tributary_chance = (elevation + 1.0) * 0.15;

                if rand.random() < tributary_chance {
                    // Check minimum distance from other tributaries
                    let too_close = tributaries.iter().any(|&(tx, ty)| {
                        let dist = ((x as f32 - tx as f32).powi(2)
                            + (y as f32 - ty as f32).powi(2))
                        .sqrt();
                        dist < min_distance_between_tributaries as f32
                    });

                    if !too_close {
                        tributaries.push((x, y));
                    }
                }
            }
        }

        tributaries
    }

    fn flow_all_rivers(
        major_rivers: &[(i32, i32)],
        tributaries: &[(usize, usize)],
        network: &mut RiverNetwork,
        elevation_perlin: &Perlin,
        rand: &mut Rand,
        z: usize,
    ) {
        // Flow major rivers first - they get priority and flow longest
        for &(start_x, start_y) in major_rivers {
            Self::flow_river(
                start_x,
                start_y,
                RiverType::River,
                1000,
                network,
                elevation_perlin,
                rand,
                z,
            );
        }

        // Flow tributaries - they flow until they hit a major river or edge
        for &(x, y) in tributaries {
            Self::flow_river(
                x as i32,
                y as i32,
                RiverType::Creek,
                100,
                network,
                elevation_perlin,
                rand,
                z,
            );
        }

        // Upgrade river segments based on confluences
        Self::upgrade_confluence_segments(network);
    }

    fn flow_cavern_river(
        start_x: i32,
        start_y: i32,
        target_x: i32,
        target_y: i32,
        river_type: RiverType,
        network: &mut RiverNetwork,
        cave_perlin: &Perlin,
        rand: &mut Rand,
        z: usize,
    ) {
        let mut x = start_x;
        let mut y = start_y;
        let mut visited = HashSet::new();
        let max_steps = 200;

        for _step in 0..max_steps {
            // Check bounds
            if x < 0 || y < 0 || x >= MAP_SIZE.0 as i32 || y >= MAP_SIZE.1 as i32 {
                break;
            }

            let current_zone_idx = zone_idx(x as usize, y as usize, z);

            // Avoid loops
            if visited.contains(&current_zone_idx) {
                break;
            }
            visited.insert(current_zone_idx);

            // Add to network
            network.nodes.insert(current_zone_idx);

            // Check if we reached target
            if (x - target_x).abs() <= 1 && (y - target_y).abs() <= 1 {
                break;
            }

            // Find next position toward target with some randomness
            let dx = (target_x - x).signum();
            let dy = (target_y - y).signum();

            // Add some meandering based on perlin noise
            let noise = cave_perlin.get(x as f32 * 0.5, y as f32 * 0.5);

            // Only move in cardinal directions (no diagonals between zones)
            let (next_x, next_y) = if dx != 0 && dy != 0 {
                // Both directions available - choose one based on noise and randomness
                if rand.random() < 0.5 + noise * 0.2 {
                    (x + dx, y) // Move horizontally
                } else {
                    (x, y + dy) // Move vertically
                }
            } else if dx != 0 {
                // Only horizontal movement needed
                (x + dx, y)
            } else if dy != 0 {
                // Only vertical movement needed
                (x, y + dy)
            } else {
                // At target or need to meander
                let meander = if rand.random() < 0.3 + noise * 0.2 {
                    if rand.bool(0.5) {
                        if rand.bool(0.5) { (1, 0) } else { (-1, 0) } // Horizontal meander
                    } else if rand.bool(0.5) {
                        (0, 1)
                    } else {
                        (0, -1)
                    }
                } else {
                    (0, 0)
                };
                (x + meander.0, y + meander.1)
            };

            // Ensure next position is valid
            if next_x >= 0
                && next_y >= 0
                && next_x < MAP_SIZE.0 as i32
                && next_y < MAP_SIZE.1 as i32
            {
                let from = current_zone_idx;
                let to = zone_idx(next_x as usize, next_y as usize, z);
                let _direction = Self::get_direction(x, y, next_x, next_y);

                network.edges.insert(
                    (from, to),
                    RiverSegment {
                        river_type,
                        depth: Self::calculate_depth(river_type),
                    },
                );

                x = next_x;
                y = next_y;
            } else {
                break;
            }
        }
    }

    fn flow_river(
        start_x: i32,
        start_y: i32,
        initial_type: RiverType,
        max_length: usize,
        network: &mut RiverNetwork,
        elevation_perlin: &Perlin,
        rand: &mut Rand,
        z: usize,
    ) {
        let mut x = start_x;
        let mut y = start_y;
        let river_type = initial_type;
        let mut visited = HashSet::new();
        let mut previous_direction = None;

        // Length is proportional to river size
        let flow_length = match river_type {
            RiverType::Creek => max_length.min(100),
            RiverType::Stream => max_length.min(200),
            RiverType::River => max_length.min(500),
            RiverType::MightyRiver => max_length, // No limit for mighty rivers
        };

        for step in 0..flow_length {
            // Check if we're within map bounds
            if x < 0 || y < 0 || x >= MAP_SIZE.0 as i32 || y >= MAP_SIZE.1 as i32 {
                // River flows off map - this is fine for major rivers
                if step > 10 {
                    // But only if we've flowed for a bit
                    break;
                }
                // Otherwise continue to enter the map
            } else {
                let zone_idx = zone_idx(x as usize, y as usize, z);

                // Avoid loops
                if visited.contains(&zone_idx) {
                    break;
                }
                visited.insert(zone_idx);

                // Check for confluence with existing river
                if network.nodes.contains(&zone_idx) {
                    // Merge into existing river and stop
                    network.confluences.push(zone_idx);
                    break;
                }

                network.nodes.insert(zone_idx);
            }

            // Find next position (strongly prefer downhill and forward)
            let next = Self::find_next_position_with_direction(
                x,
                y,
                previous_direction.clone(),
                elevation_perlin,
                rand,
            );

            if let Some((next_x, next_y)) = next {
                // Add segment if both positions are in map
                if x >= 0
                    && y >= 0
                    && x < MAP_SIZE.0 as i32
                    && y < MAP_SIZE.1 as i32
                    && next_x >= 0
                    && next_y >= 0
                    && next_x < MAP_SIZE.0 as i32
                    && next_y < MAP_SIZE.1 as i32
                {
                    let from = zone_idx(x as usize, y as usize, z);
                    let to = zone_idx(next_x as usize, next_y as usize, z);
                    let direction = Self::get_direction(x, y, next_x, next_y);

                    network.edges.insert(
                        (from, to),
                        RiverSegment {
                            river_type,
                            depth: Self::calculate_depth(river_type),
                        },
                    );

                    previous_direction = Some(direction);
                }

                x = next_x;
                y = next_y;
            } else {
                break; // No valid next position
            }
        }
    }

    fn find_next_position_with_direction(
        x: i32,
        y: i32,
        previous_direction: Option<Direction>,
        elevation_perlin: &Perlin,
        rand: &mut Rand,
    ) -> Option<(i32, i32)> {
        let current_elevation =
            if x >= 0 && y >= 0 && x < MAP_SIZE.0 as i32 && y < MAP_SIZE.1 as i32 {
                elevation_perlin.get(x as f32, y as f32)
            } else {
                0.5 // Assume moderate elevation outside map
            };

        // Define all possible directions with preferences
        let mut candidates = Vec::new();

        // Cardinal directions
        let directions = [
            (0, 1, Direction::South),  // South (downward on map)
            (1, 0, Direction::East),   // East
            (-1, 0, Direction::West),  // West
            (0, -1, Direction::North), // North (upward on map)
        ];

        for (dx, dy, dir) in directions {
            let nx = x + dx;
            let ny = y + dy;

            // Calculate elevation at neighbor
            let neighbor_elevation =
                if nx >= -5 && ny >= -5 && nx < MAP_SIZE.0 as i32 + 5 && ny < MAP_SIZE.1 as i32 + 5
                {
                    if nx >= 0 && ny >= 0 && nx < MAP_SIZE.0 as i32 && ny < MAP_SIZE.1 as i32 {
                        elevation_perlin.get(nx as f32, ny as f32)
                    } else {
                        0.3 // Lower elevation outside map to encourage flow off-map
                    }
                } else {
                    continue; // Too far outside map
                };

            // Calculate flow preference
            let elevation_diff = current_elevation - neighbor_elevation;
            let mut preference = elevation_diff * 10.0; // Strong preference for downhill

            // Strongly discourage backward flow
            if let Some(ref prev_dir) = previous_direction {
                if dir == Self::opposite_direction(prev_dir.clone()) {
                    preference -= 5.0; // Heavy penalty for going backward
                } else if dir == *prev_dir {
                    preference += 2.0; // Bonus for continuing same direction
                }
            }

            // Add some noise for natural meandering
            preference += rand.random() * 0.5;

            // Only consider if it's generally downhill or slightly uphill
            if preference > -0.2 {
                candidates.push(((nx, ny), preference));
            }
        }

        if candidates.is_empty() {
            return None;
        }

        // Sort by preference and select from top options
        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let selection_range = candidates.len().min(2); // Less randomness for more consistent flow
        let selected_idx = rand.range_n(0, selection_range as i32) as usize;

        Some(candidates[selected_idx].0)
    }

    fn opposite_direction(dir: Direction) -> Direction {
        match dir {
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::East => Direction::West,
            Direction::West => Direction::East,
        }
    }

    fn get_direction(from_x: i32, from_y: i32, to_x: i32, to_y: i32) -> Direction {
        if to_x > from_x {
            Direction::East
        } else if to_x < from_x {
            Direction::West
        } else if to_y > from_y {
            Direction::South
        } else {
            Direction::North
        }
    }

    fn calculate_depth(river_type: RiverType) -> f32 {
        match river_type {
            RiverType::Creek => 0.5,
            RiverType::Stream => 1.0,
            RiverType::River => 1.5,
            RiverType::MightyRiver => 2.0,
        }
    }

    fn upgrade_confluence_segments(network: &mut RiverNetwork) {
        // For each confluence, upgrade all downstream segments
        for &confluence in &network.confluences.clone() {
            let mut to_upgrade = vec![confluence];
            let mut visited = HashSet::new();

            while let Some(current) = to_upgrade.pop() {
                if visited.contains(&current) {
                    continue;
                }
                visited.insert(current);

                // Find all segments flowing from this point
                let downstream: Vec<_> = network
                    .edges
                    .iter()
                    .filter(|((from, _), _)| *from == current)
                    .map(|((from, to), _)| (*from, *to))
                    .collect();

                for (from, to) in downstream {
                    // Upgrade the segment
                    if let Some(segment) = network.edges.get_mut(&(from, to)) {
                        segment.river_type = segment.river_type.upgrade();
                        segment.depth = Self::calculate_depth(segment.river_type);
                    }

                    // Continue upgrading downstream
                    to_upgrade.push(to);
                }
            }
        }
    }
}
