use crate::{
    cfg::ZONE_SIZE,
    common::Grid,
    domain::{PrefabId, SpawnConfig, Terrain, ZoneConstraintType, OverworldZone},
    rendering::zone_local_to_world,
};

use super::road_builder::{RoadBuilder, RoadCategory, RoadConnection};

#[derive(Clone, Copy)]
pub enum EdgeDirection {
    North,
    South,
    East,
    West,
}

impl EdgeDirection {
    fn get_position(&self, index: usize) -> (usize, usize) {
        match self {
            EdgeDirection::North => (index, 0),
            EdgeDirection::South => (index, ZONE_SIZE.1 - 1),
            EdgeDirection::East => (ZONE_SIZE.0 - 1, index),
            EdgeDirection::West => (0, index),
        }
    }

    fn get_road_category(&self) -> RoadCategory {
        match self {
            EdgeDirection::North => RoadCategory::North,
            EdgeDirection::South => RoadCategory::South,
            EdgeDirection::East => RoadCategory::East,
            EdgeDirection::West => RoadCategory::West,
        }
    }

    fn get_constraints<'a>(&self, ozone: &'a OverworldZone) -> &'a Vec<ZoneConstraintType> {
        match self {
            EdgeDirection::North => &ozone.constraints.north.0,
            EdgeDirection::South => &ozone.constraints.south.0,
            EdgeDirection::East => &ozone.constraints.east.0,
            EdgeDirection::West => &ozone.constraints.west.0,
        }
    }
}

pub struct ConstraintHandler;

impl ConstraintHandler {
    pub fn apply_all_constraints(
        ozone: &OverworldZone,
        terrain: &mut Grid<Terrain>,
        entities: &mut Grid<Vec<SpawnConfig>>,
        locked: &mut Grid<bool>,
        road_builder: &mut RoadBuilder,
        road_terrain: Terrain,
    ) {
        Self::apply_edge_constraints(ozone, terrain, entities, locked, road_builder, road_terrain);
        Self::apply_vertical_constraints(ozone, entities, locked, road_builder);
    }

    fn apply_edge_constraints(
        ozone: &OverworldZone,
        terrain: &mut Grid<Terrain>,
        entities: &mut Grid<Vec<SpawnConfig>>,
        locked: &mut Grid<bool>,
        road_builder: &mut RoadBuilder,
        road_terrain: Terrain,
    ) {
        let directions = [
            EdgeDirection::North,
            EdgeDirection::South,
            EdgeDirection::East,
            EdgeDirection::West,
        ];

        for direction in &directions {
            Self::apply_edge_constraints_for_direction(
                direction,
                ozone,
                terrain,
                entities,
                locked,
                road_builder,
                road_terrain,
            );
        }
    }

    fn apply_edge_constraints_for_direction(
        direction: &EdgeDirection,
        ozone: &OverworldZone,
        terrain: &mut Grid<Terrain>,
        entities: &mut Grid<Vec<SpawnConfig>>,
        locked: &mut Grid<bool>,
        road_builder: &mut RoadBuilder,
        road_terrain: Terrain,
    ) {
        let constraints = direction.get_constraints(ozone);
        let mut current_road: Option<(usize, usize)> = None;
        let mut road_width = 0;

        for (index, constraint) in constraints.iter().enumerate() {
            let (x, y) = direction.get_position(index);

            match constraint {
                ZoneConstraintType::Road(_) => {
                    terrain.set(x, y, road_terrain);
                    locked.set(x, y, true);

                    if current_road.is_none() {
                        current_road = Some((x, y));
                    }
                    road_width += 1;
                }
                ZoneConstraintType::Rock => {
                    Self::place_boulder(ozone.zone_idx, x, y, entities, locked);
                    Self::finish_current_road(&mut current_road, road_width, direction, road_builder);
                    road_width = 0;
                }
                _ => {
                    Self::finish_current_road(&mut current_road, road_width, direction, road_builder);
                    road_width = 0;
                }
            }
        }

        // Finish any remaining road at the end
        Self::finish_current_road(&mut current_road, road_width, direction, road_builder);
    }

    fn apply_vertical_constraints(
        ozone: &OverworldZone,
        entities: &mut Grid<Vec<SpawnConfig>>,
        locked: &mut Grid<bool>,
        road_builder: &mut RoadBuilder,
    ) {
        // Handle stairs going up (from down constraints)
        for constraint in &ozone.constraints.down.0 {
            if constraint.constraint == ZoneConstraintType::StairDown {
                let (x, y) = constraint.position;
                let world_pos = zone_local_to_world(ozone.zone_idx, x, y);
                let stair_config = SpawnConfig::new(PrefabId::StairUp, world_pos);

                if let Some(entities_at_pos) = entities.get_mut(x, y) {
                    entities_at_pos.push(stair_config);
                }
                locked.set(x, y, true);
                road_builder.add_connection(RoadConnection {
                    category: RoadCategory::Stairs,
                    pos: (x, y),
                    width: 1,
                });
            }
        }

        // Handle stairs going down (from up constraints)
        for constraint in &ozone.constraints.up.0 {
            if constraint.constraint == ZoneConstraintType::StairDown {
                let (x, y) = constraint.position;
                let world_pos = zone_local_to_world(ozone.zone_idx, x, y);
                let stair_config = SpawnConfig::new(PrefabId::StairDown, world_pos);

                if let Some(entities_at_pos) = entities.get_mut(x, y) {
                    entities_at_pos.push(stair_config);
                }
                locked.set(x, y, true);
                road_builder.add_connection(RoadConnection {
                    category: RoadCategory::Stairs,
                    pos: (x, y),
                    width: 1,
                });
            }
        }
    }

    fn place_boulder(
        zone_idx: usize,
        x: usize,
        y: usize,
        entities: &mut Grid<Vec<SpawnConfig>>,
        locked: &mut Grid<bool>,
    ) {
        let world_pos = zone_local_to_world(zone_idx, x, y);
        let boulder_config = SpawnConfig::new(PrefabId::Boulder, world_pos);
        
        if let Some(entities_at_pos) = entities.get_mut(x, y) {
            entities_at_pos.push(boulder_config);
        }
        locked.set(x, y, true);
    }

    fn finish_current_road(
        current_road: &mut Option<(usize, usize)>,
        road_width: usize,
        direction: &EdgeDirection,
        road_builder: &mut RoadBuilder,
    ) {
        if let Some(road_pos) = current_road.take() {
            if road_width > 0 {
                road_builder.add_connection(RoadConnection {
                    category: direction.get_road_category(),
                    pos: road_pos,
                    width: road_width,
                });
            }
        }
    }
}