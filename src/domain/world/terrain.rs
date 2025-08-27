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
}

impl Terrain {
    pub fn tile(&self) -> usize {
        match self {
            Terrain::Grass => 1,
            Terrain::Dirt => 129,
            Terrain::Sand => 33,
            Terrain::River => 34,
            Terrain::OpenAir => 0,
        }
    }

    pub fn tiles(&self) -> Vec<usize> {
        match self {
            Terrain::Grass => vec![0, 1, 2, 3],
            Terrain::Dirt => vec![16, 17, 18, 19],
            Terrain::Sand => vec![32, 33, 34, 35],
            Terrain::River => vec![31, 32, 33, 34],
            Terrain::OpenAir => vec![0],
        }
    }

    pub fn colors(&self) -> (Option<u32>, Option<u32>) {
        match self {
            Terrain::Grass => (None, Some(Palette::DarkCyan.into())),
            Terrain::Dirt => (None, Some(Palette::Brown.into())),
            Terrain::Sand => (None, Some(Palette::Red.into())),
            Terrain::River => (Some(Palette::DarkBlue.into()), Some(Palette::Blue.into())),
            Terrain::OpenAir => (None, None),
        }
    }
}

#[derive(Resource)]
pub struct TerrainNoise {
    pub sand: Perlin,
    pub grass: Perlin,
    pub dirt: Perlin,
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
            sand: Perlin::new(seed + 120, 0.1, 1, 1.5),
            grass: Perlin::new(seed + 200, 0.08, 1, 1.2),
            dirt: Perlin::new(seed + 300, 0.12, 1, 1.8),
        }
    }

    pub fn sand(&mut self, pos: (usize, usize)) -> Style {
        let v = self.sand.get(pos.0 as f32, pos.1 as f32 / 2.);
        let sand_tiles = Terrain::Sand.tiles();

        let tile_idx = (v * sand_tiles.len() as f32) as usize;
        let tile_idx = tile_idx.min(sand_tiles.len() - 1);

        Style {
            idx: sand_tiles[tile_idx],
            fg1: Palette::Red.into(),
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
            fg1: Palette::DarkCyan.into(),
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
            Terrain::River => self.sand(pos), // Using sand for river for now
            Terrain::Sand => self.sand(pos),
        }
    }
}
