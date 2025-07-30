use bevy_ecs::{component::Component, entity::Entity, resource::Resource};
use serde::{Deserialize, Serialize};

use crate::{common::{Grid, Palette}, domain::ZoneSaveData};

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
            Terrain::Dirt => 16,
            Terrain::River => 34,
        }
    }

    pub fn colors(&self) -> (Option<u32>, Option<u32>) {
        match self {
            Terrain::Grass => (None, Some(Palette::Green.into())),
            Terrain::Dirt => (None, Some(Palette::Brown.into())),
            Terrain::River => (Some(Palette::Blue.into()), Some(Palette::Cyan.into())),
        }
    }
}

#[derive(Component)]
pub struct Zone {
    pub idx: usize,
    pub tiles: Grid<Entity>,
    pub terrain: Grid<Terrain>,
}

impl Zone {
    pub fn new(idx: usize, terrain: Grid<Terrain>, tiles: Grid<Entity>) -> Self
    {
        Self { idx, tiles, terrain }
    }

    pub fn to_save(&self) -> ZoneSaveData {
        ZoneSaveData { idx: self.idx, terrain: self.terrain.clone() }
    }
}
