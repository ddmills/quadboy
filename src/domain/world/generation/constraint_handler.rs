use crate::{
    cfg::ZONE_SIZE,
    common::Grid,
    domain::{OverworldZone, Prefab, PrefabId, RiverType, Terrain, ZoneConstraintType},
    rendering::zone_local_to_world,
};

use super::{
    river_builder::{RiverBuilder, RiverCategory, RiverConnection},
    road_builder::{RoadBuilder, RoadCategory, RoadConnection},
};

#[derive(Clone, Copy, Debug, PartialEq)]
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
        entities: &mut Grid<Vec<Prefab>>,
        locked: &mut Grid<bool>,
        road_builder: &mut RoadBuilder,
        river_builder: &mut RiverBuilder,
        road_terrain: Terrain,
    ) {
        // Apply edge constraints for roads and rivers
        Self::apply_road_constraints(ozone, terrain, locked, road_builder, road_terrain);
        Self::apply_river_constraints(ozone, terrain, locked, river_builder);
        // Apply other constraints
        Self::apply_rock_constraints(ozone, entities, locked, road_builder);
        Self::apply_foliage_constraints(ozone, entities, locked);
        Self::apply_vertical_constraints(ozone, entities, locked, road_builder);
    }

    fn apply_vertical_constraints(
        ozone: &OverworldZone,
        entities: &mut Grid<Vec<Prefab>>,
        locked: &mut Grid<bool>,
        road_builder: &mut RoadBuilder,
    ) {
        // Handle stairs going up (from down constraints)
        for constraint in &ozone.constraints.down.0 {
            if constraint.constraint == ZoneConstraintType::StairDown {
                let (x, y) = constraint.position;
                let world_pos = zone_local_to_world(ozone.zone_idx, x, y);
                let stair_config = Prefab::new(PrefabId::StairUp, world_pos);

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
                let stair_config = Prefab::new(PrefabId::StairDown, world_pos);

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
        entities: &mut Grid<Vec<Prefab>>,
        locked: &mut Grid<bool>,
    ) {
        let world_pos = zone_local_to_world(zone_idx, x, y);
        let boulder_config = Prefab::new(PrefabId::Boulder, world_pos);

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
        if let Some(road_pos) = current_road.take()
            && road_width > 0
        {
            road_builder.add_connection(RoadConnection {
                category: direction.get_road_category(),
                pos: road_pos,
                width: road_width,
            });
        }
    }

    fn apply_road_constraints(
        ozone: &OverworldZone,
        terrain: &mut Grid<Terrain>,
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
                    _ => {
                        Self::finish_current_road(
                            &mut current_road,
                            road_width,
                            direction,
                            road_builder,
                        );
                        road_width = 0;
                    }
                }
            }

            Self::finish_current_road(&mut current_road, road_width, direction, road_builder);
        }
    }

    fn apply_river_constraints(
        ozone: &OverworldZone,
        terrain: &mut Grid<Terrain>,
        locked: &mut Grid<bool>,
        river_builder: &mut RiverBuilder,
    ) {
        let directions = [
            EdgeDirection::North,
            EdgeDirection::South,
            EdgeDirection::East,
            EdgeDirection::West,
        ];

        for direction in &directions {
            let constraints = direction.get_constraints(ozone);
            let river_category = match direction {
                EdgeDirection::North => RiverCategory::North,
                EdgeDirection::South => RiverCategory::South,
                EdgeDirection::East => RiverCategory::East,
                EdgeDirection::West => RiverCategory::West,
            };

            // Group consecutive river tiles
            let mut current_river_start: Option<usize> = None;
            let mut current_river_type = RiverType::Creek;

            for (index, constraint) in constraints.iter().enumerate() {
                let (x, y) = direction.get_position(index);

                if let ZoneConstraintType::River(river_type) = constraint {
                    // Mark edge tiles as river
                    terrain.set(x, y, Terrain::River);
                    locked.set(x, y, true);

                    // Track the start of a river segment
                    if current_river_start.is_none() {
                        current_river_start = Some(index);
                        current_river_type = *river_type;
                    }
                } else if let Some(start) = current_river_start {
                    // End of river segment - add connection at middle point
                    let middle = start + (index - start) / 2;
                    let (mx, my) = direction.get_position(middle);

                    river_builder.add_connection(RiverConnection {
                        category: river_category,
                        pos: (mx, my),
                        river_type: current_river_type,
                    });

                    current_river_start = None;
                }
            }

            // Handle river at end of edge
            if let Some(start) = current_river_start {
                let edge_length = match direction {
                    EdgeDirection::North | EdgeDirection::South => ZONE_SIZE.0,
                    EdgeDirection::East | EdgeDirection::West => ZONE_SIZE.1,
                };
                let middle = start + (edge_length - start) / 2;
                let (mx, my) = direction.get_position(middle);

                river_builder.add_connection(RiverConnection {
                    category: river_category,
                    pos: (mx, my),
                    river_type: current_river_type,
                });
            }
        }
    }

    fn apply_rock_constraints(
        ozone: &OverworldZone,
        entities: &mut Grid<Vec<Prefab>>,
        locked: &mut Grid<bool>,
        _road_builder: &mut RoadBuilder,
    ) {
        let directions = [
            EdgeDirection::North,
            EdgeDirection::South,
            EdgeDirection::East,
            EdgeDirection::West,
        ];

        for direction in &directions {
            let constraints = direction.get_constraints(ozone);

            for (index, constraint) in constraints.iter().enumerate() {
                let (x, y) = direction.get_position(index);

                if let ZoneConstraintType::Rock = constraint {
                    // Only place rock if the tile isn't already locked by rivers or roads
                    if !locked.get(x, y).copied().unwrap_or(false) {
                        Self::place_boulder(ozone.zone_idx, x, y, entities, locked);
                    }
                }
            }
        }
    }

    fn apply_foliage_constraints(
        ozone: &OverworldZone,
        entities: &mut Grid<Vec<Prefab>>,
        locked: &mut Grid<bool>,
    ) {
        let directions = [
            EdgeDirection::North,
            EdgeDirection::South,
            EdgeDirection::East,
            EdgeDirection::West,
        ];

        for direction in &directions {
            let constraints = direction.get_constraints(ozone);

            for (index, constraint) in constraints.iter().enumerate() {
                let (x, y) = direction.get_position(index);

                if let ZoneConstraintType::Foliage = constraint {
                    // Only place foliage if the tile isn't already locked
                    if !locked.get(x, y).copied().unwrap_or(false) {
                        Self::place_foliage(ozone, x, y, entities, locked);
                    }
                }
            }
        }
    }

    fn place_foliage(
        ozone: &OverworldZone,
        x: usize,
        y: usize,
        entities: &mut Grid<Vec<Prefab>>,
        locked: &mut Grid<bool>,
    ) {
        let world_pos = zone_local_to_world(ozone.zone_idx, x, y);

        let prefab_id = match ozone.biome_type {
            crate::domain::BiomeType::Forest => PrefabId::PineTree,
            crate::domain::BiomeType::Desert => PrefabId::Cactus,
            crate::domain::BiomeType::Cavern => PrefabId::GiantMushroom,
            crate::domain::BiomeType::Mountain => PrefabId::PineTree,
            _ => return,
        };

        let config = Prefab::new(prefab_id, world_pos);

        if let Some(entities_at_pos) = entities.get_mut(x, y) {
            entities_at_pos.push(config);
        }
        locked.set(x, y, true);
    }
}
