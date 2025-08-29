use bevy_ecs::{component::Component, resource::Resource};
use serde::{Deserialize, Serialize};

use crate::common::{Palette, Perlin};

#[repr(u8)]
#[derive(Clone, Hash, Copy, Default, Deserialize, Serialize, PartialEq, Eq, Debug, Component)]
pub enum Terrain {
    OpenAir = 1,
    #[default]
    Grass = 2,
    Dirt = 3,
    River = 4,
    Sand = 5,
    Shallows = 6,
}

impl Terrain {
    pub fn tiles(&self) -> Vec<usize> {
        match self {
            Terrain::Grass => vec![0, 1, 2, 3],
            Terrain::Dirt => vec![48, 49],
            Terrain::Sand => vec![32, 33, 34, 35],
            Terrain::River => vec![34],
            Terrain::Shallows => vec![32],
            Terrain::OpenAir => vec![0],
        }
    }

    pub fn colors(&self) -> (Option<u32>, Option<u32>) {
        match self {
            Terrain::Grass => (None, Some(Palette::DarkCyan.into())),
            Terrain::Dirt => (None, Some(Palette::Brown.into())),
            Terrain::Sand => (None, Some(Palette::Red.into())),
            Terrain::River => (Some(Palette::DarkBlue.into()), Some(Palette::Blue.into())),
            Terrain::Shallows => (Some(Palette::DarkBlue.into()), Some(Palette::Blue.into())),
            Terrain::OpenAir => (None, None),
        }
    }
}

#[derive(Resource)]
pub struct TerrainNoise {
    pub sand: Perlin,
    pub grass: Perlin,
    pub dirt: Perlin,
    pub river: Perlin,
}

pub struct Style {
    pub idx: usize,
    pub fg1: Option<Palette>,
    pub fg2: Option<Palette>,
    pub bg: Option<Palette>,
    pub outline: Option<Palette>,
}

impl TerrainNoise {
    pub fn new(seed: u32) -> Self {
        Self {
            sand: Perlin::new(seed + 120, 0.2, 2, 1.5),
            grass: Perlin::new(seed + 200, 0.08, 1, 1.2),
            dirt: Perlin::new(seed + 300, 0.12, 1, 1.8),
            river: Perlin::new(seed + 300, 0.12, 1, 1.8),
        }
    }

    pub fn sand(&mut self, pos: (usize, usize)) -> Style {
        let v = self.sand.get(pos.0 as f32, pos.1 as f32 / 2.);
        let sand_tiles = Terrain::Sand.tiles();

        let tile_idx = (v * sand_tiles.len() as f32) as usize;
        let tile_idx = tile_idx.min(sand_tiles.len() - 2);

        Style {
            idx: sand_tiles[tile_idx],
            fg1: Palette::Brown.into(),
            fg2: None,
            bg: None,
            outline: None,
        }
    }

    pub fn grass(&mut self, pos: (usize, usize)) -> Style {
        let v = self.grass.get(pos.0 as f32, pos.1 as f32);
        let grass_tiles = Terrain::Grass.tiles();

        let tile_idx = (v * grass_tiles.len() as f32) as usize;
        let tile_idx = tile_idx.min(grass_tiles.len() - 1);

        Style {
            idx: grass_tiles[tile_idx],
            fg1: Palette::DarkGreen.into(),
            fg2: None,
            bg: None,
            outline: None,
        }
    }

    pub fn dirt(&mut self, pos: (usize, usize)) -> Style {
        let v = self.dirt.get(pos.0 as f32, pos.1 as f32);
        let dirt_tiles = Terrain::Dirt.tiles();

        let tile_idx = (v * dirt_tiles.len() as f32) as usize;
        let tile_idx = tile_idx.min(dirt_tiles.len() - 1);

        Style {
            idx: dirt_tiles[tile_idx],
            fg1: Palette::Brown.into(),
            fg2: None,
            bg: None,
            outline: None,
        }
    }

    pub fn river(&mut self, pos: (usize, usize)) -> Style {
        let v = self.river.get(pos.0 as f32, pos.1 as f32);
        let river_tiles = Terrain::River.tiles();

        let tile_idx = (v * river_tiles.len() as f32) as usize;
        let tile_idx = tile_idx.min(river_tiles.len() - 1);

        Style {
            idx: river_tiles[tile_idx],
            fg1: Palette::Blue.into(),
            fg2: None,
            bg: Palette::DarkBlue.into(),
            outline: None,
        }
    }

    pub fn shallows(&mut self, pos: (usize, usize)) -> Style {
        let v = self.river.get(pos.0 as f32, pos.1 as f32);
        let shallows_tiles = Terrain::Shallows.tiles();

        let tile_idx = (v * shallows_tiles.len() as f32) as usize;
        let tile_idx = tile_idx.min(shallows_tiles.len() - 1);

        Style {
            idx: shallows_tiles[tile_idx],
            fg1: Palette::Blue.into(),
            fg2: None,
            bg: Palette::DarkBlue.into(),
            outline: None,
        }
    }

    pub fn style(&mut self, terrain: Terrain, pos: (usize, usize)) -> Style {
        match terrain {
            Terrain::OpenAir => Style {
                idx: 0,
                fg1: None,
                fg2: None,
                bg: None,
                outline: None,
            },
            Terrain::Grass => self.grass(pos),
            Terrain::Dirt => self.dirt(pos),
            Terrain::River => self.river(pos),
            Terrain::Sand => self.sand(pos),
            Terrain::Shallows => self.shallows(pos),
        }
    }
}
