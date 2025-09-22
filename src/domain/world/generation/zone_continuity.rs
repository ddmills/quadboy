use crate::{
    cfg::{MAP_SIZE, SURFACE_LEVEL_Z, ZONE_SIZE},
    common::{Direction, Rand},
    domain::{BiomeType, Overworld, RiverType, RoadType},
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
    Road(RoadType),
    River(RiverType),
    StairDown,
    Rock,
    Foliage,
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

            for constraint in edge_constraints.iter_mut().take(end).skip(start) {
                *constraint = ZoneConstraintType::Rock;
            }
        }

        // Add foliage generation based on biome
        let (foliage_density, foliage_min_length, foliage_max_length) = match zone_biome {
            BiomeType::Forest => (0.5, 1, 3),     // High density, small clusters
            BiomeType::Desert => (0.25, 1, 2),    // Medium density, smaller clusters
            BiomeType::DustyPlains => (0.15, 1, 2), // Low density, small clusters
            BiomeType::Cavern => (0.15, 1, 2),    // Low density, small clusters
            BiomeType::Mountain => (0.3, 1, 2),   // Moderate density, small clusters
            _ => (0.0, 0, 0),                     // No foliage for other biomes
        };

        if foliage_density > 0.0 {
            // Count available spaces (not Rock)
            let available_spaces: Vec<usize> = edge_constraints
                .iter()
                .enumerate()
                .filter_map(|(i, &c)| {
                    if c == ZoneConstraintType::None {
                        Some(i)
                    } else {
                        None
                    }
                })
                .collect();

            let num_foliage_sections =
                ((available_spaces.len() as f32 * foliage_density) / 2.0) as i32;

            for _ in 0..num_foliage_sections {
                if available_spaces.is_empty() {
                    break;
                }

                let start_idx = rand.range_n(0, available_spaces.len() as i32) as usize;
                let start = available_spaces[start_idx];
                let length = rand.range_n(foliage_min_length, foliage_max_length + 1) as usize;
                let end = (start + length).min(edge_length);

                for constraint in edge_constraints.iter_mut().take(end).skip(start) {
                    if *constraint == ZoneConstraintType::None {
                        *constraint = ZoneConstraintType::Foliage;
                    }
                }
            }
        }

        // Determine positions for roads and rivers with randomness
        let mut road_position = None;
        let mut river_position = None;

        // Check if both road and river exist
        let has_road = overworld.zone_has_road(zone_idx) && overworld.zone_has_road(neighbor);
        let has_river = overworld.zone_has_river(zone_idx) && overworld.zone_has_river(neighbor);

        if has_road && has_river {
            // Both exist - place them at different positions
            // Use consistent random seed for this edge - must be same for both zones sharing the edge
            // Sort zone indices to ensure consistency
            let (smaller_idx, larger_idx) = if zone_idx < neighbor {
                (zone_idx as u32, neighbor as u32)
            } else {
                (neighbor as u32, zone_idx as u32)
            };
            let edge_rand_seed = overworld.seed + smaller_idx * 10000 + larger_idx * 100;
            let mut edge_rand = Rand::seed(edge_rand_seed);

            // Divide edge into thirds for better separation
            let third = edge_length / 3;
            let river_zone = edge_rand.range_n(0, 3); // 0, 1, or 2
            let river_offset = edge_rand.range_n(5, third as i32 - 5).max(5) as usize;
            river_position = Some(river_zone as usize * third + river_offset);

            // Place road in a different third
            let road_zone = (river_zone + 1 + edge_rand.range_n(0, 2)) % 3;
            let road_offset = edge_rand.range_n(5, third as i32 - 5).max(5) as usize;
            road_position = Some(road_zone as usize * third + road_offset);
        } else if has_road || has_river {
            // Only one exists - can place anywhere along edge
            // Use consistent random seed for this edge - must be same for both zones sharing the edge
            // Sort zone indices to ensure consistency
            let (smaller_idx, larger_idx) = if zone_idx < neighbor {
                (zone_idx as u32, neighbor as u32)
            } else {
                (neighbor as u32, zone_idx as u32)
            };
            let edge_rand_seed = overworld.seed + smaller_idx * 10000 + larger_idx * 100;
            let mut edge_rand = Rand::seed(edge_rand_seed);

            // Random position with buffer from edges
            let buffer = 10.min(edge_length / 4);
            let position = edge_rand.range_n(buffer as i32, (edge_length - buffer) as i32) as usize;

            if has_road {
                road_position = Some(position);
            } else {
                river_position = Some(position);
            }
        }

        // Apply road constraints
        if let Some(pos) = road_position
            && let Some(road_network) = overworld.get_road_network(z)
            && let Some(road_segment) = road_network
                .edges
                .get(&(zone_idx, neighbor))
                .or_else(|| road_network.edges.get(&(neighbor, zone_idx)))
        {
            let width = road_segment.road_type.width();
            for i in 0..width {
                let tile_pos = (pos + i).saturating_sub(width / 2);
                if tile_pos < edge_constraints.len() {
                    edge_constraints[tile_pos] = ZoneConstraintType::Road(road_segment.road_type);
                }
            }
        }

        // Rivers take highest precedence - overwrite any rock/road constraints
        if let Some(pos) = river_position
            && let Some(river_network) = overworld.get_river_network(z)
            && let Some(river_segment) = river_network.get_river_at_edge(zone_idx, neighbor)
        {
            let width = river_segment.river_type.width();
            for i in 0..width {
                let tile_pos = (pos + i).saturating_sub(width / 2);
                if tile_pos < edge_constraints.len() {
                    edge_constraints[tile_pos] =
                        ZoneConstraintType::River(river_segment.river_type);
                }
            }
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
