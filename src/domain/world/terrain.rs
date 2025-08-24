use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

use crate::common::Palette;

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
