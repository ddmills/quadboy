use std::collections::HashMap;

use crate::{
    cfg::ZONE_SIZE,
    common::{
        Grid,
        algorithm::{
            astar::{AStarSettings, astar},
            distance::Distance,
        },
    },
    domain::{RiverType, Terrain, ZoneGrid},
};

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum RiverCategory {
    North,
    South,
    East,
    West,
}

#[derive(Clone, Copy)]
pub struct RiverConnection {
    pub category: RiverCategory,
    pub pos: (usize, usize),
    pub river_type: RiverType,
}

pub struct RiverBuilder {
    pub connections: Vec<RiverConnection>,
    pub river_grid: Grid<Option<RiverType>>,
}

impl RiverBuilder {
    pub fn new() -> Self {
        Self {
            connections: Vec::new(),
            river_grid: Grid::init(ZONE_SIZE.0, ZONE_SIZE.1, None),
        }
    }

    pub fn add_connection(&mut self, connection: RiverConnection) {
        self.connections.push(connection);
    }

    pub fn build_rivers(&mut self, locked_grid: &Grid<bool>, zone_idx: usize) {
        if self.connections.is_empty() {
            return;
        }

        let grouped_bycat = self.group_connections_by_category();

        // Connect opposite edges (North-South, East-West)
        self.connect_north_south_rivers(&grouped_bycat, locked_grid, zone_idx);
        self.connect_east_west_rivers(&grouped_bycat, locked_grid, zone_idx);

        // Connect any remaining unconnected rivers
        self.connect_remaining_rivers(&grouped_bycat, locked_grid, zone_idx);
    }

    pub fn apply_rivers_to_terrain(&self, terrain: &mut Grid<Terrain>, locked: &mut Grid<bool>) {
        for (x, y, river_type) in self.river_grid.iter_xy() {
            if river_type.is_some() {
                terrain.set(x, y, Terrain::River);
                locked.set(x, y, true);
            }
        }
    }

    fn group_connections_by_category(&self) -> HashMap<RiverCategory, Vec<RiverConnection>> {
        let mut grouped: HashMap<RiverCategory, Vec<RiverConnection>> = HashMap::new();

        for connection in &self.connections {
            grouped
                .entry(connection.category)
                .or_default()
                .push(*connection);
        }

        grouped
    }

    fn connect_north_south_rivers(
        &mut self,
        grouped: &HashMap<RiverCategory, Vec<RiverConnection>>,
        locked_grid: &Grid<bool>,
        zone_idx: usize,
    ) {
        let empty_north = vec![];
        let empty_south = vec![];
        let river_north = grouped.get(&RiverCategory::North).unwrap_or(&empty_north);
        let river_south = grouped.get(&RiverCategory::South).unwrap_or(&empty_south);

        for north_river in river_north {
            for south_river in river_south {
                let river_type = north_river.river_type.min(south_river.river_type);
                self.connect_river_points(
                    north_river.pos,
                    south_river.pos,
                    river_type,
                    locked_grid,
                    zone_idx,
                );
                break; // Only connect first pair
            }
        }
    }

    fn connect_east_west_rivers(
        &mut self,
        grouped: &HashMap<RiverCategory, Vec<RiverConnection>>,
        locked_grid: &Grid<bool>,
        zone_idx: usize,
    ) {
        let empty_east = vec![];
        let empty_west = vec![];
        let river_east = grouped.get(&RiverCategory::East).unwrap_or(&empty_east);
        let river_west = grouped.get(&RiverCategory::West).unwrap_or(&empty_west);

        for east_river in river_east {
            for west_river in river_west {
                let river_type = east_river.river_type.min(west_river.river_type);
                self.connect_river_points(
                    east_river.pos,
                    west_river.pos,
                    river_type,
                    locked_grid,
                    zone_idx,
                );
                break; // Only connect first pair
            }
        }
    }

    fn connect_remaining_rivers(
        &mut self,
        grouped: &HashMap<RiverCategory, Vec<RiverConnection>>,
        locked_grid: &Grid<bool>,
        zone_idx: usize,
    ) {
        let mut connected_positions = Vec::new();

        // Track which positions have been connected
        for (x, y, river_type) in self.river_grid.iter_xy() {
            if river_type.is_some() {
                connected_positions.push((x, y));
            }
        }

        // Connect any unconnected rivers to nearest connected point or center
        for (_category, connections) in grouped {
            for connection in connections {
                if !connected_positions.contains(&connection.pos) {
                    if let Some(nearest) =
                        self.find_nearest_river_point(connection.pos, &connected_positions)
                    {
                        self.connect_river_points(
                            connection.pos,
                            nearest,
                            connection.river_type,
                            locked_grid,
                            zone_idx,
                        );
                    } else {
                        // Connect to zone center
                        let center = (ZONE_SIZE.0 / 2, ZONE_SIZE.1 / 2);
                        self.connect_river_points(
                            connection.pos,
                            center,
                            connection.river_type,
                            locked_grid,
                            zone_idx,
                        );
                    }
                    connected_positions.push(connection.pos);
                }
            }
        }
    }

    fn find_nearest_river_point(
        &self,
        pos: (usize, usize),
        points: &[(usize, usize)],
    ) -> Option<(usize, usize)> {
        points
            .iter()
            .min_by_key(|&&(px, py)| {
                let dx = (pos.0 as i32 - px as i32).abs();
                let dy = (pos.1 as i32 - py as i32).abs();
                dx + dy // Manhattan distance
            })
            .copied()
    }

    fn connect_river_points(
        &mut self,
        a: (usize, usize),
        b: (usize, usize),
        river_type: RiverType,
        locked_grid: &Grid<bool>,
        zone_idx: usize,
    ) {
        // Use a seeded random grid for natural river meandering
        let seed = (a.0 * 1000 + a.1 * 100 + b.0 * 10 + b.1 + zone_idx) as u32;
        let grid_bool = ZoneGrid::rand_bool(seed);

        let result = astar(AStarSettings {
            start: [a.0, a.1],
            is_goal: |p| p[0] == b.0 && p[1] == b.1,
            cost: |_, [x, y]| {
                // Rivers prefer to flow through unlocked tiles but can flow through locked ones
                let locked_cost = if *locked_grid.get(x, y).unwrap_or(&false) {
                    5.0 // Higher cost for locked tiles but not prohibitive
                } else {
                    1.0
                };

                // Use random grid to create meandering effect
                let r = grid_bool.get(x, y).unwrap_or(&false);
                let meander_cost = if *r { 3.0 } else { 1.0 };

                locked_cost * meander_cost
            },
            heuristic: |[x, y]| {
                // Lower weight for more meandering
                0.5 * Distance::manhattan([x as i32, y as i32, 0], [b.0 as i32, b.1 as i32, 0])
            },
            neighbors: |[x, y]| get_river_neighbors((x, y), river_type),
            max_depth: 10000,
            max_cost: None,
        });

        if result.is_success {
            let width = river_type.width();
            for [path_x, path_y] in result.path {
                // Apply width similar to road builder
                for w1 in 0..width {
                    for w2 in 0..width {
                        let x = path_x.saturating_sub(width / 2) + w1;
                        let y = path_y.saturating_sub(width / 2) + w2;

                        if x < ZONE_SIZE.0 && y < ZONE_SIZE.1 {
                            // Use manhattan distance for diamond-shaped rivers
                            let dist_from_center = w1.abs_diff(width / 2) + w2.abs_diff(width / 2);
                            if dist_from_center <= width / 2 {
                                self.river_grid.set(x, y, Some(river_type));
                            }
                        }
                    }
                }
            }
        }
    }
}

fn get_river_neighbors(pos: (usize, usize), river_type: RiverType) -> Vec<[usize; 2]> {
    let mut neighbors = vec![];
    let (x, y) = pos;

    // Width-1 rivers (creeks) should only use cardinal directions for more natural flow
    let allow_diagonals = river_type.width() > 1;

    // Cardinals - always allowed
    if x > 0 {
        neighbors.push([x - 1, y]);
    }

    if x < ZONE_SIZE.0 - 1 {
        neighbors.push([x + 1, y]);
    }

    if y > 0 {
        neighbors.push([x, y - 1]);
    }

    if y < ZONE_SIZE.1 - 1 {
        neighbors.push([x, y + 1]);
    }

    // Diagonals - only for wider rivers
    if allow_diagonals {
        if x > 0 {
            if y > 0 {
                neighbors.push([x - 1, y - 1]);
            }
            if y < ZONE_SIZE.1 - 1 {
                neighbors.push([x - 1, y + 1]);
            }
        }

        if x < ZONE_SIZE.0 - 1 {
            if y > 0 {
                neighbors.push([x + 1, y - 1]);
            }
            if y < ZONE_SIZE.1 - 1 {
                neighbors.push([x + 1, y + 1]);
            }
        }
    }

    neighbors
}

impl RiverType {
    fn max(self, other: RiverType) -> RiverType {
        match (self, other) {
            (RiverType::MightyRiver, _) | (_, RiverType::MightyRiver) => RiverType::MightyRiver,
            (RiverType::River, _) | (_, RiverType::River) => RiverType::River,
            (RiverType::Stream, _) | (_, RiverType::Stream) => RiverType::Stream,
            (RiverType::Creek, RiverType::Creek) => RiverType::Creek,
        }
    }

    fn min(self, other: RiverType) -> RiverType {
        match (self, other) {
            (RiverType::Creek, _) | (_, RiverType::Creek) => RiverType::Creek,
            (RiverType::Stream, _) | (_, RiverType::Stream) => RiverType::Stream,
            (RiverType::River, _) | (_, RiverType::River) => RiverType::River,
            (RiverType::MightyRiver, RiverType::MightyRiver) => RiverType::MightyRiver,
        }
    }
}
