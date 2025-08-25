use std::collections::HashMap;

use crate::{
    cfg::{MAP_SIZE, SURFACE_LEVEL_Z},
    common::{
        Grid, Perlin,
        algorithm::astar::{AStarSettings, astar},
    },
    domain::world::generation::{OverworldTown, RoadNetwork, RoadSegment, RoadType},
    rendering::{zone_idx, zone_xyz},
};

pub struct OverworldRoadGenerator;

impl OverworldRoadGenerator {
    pub fn generate_roads(towns: &HashMap<usize, OverworldTown>, seed: u32) -> RoadNetwork {
        let mut network = RoadNetwork::default();

        if towns.is_empty() {
            return network;
        }

        let town_positions: Vec<(usize, (usize, usize, usize))> = towns
            .keys()
            .map(|&zone_idx| {
                let pos = zone_xyz(zone_idx);
                (zone_idx, pos)
            })
            .collect();

        // Connect each town to its nearest neighbors
        for (town_idx, town_pos) in &town_positions {
            let mut distances: Vec<(usize, f32)> = town_positions
                .iter()
                .filter(|(other_idx, _)| other_idx != town_idx)
                .map(|(other_idx, other_pos)| {
                    let distance = Self::calculate_distance(*town_pos, *other_pos);
                    (*other_idx, distance)
                })
                .collect();

            distances.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

            let max_connections = if distances.len() <= 3 {
                distances.len()
            } else {
                3
            };
            let max_distance = 15.0; // Maximum distance to connect towns

            for (other_idx, distance) in distances.into_iter().take(max_connections) {
                if distance <= max_distance && !Self::has_connection(&network, *town_idx, other_idx)
                {
                    Self::add_road_connection(&mut network, *town_idx, other_idx, distance, seed);
                }
            }
        }

        network
    }

    fn calculate_distance(pos1: (usize, usize, usize), pos2: (usize, usize, usize)) -> f32 {
        let dx = pos1.0 as f32 - pos2.0 as f32;
        let dy = pos1.1 as f32 - pos2.1 as f32;
        let dz = pos1.2 as f32 - pos2.2 as f32;

        (dx * dx + dy * dy + dz * dz * 0.1).sqrt()
    }

    fn has_connection(network: &RoadNetwork, zone1: usize, zone2: usize) -> bool {
        network.edges.contains_key(&(zone1, zone2)) || network.edges.contains_key(&(zone2, zone1))
    }

    fn add_road_connection(
        network: &mut RoadNetwork,
        zone1: usize,
        zone2: usize,
        distance: f32,
        seed: u32,
    ) {
        let road_type = Self::determine_road_type(distance);

        let intermediate_zones = Self::find_astar_path(zone1, zone2, seed);
        
        // Create a full path including start and end zones
        let mut full_path = vec![zone1];
        full_path.extend(intermediate_zones);
        full_path.push(zone2);
        
        // Create edges between consecutive zones in the path
        for window in full_path.windows(2) {
            let from = window[0];
            let to = window[1];
            
            let segment = RoadSegment {
                road_type,
                length: 1.0, // Distance between adjacent zones is always 1
            };
            
            network.edges.insert((from, to), segment.clone());
            network.edges.insert((to, from), segment);
            network.nodes.insert(from);
            network.nodes.insert(to);
        }
    }

    fn determine_road_type(distance: f32) -> RoadType {
        if distance <= 8.0 {
            RoadType::Footpath
        } else if distance <= 15.0 {
            RoadType::Road
        } else {
            RoadType::RoyalHighway
        }
    }

    fn find_astar_path(zone1: usize, zone2: usize, seed: u32) -> Vec<usize> {
        let pos1 = zone_xyz(zone1);
        let pos2 = zone_xyz(zone2);

        let start = (pos1.0, pos1.1);
        let goal = (pos2.0, pos2.1);

        let mut perlin = Perlin::new(seed + 500, 0.1, 1, 1.5);

        let min_x = pos1.0.min(pos2.0).saturating_sub(5);
        let max_x = (pos1.0.max(pos2.0) + 5).min(MAP_SIZE.0 - 1);
        let min_y = pos1.1.min(pos2.1).saturating_sub(5);
        let max_y = (pos1.1.max(pos2.1) + 5).min(MAP_SIZE.1 - 1);

        let grid_width = max_x - min_x + 1;
        let grid_height = max_y - min_y + 1;

        let terrain_costs = Grid::init_fill(grid_width, grid_height, |grid_x, grid_y| {
            let world_x = min_x + grid_x;
            let world_y = min_y + grid_y;
            1.0 + perlin.get(world_x as f32, world_y as f32) * 2.0 // 1.0 to 3.0 range
        });

        let settings = AStarSettings {
            start,
            is_goal: move |pos: (usize, usize)| pos == goal,
            cost: move |_from: (usize, usize), to: (usize, usize)| {
                // Convert world coordinates to grid coordinates
                if to.0 >= min_x && to.0 <= max_x && to.1 >= min_y && to.1 <= max_y {
                    let grid_x = to.0 - min_x;
                    let grid_y = to.1 - min_y;
                    *terrain_costs.get(grid_x, grid_y).unwrap_or(&1.0)
                } else {
                    1.0
                }
            },
            heuristic: move |pos: (usize, usize)| {
                Self::manhattan_distance(pos.0, pos.1, goal.0, goal.1)
            },
            neighbors: move |pos: (usize, usize)| {
                let mut neighbors = Vec::new();
                let (x, y) = pos;

                let candidates = [
                    (x.wrapping_sub(1), y),
                    (x + 1, y),
                    (x, y.wrapping_sub(1)),
                    (x, y + 1),
                ];

                for (nx, ny) in candidates {
                    // Bounds check
                    if nx < MAP_SIZE.0 && ny < MAP_SIZE.1 {
                        neighbors.push((nx, ny));
                    }
                }

                neighbors
            },
            max_depth: 1000,
            max_cost: None,
        };

        let result = astar(settings);

        if !result.is_success {
            return Vec::new();
        }

        let mut zone_path = Vec::new();
        for &(x, y) in result.path.iter().skip(1).rev().skip(1) {
            let zone_index = zone_idx(x, y, SURFACE_LEVEL_Z);
            zone_path.push(zone_index);
        }

        zone_path
    }

    fn manhattan_distance(x1: usize, y1: usize, x2: usize, y2: usize) -> f32 {
        let dx = (x1 as i32 - x2 as i32).abs() as f32;
        let dy = (y1 as i32 - y2 as i32).abs() as f32;
        dx + dy
    }
}
