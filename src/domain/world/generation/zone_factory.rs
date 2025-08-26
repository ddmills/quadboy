use std::collections::HashMap;

use bevy_ecs::relationship::RelationshipSourceCollection;
use macroquad::prelude::trace;
use rand::rand_core::le;

use crate::{
    cfg::ZONE_SIZE,
    common::{
        Grid,
        algorithm::{
            astar::{AStarSettings, astar},
            distance::Distance,
        },
        remap,
    },
    domain::{
        BiomeBuilder, CavernBiomeBuilder, DesertBiomeBuilder, ForestBiomeBuilder,
        OpenAirBiomeBuilder, OverworldZone, PrefabId, RoadType, SpawnConfig, Terrain,
        ZoneConstraintType, ZoneData, ZoneGrid, ZoneType,
    },
    rendering::zone_local_to_world,
};

pub struct ZoneFactory {
    pub zone_idx: usize,
    pub ozone: OverworldZone,
    pub terrain: Grid<Terrain>,
    pub entities: Grid<Vec<SpawnConfig>>,
    pub locked: Grid<bool>,
    pub roads: Vec<(usize, usize, usize)>, // x, y, width
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
        }
    }

    pub fn build(&mut self) -> ZoneData {
        self.apply_edge_constraints();
        self.apply_up_vertical_constraints();
        self.apply_vertical_constraints();
        self.connect_roads();
        self.apply_biome();

        self.to_zone_data()
    }

    pub fn to_zone_data(&self) -> ZoneData {
        ZoneData {
            zone_idx: self.zone_idx,
            terrain: self.terrain.clone(),
            entities: self.entities.clone(),
        }
    }

    pub fn apply_biome(&mut self) {
        let mut builder = Self::get_biome_builder(self.ozone.zone_type);
        builder.build(self);
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

    fn get_biome_builder(zone_type: ZoneType) -> Box<dyn BiomeBuilder> {
        match zone_type {
            ZoneType::OpenAir => Box::new(OpenAirBiomeBuilder),
            ZoneType::Forest => Box::new(ForestBiomeBuilder),
            ZoneType::Desert => Box::new(DesertBiomeBuilder),
            ZoneType::Cavern => Box::new(CavernBiomeBuilder),
        }
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
                self.roads.push((x, y, 1));
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
                self.roads.push((x, y, 1));
            }
        }
    }

    pub fn apply_edge_constraints(&mut self) {
        let mut c_road_pos = (0, 0);
        let mut c_road_width = 0;
        let mut on_road = false;

        // NORTH
        for (x, constraint) in self.ozone.constraints.north.0.iter().enumerate() {
            if let ZoneConstraintType::Road(_) = constraint {
                let y = 0;
                // self.terrain.set(x, y, Terrain::Dirt);
                // self.locked.set(x, y, true);

                if !on_road {
                    c_road_pos = (x, y);
                }

                on_road = true;
                c_road_width += 1;
            } else if on_road {
                self.roads.push((c_road_pos.0, c_road_pos.1, c_road_width));
                on_road = false;
                c_road_width = 0;
            }
        }

        on_road = false;
        c_road_width = 0;

        // SOUTH
        for (x, constraint) in self.ozone.constraints.south.0.iter().enumerate() {
            if let ZoneConstraintType::Road(_) = constraint {
                let y = ZONE_SIZE.1 - 1;
                // self.terrain.set(x, y, Terrain::Dirt);
                // self.locked.set(x, y, true);

                if !on_road {
                    c_road_pos = (x, y);
                }

                on_road = true;
                c_road_width += 1;
            } else if on_road {
                self.roads.push((c_road_pos.0, c_road_pos.1, c_road_width));
                on_road = false;
                c_road_width = 0;
            }
        }

        on_road = false;
        c_road_width = 0;

        // EAST
        for (y, constraint) in self.ozone.constraints.east.0.iter().enumerate() {
            if let ZoneConstraintType::Road(_) = constraint {
                let x = ZONE_SIZE.0 - 1;
                // self.terrain.set(x, y, Terrain::Dirt);
                // self.locked.set(x, y, true);

                if !on_road {
                    c_road_pos = (x, y);
                }

                on_road = true;
                c_road_width += 1;
            } else if on_road {
                self.roads.push((c_road_pos.0, c_road_pos.1, c_road_width));
                on_road = false;
                c_road_width = 0;
            }
        }

        on_road = false;
        c_road_width = 0;

        // WEST
        for (y, constraint) in self.ozone.constraints.west.0.iter().enumerate() {
            if let ZoneConstraintType::Road(_) = constraint {
                let x = 0;
                // self.terrain.set(x, y, Terrain::Dirt);
                // self.locked.set(x, y, true);

                if !on_road {
                    c_road_pos = (x, y);
                }

                on_road = true;
                c_road_width += 1;
            } else if on_road {
                self.roads.push((c_road_pos.0, c_road_pos.1, c_road_width));
                on_road = false;
                c_road_width = 0;
            }
        }
    }

    pub fn connect_roads(&mut self) {
        if self.roads.is_empty() {
            return;
        }

        let mut all_paths = Grid::init(ZONE_SIZE.0, ZONE_SIZE.1, false);
        let mut grouped_by_width: HashMap<usize, Vec<(usize, usize)>> = HashMap::new();

        trace!("=====ROADS=====");

        let mut biggest_width = 0;

        for r in self.roads.iter() {
            trace!("{},{} --> W={}", r.0, r.1, r.2);

            if r.2 > biggest_width {
                biggest_width = r.2;
            }

            if let Some(v) = grouped_by_width.get_mut(&r.2) {
                v.push((r.0, r.1));
            } else {
                grouped_by_width.insert(r.2, vec![(r.0, r.1)]);
            }
        }

        let Some(widest_path_endpoints) = grouped_by_width.get(&biggest_width) else {
            return;
        };

        for (p1_idx, p1) in widest_path_endpoints.iter().enumerate() {
            (p1_idx..widest_path_endpoints.len()).for_each(|p2_idx| {
                let p2 = widest_path_endpoints[p2_idx];

                if p1_idx == p2_idx {
                    return;
                }

                trace!(
                    "Connect wide path {}->{}. Width={}",
                    p1_idx, p2_idx, biggest_width
                );
                self.connect_paths(*p1, p2, biggest_width, Terrain::Dirt);
            });
        }
    }

    fn connect_paths(
        &mut self,
        a: (usize, usize),
        b: (usize, usize),
        width: usize,
        terrain: Terrain,
    ) {
        // TODO: these should probably be instance variables
        let grid_buffer = ZoneGrid::edge_gradient(8, 1.);
        let grid_bool = ZoneGrid::bool(self.zone_idx as u32);

        let result = astar(AStarSettings {
            start: [a.0, a.1],
            is_goal: |p| p[0] == b.0 && p[1] == b.1,
            cost: |_, [x, y]| {
                let r = grid_bool.get(x, y).unwrap();
                let e = remap(1. - grid_buffer.get(x, y).unwrap(), 0.25, 1.);

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

        // TODO: store in a grid, then stamp. avoid locked tiles.
        if result.is_success {
            for [s_x, s_y] in result.path {
                // need to "stamp" this with a width x width stamp
                for w1 in 0..width {
                    for w2 in 0..width {
                        let x = s_x + w1;
                        let y = s_y + w2;

                        if in_zone_bounds(x, y) {
                            self.terrain.set(x, y, terrain);
                            self.locked.set(x, y, true);
                        }
                    }
                }
            }
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
