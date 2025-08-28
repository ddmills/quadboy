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
    domain::{
        OverworldZone, PrefabId, SpawnConfig, Terrain, ZoneConstraintType, ZoneData, ZoneGrid,
    },
    rendering::zone_local_to_world,
};

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
enum RoadCategory {
    North,
    South,
    East,
    West,
    Stairs,
}

#[derive(Clone, Copy)]
struct RoadConnection {
    category: RoadCategory,
    pos: (usize, usize),
    width: usize,
}

pub struct ZoneFactory {
    pub zone_idx: usize,
    pub ozone: OverworldZone,
    pub terrain: Grid<Terrain>,
    pub entities: Grid<Vec<SpawnConfig>>,
    pub locked: Grid<bool>,
    pub roads: Vec<RoadConnection>,
    pub road_grid: Grid<bool>,
}

impl ZoneFactory {
    pub fn new(ozone: OverworldZone) -> Self {
        Self {
            zone_idx: ozone.zone_idx,
            ozone,
            terrain: Grid::init_fill(ZONE_SIZE.0, ZONE_SIZE.1, |_, _| Terrain::Grass),
            entities: Grid::init_fill(ZONE_SIZE.0, ZONE_SIZE.1, |_, _| vec![]),
            locked: Grid::init_fill(ZONE_SIZE.0, ZONE_SIZE.1, |_, _| false),
            roads: Vec::new(),
            road_grid: Grid::init_fill(ZONE_SIZE.0, ZONE_SIZE.1, |_, _| false),
        }
    }

    pub fn build(&mut self) -> ZoneData {
        self.apply_edge_constraints();
        self.apply_up_vertical_constraints();
        self.apply_vertical_constraints();
        self.apply_roads();

        let bt = self.ozone.biome_type;
        bt.apply_to_zone(self);

        self.to_zone_data()
    }

    pub fn to_zone_data(&self) -> ZoneData {
        ZoneData {
            zone_idx: self.zone_idx,
            terrain: self.terrain.clone(),
            entities: self.entities.clone(),
        }
    }

    pub fn push_entity(&mut self, x: usize, y: usize, config: SpawnConfig) {
        if let Some(ents) = self.entities.get_mut(x, y) {
            ents.push(config);
        };
    }

    pub fn lock_tile(&mut self, x: usize, y: usize) {
        self.locked.set(x, y, true);
    }

    pub fn set_terrain(&mut self, x: usize, y: usize, terrain: Terrain) {
        self.terrain.set(x, y, terrain);
    }

    pub fn is_locked_tile(&mut self, x: usize, y: usize) -> bool {
        *self.locked.get(x, y).unwrap_or(&false)
    }

    pub fn apply_vertical_constraints(&mut self) {
        for constraint in self.ozone.constraints.up.0.iter() {
            if constraint.constraint == ZoneConstraintType::StairDown {
                let (x, y) = constraint.position;
                let world_pos = zone_local_to_world(self.zone_idx, x, y);
                let stair_config = SpawnConfig::new(PrefabId::StairDown, world_pos);

                if let Some(entities_at_pos) = self.entities.get_mut(x, y) {
                    entities_at_pos.push(stair_config);
                }
                self.locked.set(x, y, true);
                self.roads.push(RoadConnection {
                    category: RoadCategory::Stairs,
                    pos: (x, y),
                    width: 1,
                });
            }
        }
    }

    pub fn apply_up_vertical_constraints(&mut self) {
        for constraint in self.ozone.constraints.down.0.iter() {
            if constraint.constraint == ZoneConstraintType::StairDown {
                let (x, y) = constraint.position;
                let world_pos = zone_local_to_world(self.zone_idx, x, y);
                let stair_config = SpawnConfig::new(PrefabId::StairUp, world_pos);

                if let Some(entities_at_pos) = self.entities.get_mut(x, y) {
                    entities_at_pos.push(stair_config);
                }
                self.locked.set(x, y, true);
                self.roads.push(RoadConnection {
                    category: RoadCategory::Stairs,
                    pos: (x, y),
                    width: 1,
                });
            }
        }
    }

    pub fn apply_edge_constraints(&mut self) {
        let mut c_road_pos = (0, 0);
        let mut c_road_width = 0;
        let mut on_road = false;
        let road_terrain = self.ozone.biome_type.get_road_terrain();

        // Collect constraints to avoid borrow checker issues
        let north_constraints: Vec<_> = self
            .ozone
            .constraints
            .north
            .0
            .iter()
            .cloned()
            .enumerate()
            .collect();
        let south_constraints: Vec<_> = self
            .ozone
            .constraints
            .south
            .0
            .iter()
            .cloned()
            .enumerate()
            .collect();
        let east_constraints: Vec<_> = self
            .ozone
            .constraints
            .east
            .0
            .iter()
            .cloned()
            .enumerate()
            .collect();
        let west_constraints: Vec<_> = self
            .ozone
            .constraints
            .west
            .0
            .iter()
            .cloned()
            .enumerate()
            .collect();

        // NORTH
        for (x, constraint) in north_constraints {
            match constraint {
                ZoneConstraintType::Road(_) => {
                    let y = 0;
                    self.terrain.set(x, y, road_terrain);
                    self.locked.set(x, y, true);

                    if !on_road {
                        c_road_pos = (x, y);
                    }

                    on_road = true;
                    c_road_width += 1;
                }
                ZoneConstraintType::Rock => {
                    let y = 0;
                    let world_pos = zone_local_to_world(self.zone_idx, x, y);
                    let boulder_config = SpawnConfig::new(PrefabId::Boulder, world_pos);
                    self.push_entity(x, y, boulder_config);
                    self.lock_tile(x, y);

                    if on_road {
                        self.roads.push(RoadConnection {
                            category: RoadCategory::North,
                            pos: (c_road_pos.0, c_road_pos.1),
                            width: c_road_width,
                        });
                        on_road = false;
                        c_road_width = 0;
                    }
                }
                _ => {
                    if on_road {
                        self.roads.push(RoadConnection {
                            category: RoadCategory::North,
                            pos: (c_road_pos.0, c_road_pos.1),
                            width: c_road_width,
                        });
                        on_road = false;
                        c_road_width = 0;
                    }
                }
            }
        }

        on_road = false;
        c_road_width = 0;

        // SOUTH
        for (x, constraint) in south_constraints {
            match constraint {
                ZoneConstraintType::Road(_) => {
                    let y = ZONE_SIZE.1 - 1;
                    self.terrain.set(x, y, road_terrain);
                    self.locked.set(x, y, true);

                    if !on_road {
                        c_road_pos = (x, y);
                    }

                    on_road = true;
                    c_road_width += 1;
                }
                ZoneConstraintType::Rock => {
                    let y = ZONE_SIZE.1 - 1;
                    let world_pos = zone_local_to_world(self.zone_idx, x, y);
                    let boulder_config = SpawnConfig::new(PrefabId::Boulder, world_pos);
                    self.push_entity(x, y, boulder_config);
                    self.lock_tile(x, y);

                    if on_road {
                        self.roads.push(RoadConnection {
                            category: RoadCategory::South,
                            pos: (c_road_pos.0, c_road_pos.1),
                            width: c_road_width,
                        });
                        on_road = false;
                        c_road_width = 0;
                    }
                }
                _ => {
                    if on_road {
                        self.roads.push(RoadConnection {
                            category: RoadCategory::South,
                            pos: (c_road_pos.0, c_road_pos.1),
                            width: c_road_width,
                        });
                        on_road = false;
                        c_road_width = 0;
                    }
                }
            }
        }

        on_road = false;
        c_road_width = 0;

        // EAST
        for (y, constraint) in east_constraints {
            match constraint {
                ZoneConstraintType::Road(_) => {
                    let x = ZONE_SIZE.0 - 1;
                    self.terrain.set(x, y, road_terrain);
                    self.locked.set(x, y, true);

                    if !on_road {
                        c_road_pos = (x, y);
                    }

                    on_road = true;
                    c_road_width += 1;
                }
                ZoneConstraintType::Rock => {
                    let x = ZONE_SIZE.0 - 1;
                    let world_pos = zone_local_to_world(self.zone_idx, x, y);
                    let boulder_config = SpawnConfig::new(PrefabId::Boulder, world_pos);
                    self.push_entity(x, y, boulder_config);
                    self.lock_tile(x, y);

                    if on_road {
                        self.roads.push(RoadConnection {
                            category: RoadCategory::East,
                            pos: (c_road_pos.0, c_road_pos.1),
                            width: c_road_width,
                        });
                        on_road = false;
                        c_road_width = 0;
                    }
                }
                _ => {
                    if on_road {
                        self.roads.push(RoadConnection {
                            category: RoadCategory::East,
                            pos: (c_road_pos.0, c_road_pos.1),
                            width: c_road_width,
                        });
                        on_road = false;
                        c_road_width = 0;
                    }
                }
            }
        }

        on_road = false;
        c_road_width = 0;

        // WEST
        for (y, constraint) in west_constraints {
            match constraint {
                ZoneConstraintType::Road(_) => {
                    let x = 0;
                    self.terrain.set(x, y, road_terrain);
                    self.locked.set(x, y, true);

                    if !on_road {
                        c_road_pos = (x, y);
                    }

                    on_road = true;
                    c_road_width += 1;
                }
                ZoneConstraintType::Rock => {
                    let x = 0;
                    let world_pos = zone_local_to_world(self.zone_idx, x, y);
                    let boulder_config = SpawnConfig::new(PrefabId::Boulder, world_pos);
                    self.push_entity(x, y, boulder_config);
                    self.lock_tile(x, y);

                    if on_road {
                        self.roads.push(RoadConnection {
                            category: RoadCategory::West,
                            pos: (c_road_pos.0, c_road_pos.1),
                            width: c_road_width,
                        });
                        on_road = false;
                        c_road_width = 0;
                    }
                }
                _ => {
                    if on_road {
                        self.roads.push(RoadConnection {
                            category: RoadCategory::West,
                            pos: (c_road_pos.0, c_road_pos.1),
                            width: c_road_width,
                        });
                        on_road = false;
                        c_road_width = 0;
                    }
                }
            }
        }
    }

    pub fn apply_roads(&mut self) {
        if self.roads.is_empty() {
            return;
        }

        let mut all_paths = Grid::init(ZONE_SIZE.0, ZONE_SIZE.1, false);
        let mut grouped_bycat: HashMap<RoadCategory, Vec<RoadConnection>> = HashMap::new();

        for r in self.roads.iter() {
            if let Some(v) = grouped_bycat.get_mut(&r.category) {
                v.push(*r);
            } else {
                grouped_bycat.insert(r.category, vec![*r]);
            }
        }

        let empty_east = vec![];
        let road_east = grouped_bycat
            .get(&RoadCategory::East)
            .unwrap_or(&empty_east);

        let empty_west = vec![];
        let road_west = grouped_bycat
            .get(&RoadCategory::West)
            .unwrap_or(&empty_west);

        // connect east/west, use widest road
        for p1 in road_east.iter() {
            for p2 in road_west.iter() {
                let width = p1.width.max(p2.width);

                self.connect_points(p1.pos, p2.pos, width);
            }
        }

        let road_north = grouped_bycat
            .get(&RoadCategory::North)
            .cloned()
            .unwrap_or_default();

        let road_south = grouped_bycat
            .get(&RoadCategory::South)
            .cloned()
            .unwrap_or_default();

        // connect north/south, use widest road
        for p1 in road_north.iter() {
            for p2 in road_south.iter() {
                let width = p1.width.max(p2.width);

                self.connect_points(p1.pos, p2.pos, width);
            }
        }

        for (cat, c) in grouped_bycat.iter() {
            if *cat == RoadCategory::Stairs {
                continue;
            }
            for i in c {
                self.road_grid.set(i.pos.0, i.pos.1, true);
                self.connect_nearest(i);
            }
        }

        for stairs in grouped_bycat.get(&RoadCategory::Stairs).iter() {
            for stair in stairs.iter() {
                self.connect_nearest(stair);
            }
        }

        let road_terrain = self.ozone.biome_type.get_road_terrain();

        // stamp road grid
        for (x, y, v) in self.road_grid.iter_xy() {
            if *v {
                self.terrain.set(x, y, road_terrain);
                self.locked.set(x, y, true);
            }
        }
    }

    fn connect_points(&mut self, a: (usize, usize), b: (usize, usize), width: usize) {
        let grid_bool = ZoneGrid::rand_bool(self.zone_idx as u32);

        let result = astar(AStarSettings {
            start: [a.0, a.1],
            is_goal: |p| p[0] == b.0 && p[1] == b.1,
            cost: |_, [x, y]| {
                let r = grid_bool.get(x, y).unwrap();

                if *self.locked.get(x, y).unwrap_or(&true) {
                    return 100.0;
                }

                let rand_cost = match r {
                    true => 10.0,
                    false => 1.0,
                };

                1. + rand_cost
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

                        if in_zone_bounds(x, y) && !self.is_locked_tile(x, y) {
                            self.road_grid.set(x, y, true);
                        }
                    }
                }
            }
        } else {
            trace!("failed to connect road!");
        }
    }

    fn connect_nearest(&mut self, c: &RoadConnection) {
        let mut nearest_pos = None;
        let mut min_distance = f32::INFINITY;

        for (x, y, is_road) in self.road_grid.iter_xy() {
            if *is_road && x != c.pos.0 && y != c.pos.1 {
                let distance = Distance::manhattan(
                    [c.pos.0 as i32, c.pos.1 as i32, 0],
                    [x as i32, y as i32, 0],
                );

                if distance < min_distance {
                    min_distance = distance;
                    nearest_pos = Some((x, y));
                }
            }
        }

        if let Some(target_pos) = nearest_pos {
            self.connect_points(c.pos, target_pos, c.width);
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
