use std::collections::HashMap;
use macroquad::prelude::trace;

use crate::{
    cfg::ZONE_SIZE,
    common::{
        Grid,
        algorithm::{
            astar::{AStarSettings, astar},
            distance::Distance,
        },
    },
    domain::{Terrain, ZoneGrid},
};

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum RoadCategory {
    North,
    South,
    East,
    West,
    Stairs,
}

#[derive(Clone, Copy)]
pub struct RoadConnection {
    pub category: RoadCategory,
    pub pos: (usize, usize),
    pub width: usize,
}

pub struct RoadBuilder {
    pub connections: Vec<RoadConnection>,
    pub road_grid: Grid<bool>,
}

impl RoadBuilder {
    pub fn new() -> Self {
        Self {
            connections: Vec::new(),
            road_grid: Grid::init(ZONE_SIZE.0, ZONE_SIZE.1, false),
        }
    }

    pub fn add_connection(&mut self, connection: RoadConnection) {
        self.connections.push(connection);
    }

    pub fn build_roads(&mut self, locked_grid: &Grid<bool>, zone_idx: usize) {
        if self.connections.is_empty() {
            return;
        }

        let grouped_bycat = self.group_connections_by_category();

        self.connect_horizontal_roads(&grouped_bycat, locked_grid, zone_idx);
        self.connect_vertical_roads(&grouped_bycat, locked_grid, zone_idx);
        self.connect_edge_roads(&grouped_bycat);
        self.connect_stairs(&grouped_bycat);
    }

    pub fn apply_roads_to_terrain(&self, terrain: &mut Grid<Terrain>, locked: &mut Grid<bool>, road_terrain: Terrain) {
        for (x, y, &is_road) in self.road_grid.iter_xy() {
            if is_road {
                terrain.set(x, y, road_terrain);
                locked.set(x, y, true);
            }
        }
    }

    fn group_connections_by_category(&self) -> HashMap<RoadCategory, Vec<RoadConnection>> {
        let mut grouped: HashMap<RoadCategory, Vec<RoadConnection>> = HashMap::new();
        
        for connection in &self.connections {
            grouped.entry(connection.category).or_default().push(*connection);
        }
        
        grouped
    }

    fn connect_horizontal_roads(&mut self, grouped: &HashMap<RoadCategory, Vec<RoadConnection>>, locked_grid: &Grid<bool>, zone_idx: usize) {
        let empty_east = vec![];
        let empty_west = vec![];
        let road_east = grouped.get(&RoadCategory::East).unwrap_or(&empty_east);
        let road_west = grouped.get(&RoadCategory::West).unwrap_or(&empty_west);

        for east_road in road_east {
            for west_road in road_west {
                let width = east_road.width.max(west_road.width);
                self.connect_points(east_road.pos, west_road.pos, width, locked_grid, zone_idx);
            }
        }
    }

    fn connect_vertical_roads(&mut self, grouped: &HashMap<RoadCategory, Vec<RoadConnection>>, locked_grid: &Grid<bool>, zone_idx: usize) {
        let empty_north = vec![];
        let empty_south = vec![];
        let road_north = grouped.get(&RoadCategory::North).unwrap_or(&empty_north);
        let road_south = grouped.get(&RoadCategory::South).unwrap_or(&empty_south);

        for north_road in road_north {
            for south_road in road_south {
                let width = north_road.width.max(south_road.width);
                self.connect_points(north_road.pos, south_road.pos, width, locked_grid, zone_idx);
            }
        }
    }

    fn connect_edge_roads(&mut self, grouped: &HashMap<RoadCategory, Vec<RoadConnection>>) {
        for (category, connections) in grouped {
            if *category == RoadCategory::Stairs {
                continue;
            }
            for connection in connections {
                self.road_grid.set(connection.pos.0, connection.pos.1, true);
                self.connect_to_nearest_road(connection);
            }
        }
    }

    fn connect_stairs(&mut self, grouped: &HashMap<RoadCategory, Vec<RoadConnection>>) {
        if let Some(stair_connections) = grouped.get(&RoadCategory::Stairs) {
            for stair in stair_connections {
                self.connect_to_nearest_road(stair);
            }
        }
    }

    fn connect_points(&mut self, a: (usize, usize), b: (usize, usize), width: usize, locked_grid: &Grid<bool>, zone_idx: usize) {
        let grid_bool = ZoneGrid::rand_bool(zone_idx as u32);

        let result = astar(AStarSettings {
            start: [a.0, a.1],
            is_goal: |p| p[0] == b.0 && p[1] == b.1,
            cost: |_, [x, y]| {
                let r = grid_bool.get(x, y).unwrap();

                if *locked_grid.get(x, y).unwrap_or(&true) {
                    return 100.0;
                }

                let rand_cost = match r {
                    true => 10.0,
                    false => 1.0,
                };

                1.0 + rand_cost
            },
            heuristic: |[x, y]| {
                0.1 * Distance::chebyshev([x as i32, y as i32, 0], [b.0 as i32, b.1 as i32, 0])
            },
            neighbors: |[x, y]| get_in_bound_neighbors((x, y)),
            max_depth: 10000,
            max_cost: None,
        });

        if result.is_success {
            for [s_x, s_y] in result.path {
                for w1 in 0..width {
                    for w2 in 0..width {
                        let x = s_x + w1;
                        let y = s_y + w2;

                        if in_zone_bounds(x, y) && !*locked_grid.get(x, y).unwrap_or(&true) {
                            self.road_grid.set(x, y, true);
                        }
                    }
                }
            }
        } else {
            trace!("failed to connect road!");
        }
    }

    fn connect_to_nearest_road(&mut self, connection: &RoadConnection) {
        let mut nearest_pos = None;
        let mut min_distance = f32::INFINITY;

        for (x, y, &is_road) in self.road_grid.iter_xy() {
            if is_road && (x != connection.pos.0 || y != connection.pos.1) {
                let distance = Distance::manhattan(
                    [connection.pos.0 as i32, connection.pos.1 as i32, 0],
                    [x as i32, y as i32, 0],
                );

                if distance < min_distance {
                    min_distance = distance;
                    nearest_pos = Some((x, y));
                }
            }
        }

        if let Some(target_pos) = nearest_pos {
            let dummy_locked = Grid::init(ZONE_SIZE.0, ZONE_SIZE.1, false);
            self.connect_points(connection.pos, target_pos, connection.width, &dummy_locked, 0);
        }
    }
}

fn in_zone_bounds(x: usize, y: usize) -> bool {
    x < ZONE_SIZE.0 - 1 && y < ZONE_SIZE.1 - 1
}

fn get_in_bound_neighbors(v: (usize, usize)) -> Vec<[usize; 2]> {
    let mut n = vec![];
    let (x, y) = v;

    if x > 0 {
        n.push([x - 1, y]);

        if y > 0 {
            n.push([x - 1, y - 1]);
        }

        if y < ZONE_SIZE.1 - 1 {
            n.push([x - 1, y + 1]);
        }
    }

    if x < ZONE_SIZE.0 - 1 {
        n.push([x + 1, y]);

        if y > 0 {
            n.push([x + 1, y - 1]);
        }

        if y < ZONE_SIZE.1 - 1 {
            n.push([x + 1, y + 1]);
        }
    }

    if y > 0 {
        n.push([x, y - 1]);
    }

    if y < ZONE_SIZE.1 - 1 {
        n.push([x, y + 1]);
    }

    n
}