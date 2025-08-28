use crate::{
    cfg::{MAP_SIZE, SURFACE_LEVEL_Z, ZONE_SIZE},
    common::{Direction, Rand},
    domain::{BiomeType, Overworld, RoadType},
    rendering::{zone_idx as calculate_zone_idx, zone_xyz},
};

pub struct ZoneContinuity {
    pub north: ZoneEdge,
    pub south: ZoneEdge,
    pub east: ZoneEdge,
    pub west: ZoneEdge,
    pub up: ZoneVerticalConstraints,
    pub down: ZoneVerticalConstraints,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ZoneConstraintType {
    None,
    Water,
    Road(RoadType),
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
        north: ZoneEdge(get_edge_continuity(overworld, zone_idx, Direction::North)),
        south: ZoneEdge(get_edge_continuity(overworld, zone_idx, Direction::South)),
        east: ZoneEdge(get_edge_continuity(overworld, zone_idx, Direction::East)),
        west: ZoneEdge(get_edge_continuity(overworld, zone_idx, Direction::West)),
        up: get_vertical_continuity(overworld, zone_idx),
        down: if let Some(above_idx) = zone_above {
            get_vertical_continuity(overworld, above_idx)
        } else {
            ZoneVerticalConstraints(vec![])
        },
    }
}

pub struct ZoneEdge(pub Vec<ZoneConstraintType>);

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
        let mut rand = Rand::seed(overworld.seed + zone_idx as u32 + neighbor as u32);

        let zone_biome = overworld.get_zone_type(zone_idx);
        let is_cavern = zone_biome == BiomeType::Cavern;

        let (num_rock_sections, min_length, max_length) = if is_cavern {
            (rand.range_n(3, 7), 4, 12)
        } else {
            (rand.range_n(1, 4), 2, 8)
        };

        for _ in 0..num_rock_sections {
            let start = rand.range_n(0, edge_length as i32) as usize;
            let length = rand.range_n(min_length, max_length);
            let end = (start + length as usize).min(edge_length);

            for i in start..end {
                edge_constraints[i] = ZoneConstraintType::Rock;
            }
        }

        // Roads take precedence - overwrite any rock constraints
        if overworld.zone_has_road(zone_idx) && overworld.zone_has_road(neighbor) {
            if let Some(road_network) = overworld.get_road_network(z) {
                if let Some(road_segment) = road_network
                    .edges
                    .get(&(zone_idx, neighbor))
                    .or_else(|| road_network.edges.get(&(neighbor, zone_idx)))
                {
                    let middle = edge_length / 2;
                    let width = road_segment.road_type.width();

                    for i in 0..width {
                        let pos = middle + i - (width / 2);
                        if pos < edge_constraints.len() {
                            edge_constraints[pos] =
                                ZoneConstraintType::Road(road_segment.road_type);
                        }
                    }
                }
            };
        }
    } else {
        edge_constraints.fill(ZoneConstraintType::Rock);
    }

    edge_constraints
}

pub fn get_vertical_continuity(overworld: &Overworld, zone_idx: usize) -> ZoneVerticalConstraints {
    let (_x, _y, z) = zone_xyz(zone_idx);
    let mut constraints = vec![];

    if z < SURFACE_LEVEL_Z {
        return ZoneVerticalConstraints(constraints);
    }

    if z + 1 < MAP_SIZE.2 {
        let below_z = z + 1;
        let below_is_cavern = below_z > SURFACE_LEVEL_Z;

        if below_is_cavern {
            let mut rand = Rand::seed(overworld.seed + zone_idx as u32 + 1000);

            let center_x = ZONE_SIZE.0 / 2;
            let center_y = ZONE_SIZE.1 / 2;

            let offset_range = 10.min(center_x - 5).min(center_y - 5);
            let x_offset = rand.range_n(-(offset_range as i32), offset_range as i32 + 1);
            let y_offset = rand.range_n(-(offset_range as i32), offset_range as i32 + 1);

            let stair_x = ((center_x as i32 + x_offset).max(5) as usize).min(ZONE_SIZE.0 - 6);
            let stair_y = ((center_y as i32 + y_offset).max(5) as usize).min(ZONE_SIZE.1 - 6);

            constraints.push(PositionalConstraint {
                position: (stair_x, stair_y),
                constraint: ZoneConstraintType::StairDown,
            });
        }
    }

    ZoneVerticalConstraints(constraints)
}
