use std::vec;

use bevy_ecs::{component::Component, entity::Entity, resource::Resource, system::Query};
use serde::{Deserialize, Serialize};

use crate::{
    cfg::{CARDINALS_OFFSET, ZONE_SIZE},
    common::{Grid, HashGrid, Palette, Perlin, Rand},
    domain::ZoneSaveData,
    rendering::{is_zone_oob, world_to_zone_idx, world_to_zone_local, zone_idx, zone_xyz},
};

#[derive(Resource, Default)]
pub struct Zones {
    pub active: Vec<usize>,
    pub player: usize,
}

#[repr(u8)]
#[derive(Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq)]
pub enum Terrain {
    #[default]
    Grass = 1,
    Dirt = 2,
    River = 3,
}

impl Terrain {
    pub fn tile(&self) -> usize {
        match self {
            Terrain::Grass => 1,
            Terrain::Dirt => 129,
            Terrain::River => 34,
        }
    }

    pub fn colors(&self) -> (Option<u32>, Option<u32>) {
        match self {
            Terrain::Grass => (None, Some(Palette::DarkCyan.into())),
            Terrain::Dirt => (None, Some(Palette::Brown.into())),
            Terrain::River => (Some(Palette::DarkBlue.into()), Some(Palette::Blue.into())),
        }
    }
}

#[derive(Component)]
pub struct Zone {
    pub idx: usize,
    pub terrain: Grid<Terrain>,
    pub entities: HashGrid<Entity>,
}

impl Zone {
    pub fn new(idx: usize, terrain: Grid<Terrain>) -> Self {
        Self {
            idx,
            terrain,
            entities: HashGrid::init(ZONE_SIZE.0, ZONE_SIZE.1),
        }
    }

    pub fn to_save(&self) -> ZoneSaveData {
        ZoneSaveData {
            idx: self.idx,
            terrain: self.terrain.clone(),
            entities: vec![],
        }
    }

    pub fn get_at(world_pos: (usize, usize, usize), q_zones: &Query<&Zone>) -> Vec<Entity> {
        let (x, y, z) = world_pos;
        let zone_idx = world_to_zone_idx(x, y, z);

        let Some(zone) = q_zones.iter().find(|z| z.idx == zone_idx) else {
            return vec![];
        };

        let local = world_to_zone_local(x, y);

        let Some(entities) = zone.entities.get(local.0, local.1) else {
            return vec![];
        };

        entities.to_vec()
    }

    pub fn get_neighbors(
        world_pos: (usize, usize, usize),
        q_zones: &Query<&Zone>,
    ) -> Vec<Vec<Entity>> {
        let (x, y, z) = world_pos;

        let mut neighbors = Vec::with_capacity(4);

        for (dx, dy) in CARDINALS_OFFSET.iter() {
            let neighbor_x_i32 = x as i32 + dx;
            let neighbor_y_i32 = y as i32 + dy;

            if neighbor_x_i32 < 0 || neighbor_y_i32 < 0 {
                neighbors.push(vec![]);
                continue;
            }

            let neighbor_x = neighbor_x_i32 as usize;
            let neighbor_y = neighbor_y_i32 as usize;

            let entities = Self::get_at((neighbor_x, neighbor_y, z), q_zones);
            neighbors.push(entities);
        }

        neighbors
    }
}

pub struct ZoneContinuity {
    pub south: Vec<ZoneConstraintType>,
    pub west: Vec<ZoneConstraintType>,
    pub down: Vec<ZoneConstraintType>,
}

impl ZoneContinuity {
    pub fn empty() -> Self {
        Self {
            south: vec![],
            west: vec![],
            down: vec![],
        }
    }
}

pub struct ZoneConstraints {
    pub idx: usize,
    pub south: Vec<ZoneConstraintType>,
    pub west: Vec<ZoneConstraintType>,
    pub east: Vec<ZoneConstraintType>,
    pub north: Vec<ZoneConstraintType>,
    pub up: Vec<ZoneConstraintType>,
    pub down: Vec<ZoneConstraintType>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ZoneConstraintType {
    None,
    River,
    Footpath,
    StairDown,
    RockWall,
}

#[derive(Resource, Default)]
pub struct Map;

impl Map {
    fn get_continuity(&self, x: usize, y: usize, z: usize) -> ZoneContinuity {
        if is_zone_oob(x, y, z) {
            return ZoneContinuity::empty();
        }

        let idx = zone_idx(x, y, z);
        let mut rand = Rand::seed(idx as u64);

        let mut south = [ZoneConstraintType::None; ZONE_SIZE.0];
        let mut west = [ZoneConstraintType::None; ZONE_SIZE.1];

        if y > 0 {
            // river
            if x.is_multiple_of(3) {
                let r = rand.range_n(1, ZONE_SIZE.0 as i32 - 1) as usize;
                south[r] = ZoneConstraintType::River;
            }

            // footpath
            if x.is_multiple_of(4) {
                let r = rand.range_n(1, ZONE_SIZE.0 as i32 - 1) as usize;
                south[r] = ZoneConstraintType::Footpath;
            }
        }

        if x > 0 {
            // river
            if y.is_multiple_of(2) {
                let r = rand.range_n(1, ZONE_SIZE.1 as i32 - 1) as usize;
                west[r] = ZoneConstraintType::River;
            }

            // footpaths
            if y.is_multiple_of(2) {
                let r = rand.range_n(1, ZONE_SIZE.1 as i32 - 1) as usize;
                west[r] = ZoneConstraintType::Footpath;
            }
        }

        let mut down = vec![ZoneConstraintType::None; ZONE_SIZE.0];

        if z < crate::cfg::MAP_SIZE.2 - 1 {
            let stair_x = rand.range_n(0, ZONE_SIZE.0 as i32) as usize;
            down[stair_x] = ZoneConstraintType::StairDown;
        }

        let mut perlin = Perlin::new(idx as u32, 0.15, 2, 2.0); // Increased frequency, reduced octaves

        if y > 0 {
            let wall_threshold = 0.65; // Increased threshold for smaller clusters
            let max_wall_length = 8; // Maximum consecutive wall tiles
            let mut in_wall_section = false;
            let mut current_wall_length = 0;

            for i in 0..ZONE_SIZE.0 {
                let noise_value = perlin.get(x as f32 + i as f32 * 0.1, y as f32);

                if noise_value > wall_threshold && current_wall_length < max_wall_length {
                    if !in_wall_section {
                        in_wall_section = true;
                    }

                    if south[i] == ZoneConstraintType::None {
                        south[i] = ZoneConstraintType::RockWall;
                        current_wall_length += 1;
                    }
                } else {
                    in_wall_section = false;
                    current_wall_length = 0;
                }
            }
        }

        if x > 0 {
            let wall_threshold = 0.65;
            let max_wall_length = 8;
            let mut in_wall_section = false;
            let mut current_wall_length = 0;

            for i in 0..ZONE_SIZE.1 {
                let noise_value = perlin.get(x as f32, y as f32 + i as f32 * 0.1);

                if noise_value > wall_threshold && current_wall_length < max_wall_length {
                    if !in_wall_section {
                        in_wall_section = true;
                    }

                    if west[i] == ZoneConstraintType::None {
                        west[i] = ZoneConstraintType::RockWall;
                        current_wall_length += 1;
                    }
                } else {
                    in_wall_section = false;
                    current_wall_length = 0;
                }
            }
        }

        ZoneContinuity {
            south: south.to_vec(),
            west: west.to_vec(),
            down: down.to_vec(),
        }
    }

    pub fn get_zone_constraints(&self, idx: usize) -> ZoneConstraints {
        let (x, y, z) = zone_xyz(idx);

        let own = self.get_continuity(x, y, z);
        let east = self.get_continuity(x + 1, y, z);
        let north = self.get_continuity(x, y + 1, z);
        let up = if z > 0 {
            self.get_continuity(x, y, z - 1)
        } else {
            ZoneContinuity::empty()
        };

        ZoneConstraints {
            idx,
            north: north.south,
            south: own.south,
            east: east.west,
            west: own.west,
            up: up.down,
            down: own.down,
        }
    }
}
