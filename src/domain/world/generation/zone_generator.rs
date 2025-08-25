use bevy_ecs::world::World;
use macroquad::prelude::trace;

use crate::{
    common::Grid,
    domain::{
        CavernZoneBuilder, DesertZoneBuilder, ForestZoneBuilder, OpenAirZoneBuilder, Overworld,
        OverworldZone, SpawnConfig, Terrain, ZoneType,
    },
};

pub trait ZoneBuilder {
    fn build(&mut self, ozone: OverworldZone) -> ZoneData;
}

pub struct ZoneData {
    pub zone_idx: usize,
    pub terrain: Grid<Terrain>,
    pub entities: Grid<Vec<SpawnConfig>>,
}

pub struct ZoneGenerator;

impl ZoneGenerator {
    pub fn generate_zone(world: &mut World, zone_idx: usize) -> ZoneData {
        let mut overworld = world.get_resource_mut::<Overworld>().unwrap();
        let ozone = overworld.get_overworld_zone(zone_idx);

        trace!("Generating zone... {}, {}", zone_idx, ozone.zone_type);

        let mut builder = Self::get_builder(ozone.zone_type);

        builder.build(ozone)
    }

    fn get_builder(zone_type: ZoneType) -> Box<dyn ZoneBuilder> {
        match zone_type {
            ZoneType::OpenAir => Box::new(OpenAirZoneBuilder),
            ZoneType::Forest => Box::new(ForestZoneBuilder),
            ZoneType::Desert => Box::new(DesertZoneBuilder),
            ZoneType::Cavern => Box::new(CavernZoneBuilder),
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
    Road,
    StairDown,
    RockWall,
}
