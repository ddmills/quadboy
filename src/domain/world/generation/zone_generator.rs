use bevy_ecs::world::World;
use macroquad::prelude::trace;

use crate::{
    common::Grid,
    domain::{
        CavernZoneBuilder, DesertZoneBuilder, ForestZoneBuilder, OpenAirZoneBuilder, Overworld,
        OverworldZone, PrefabId, RoadType, SpawnConfig, Terrain, ZoneType, ZoneConstraintType,
        ZoneVerticalConstraints, ZoneEdgeConstraints,
    },
    rendering::zone_local_to_world,
    cfg::ZONE_SIZE,
};

pub trait ZoneBuilder {
    fn build(&mut self, ozone: OverworldZone) -> ZoneData;
}

pub struct ZoneData {
    pub zone_idx: usize,
    pub terrain: Grid<Terrain>,
    pub entities: Grid<Vec<SpawnConfig>>,
}

impl ZoneData {
    pub fn apply_vertical_constraints(&mut self, vertical_constraints: &ZoneVerticalConstraints) {
        for constraint in &vertical_constraints.0 {
            if constraint.constraint == ZoneConstraintType::StairDown {
                let (x, y) = constraint.position;
                let world_pos = zone_local_to_world(self.zone_idx, x, y);
                let stair_config = SpawnConfig::new(PrefabId::StairDown, world_pos);
                
                if let Some(entities_at_pos) = self.entities.get_mut(x, y) {
                    entities_at_pos.push(stair_config);
                }
            }
        }
    }
    
    pub fn apply_up_vertical_constraints(&mut self, vertical_constraints: &ZoneVerticalConstraints) {
        for constraint in &vertical_constraints.0 {
            if constraint.constraint == ZoneConstraintType::StairDown {
                let (x, y) = constraint.position;
                let world_pos = zone_local_to_world(self.zone_idx, x, y);
                let stair_config = SpawnConfig::new(PrefabId::StairUp, world_pos);
                
                if let Some(entities_at_pos) = self.entities.get_mut(x, y) {
                    entities_at_pos.push(stair_config);
                }
            }
        }
    }

    pub fn apply_edge_constraints(&mut self, 
        north: &ZoneEdgeConstraints,
        south: &ZoneEdgeConstraints, 
        east: &ZoneEdgeConstraints,
        west: &ZoneEdgeConstraints) {
        
        
        // Apply north edge constraints (north = y = 0)
        for (i, constraint) in north.0.iter().enumerate() {
            if let ZoneConstraintType::Road(_) = constraint {
                let y = 0;
                self.terrain.set(i, y, Terrain::Dirt);
            }
        }
        
        // Apply south edge constraints (south = y = ZONE_SIZE.1 - 1)
        for (i, constraint) in south.0.iter().enumerate() {
            if let ZoneConstraintType::Road(_) = constraint {
                let y = ZONE_SIZE.1 - 1;
                self.terrain.set(i, y, Terrain::Dirt);
            }
        }
        
        // Apply east edge constraints
        for (i, constraint) in east.0.iter().enumerate() {
            if let ZoneConstraintType::Road(_) = constraint {
                let x = ZONE_SIZE.0 - 1;
                self.terrain.set(x, i, Terrain::Dirt);
            }
        }
        
        // Apply west edge constraints
        for (i, constraint) in west.0.iter().enumerate() {
            if let ZoneConstraintType::Road(_) = constraint {
                let x = 0;
                self.terrain.set(x, i, Terrain::Dirt);
            }
        }
    }
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
