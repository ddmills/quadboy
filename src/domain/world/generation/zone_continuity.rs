use crate::{
    cfg::{MAP_SIZE, SURFACE_LEVEL_Z, ZONE_SIZE},
    common::Direction,
    domain::{Overworld, RoadType},
    rendering::{zone_idx as calculate_zone_idx, zone_xyz},
};

pub struct ZoneContinuity {
    pub north: ZoneEdgeConstraints,
    pub south: ZoneEdgeConstraints,
    pub east: ZoneEdgeConstraints,
    pub west: ZoneEdgeConstraints,
    pub up: ZoneVerticalConstraints,
    pub down: ZoneVerticalConstraints,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ZoneConstraintType {
    None,
    Water,
    Road(crate::domain::RoadType),
    StairDown,
    Rock,
}

pub fn get_zone_constraints(overworld: &Overworld, zone_idx: usize) -> ZoneContinuity {
    let (x, y, z) = zone_xyz(zone_idx);

    let zone_above = if z > 0 {
        Some(calculate_zone_idx(x, y, z - 1))
    } else {
        None
    };

    ZoneContinuity {
        north: ZoneEdgeConstraints(get_edge_continuity(overworld, zone_idx, Direction::North)),
        south: ZoneEdgeConstraints(get_edge_continuity(overworld, zone_idx, Direction::South)),
        east: ZoneEdgeConstraints(get_edge_continuity(overworld, zone_idx, Direction::East)),
        west: ZoneEdgeConstraints(get_edge_continuity(overworld, zone_idx, Direction::West)),
        up: get_vertical_continuity(overworld, zone_idx),
        down: if let Some(above_idx) = zone_above {
            get_vertical_continuity(overworld, above_idx)
        } else {
            ZoneVerticalConstraints(vec![])
        },
    }
}

pub struct ZoneEdgeConstraints(pub Vec<ZoneConstraintType>);

pub struct PositionalConstraint {
    pub position: (usize, usize),
    pub constraint: ZoneConstraintType,
}
pub struct ZoneVerticalConstraints(pub Vec<PositionalConstraint>);

pub fn get_edge_continuity(
    overworld: &Overworld,
    zone_idx: usize,
    direction: Direction,
) -> Vec<ZoneConstraintType> {
    let (x, y, z) = zone_xyz(zone_idx);

    let neighbor_idx = match direction {
        Direction::North => {
            if y > 0 {
                Some(calculate_zone_idx(x, y - 1, z))
            } else {
                None
            }
        }
        Direction::South => {
            if y + 1 < MAP_SIZE.1 {
                Some(calculate_zone_idx(x, y + 1, z))
            } else {
                None
            }
        }
        Direction::East => {
            if x + 1 < MAP_SIZE.0 {
                Some(calculate_zone_idx(x + 1, y, z))
            } else {
                None
            }
        }
        Direction::West => {
            if x > 0 {
                Some(calculate_zone_idx(x - 1, y, z))
            } else {
                None
            }
        }
    };

    let edge_length = match direction {
        Direction::North | Direction::South => ZONE_SIZE.0,
        Direction::East | Direction::West => ZONE_SIZE.1,
    };

    let mut edge_constraints = vec![ZoneConstraintType::None; edge_length];

    if let Some(neighbor) = neighbor_idx {
        if overworld.road_network.has_road(zone_idx) && overworld.road_network.has_road(neighbor) {
            // Get the road type from the road network
            if let Some(road_segment) = overworld
                .road_network
                .edges
                .get(&(zone_idx, neighbor))
                .or_else(|| overworld.road_network.edges.get(&(neighbor, zone_idx)))
            {
                let middle = edge_length / 2;
                let width = road_segment.road_type.width();

                // Place road constraints based on width
                for i in 0..width {
                    let pos = middle + i - (width / 2);
                    if pos < edge_constraints.len() {
                        edge_constraints[pos] = ZoneConstraintType::Road(road_segment.road_type);
                    }
                }
            }
        }
    } else {
        edge_constraints.fill(ZoneConstraintType::Rock);
    }

    edge_constraints
}

pub fn get_vertical_continuity(_overworld: &Overworld, zone_idx: usize) -> ZoneVerticalConstraints {
    let (_x, _y, z) = zone_xyz(zone_idx);
    let mut constraints = vec![];

    if z < SURFACE_LEVEL_Z {
        return ZoneVerticalConstraints(constraints);
    }

    if z + 1 < MAP_SIZE.2 {
        let below_z = z + 1;
        let below_is_cavern = below_z > SURFACE_LEVEL_Z;

        if below_is_cavern {
            constraints.push(PositionalConstraint {
                position: (ZONE_SIZE.0 / 2, ZONE_SIZE.1 / 2),
                constraint: ZoneConstraintType::StairDown,
            });
        }
    }

    ZoneVerticalConstraints(constraints)
}
