use bevy_ecs::{component::Component, resource::Resource};
use serde::{Deserialize, Serialize};

use crate::common::{Palette, Perlin};

#[repr(u8)]
#[derive(Clone, Hash, Copy, Default, Deserialize, Serialize, PartialEq, Eq, Debug, Component)]
pub enum Terrain {
    OpenAir = 1,
    #[default]
    Grass = 2,
    DyingGrass = 3,
    Gravel = 4,
    Dirt = 5,
    River = 6,
    Sand = 7,
    Shallows = 8,
    Swamp = 9,
}

impl Terrain {
    pub fn tiles(&self) -> Vec<usize> {
        match self {
            Terrain::Grass => vec![0, 1, 2],
            Terrain::DyingGrass => vec![16, 17, 0, 1, 2],
            Terrain::Swamp => vec![48, 49, 50, 51],
            Terrain::Gravel => vec![16, 17, 0, 1],
            Terrain::Dirt => vec![4, 5],
            // Terrain::Dirt => vec![48, 49],
            // Terrain::Sand => vec![4, 5],
            Terrain::Sand => vec![16, 17, 18],
            Terrain::River => vec![34],
            Terrain::Shallows => vec![5],
            Terrain::OpenAir => vec![0],
        }
    }

    pub fn label_formatted(&self) -> String {
        match self {
            Terrain::Grass => "{G|Grass}",
            Terrain::DyingGrass => "{y|Dying Grass}",
            Terrain::Gravel => "{U|Gravel}",
            Terrain::Dirt => "{X|Dirt}",
            Terrain::Sand => "{Y|Sand}",
            Terrain::River => "{B|River}",
            Terrain::Shallows => "{B|Shallows}",
            Terrain::OpenAir => "{B|Open Air}",
            Terrain::Swamp => "{P|Swamp}",
        }
        .to_owned()
    }
}

#[derive(Resource)]
pub struct TerrainNoise {
    pub sand: Perlin,
    pub grass: Perlin,
    pub dying_grass: Perlin,
    pub gravel: Perlin,
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
            dying_grass: Perlin::new(seed + 210, 0.49, 4, 1.3),
            gravel: Perlin::new(seed + 220, 0.2, 3, 1.4),
            dirt: Perlin::new(seed + 300, 0.12, 1, 1.8),
            river: Perlin::new(seed + 300, 0.12, 1, 1.8),
        }
    }

    pub fn sand(&mut self, pos: (usize, usize)) -> Style {
        let v = self.sand.get(pos.0 as f32, pos.1 as f32 / 2.);
        let sand_tiles = Terrain::Sand.tiles();

        let tile_idx = (v * sand_tiles.len() as f32) as usize;
        let tile_idx = tile_idx.min(sand_tiles.len() - 1);

        Style {
            idx: sand_tiles[tile_idx],
            fg1: Palette::DarkRed.into(),
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

    pub fn dying_grass(&mut self, pos: (usize, usize)) -> Style {
        let v = self.dying_grass.get(pos.0 as f32, pos.1 as f32);
        let dying_grass_tiles = Terrain::DyingGrass.tiles();

        let tile_idx = (v * dying_grass_tiles.len() as f32) as usize;
        let tile_idx = tile_idx.min(dying_grass_tiles.len() - 1);

        Style {
            idx: dying_grass_tiles[tile_idx],
            fg1: Palette::DarkOrange.into(),
            fg2: None,
            bg: None,
            outline: None,
        }
    }

    pub fn gravel(&mut self, pos: (usize, usize)) -> Style {
        let v = self.gravel.get(pos.0 as f32, pos.1 as f32);
        let gravel_tiles = Terrain::Gravel.tiles();

        let tile_idx = (v * gravel_tiles.len() as f32) as usize;
        let tile_idx = tile_idx.min(gravel_tiles.len() - 1);

        Style {
            idx: gravel_tiles[tile_idx],
            fg1: Palette::Gray.into(),
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
        let shallows_tiles = Terrain::River.tiles();

        let tile_idx = (v * shallows_tiles.len() as f32) as usize;
        let tile_idx = tile_idx.min(shallows_tiles.len() - 1);

        Style {
            idx: shallows_tiles[tile_idx],
            fg1: Palette::DarkBlue.into(),
            fg2: None,
            bg: Palette::Blue.into(),
            outline: None,
        }
    }

    pub fn swamp(&mut self, pos: (usize, usize)) -> Style {
        let v = self.dying_grass.get(pos.0 as f32, pos.1 as f32);
        let swamp_tiles = Terrain::Swamp.tiles();

        let tile_idx = (v * swamp_tiles.len() as f32) as usize;
        let tile_idx = tile_idx.min(swamp_tiles.len() - 1);

        Style {
            idx: swamp_tiles[tile_idx],
            fg1: Palette::DarkPurple.into(),
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
            Terrain::DyingGrass => self.dying_grass(pos),
            Terrain::Gravel => self.gravel(pos),
            Terrain::Dirt => self.dirt(pos),
            Terrain::River => self.river(pos),
            Terrain::Sand => self.sand(pos),
            Terrain::Shallows => self.shallows(pos),
            Terrain::Swamp => self.swamp(pos),
        }
    }
}
