use std::fmt::Display;

use crate::domain::{
    CavernBiomeBuilder, DesertBiomeBuilder, ForestBiomeBuilder, OpenAirBiomeBuilder, ZoneFactory,
};

pub trait BiomeBuilder {
    fn build(zone: &mut ZoneFactory);
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BiomeType {
    OpenAir,
    Forest,
    Desert,
    Cavern,
}

impl BiomeType {
    pub fn apply_to_zone(&self, zone_factory: &mut ZoneFactory) {
        match self {
            BiomeType::OpenAir => OpenAirBiomeBuilder::build(zone_factory),
            BiomeType::Forest => ForestBiomeBuilder::build(zone_factory),
            BiomeType::Desert => DesertBiomeBuilder::build(zone_factory),
            BiomeType::Cavern => CavernBiomeBuilder::build(zone_factory),
        }
    }
}

impl Display for BiomeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BiomeType::OpenAir => write!(f, "OpenAir"),
            BiomeType::Forest => write!(f, "Forest"),
            BiomeType::Desert => write!(f, "Desert"),
            BiomeType::Cavern => write!(f, "Cavern"),
        }
    }
}
