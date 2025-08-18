use bevy_ecs::{component::Component, entity::Entity, resource::Resource};
use serde::{Deserialize, Serialize};

use crate::{
    cfg::ZONE_SIZE,
    common::{Grid, HashGrid, Palette},
    domain::ZoneSaveData,
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
            Terrain::Grass => (None, Some(Palette::DarkBrown.into())),
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
