use std::vec;

use bevy_ecs::{component::Component, entity::Entity, resource::Resource};
use serde::{Deserialize, Serialize};

use crate::{
    cfg::ZONE_SIZE,
    common::{Grid, HashGrid, Palette, Rand},
    domain::ZoneSaveData,
    rendering::{is_zone_oob, zone_idx, zone_xyz},
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
            Terrain::Dirt => 18,
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
    Path,
    StairDown,
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

            // path
            if x.is_multiple_of(4) {
                let r = rand.range_n(1, ZONE_SIZE.0 as i32 - 1) as usize;
                south[r] = ZoneConstraintType::Path;
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
                west[r] = ZoneConstraintType::Path;
            }
        }

        let mut down = vec![ZoneConstraintType::None; ZONE_SIZE.0];

        if z < crate::cfg::MAP_SIZE.2 - 1 {
            let stair_x = rand.range_n(0, ZONE_SIZE.0 as i32) as usize;
            down[stair_x] = ZoneConstraintType::StairDown;
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
