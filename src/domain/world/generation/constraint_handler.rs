use crate::{
    cfg::ZONE_SIZE,
    common::{
        Grid,
        algorithm::{
            astar::{AStarSettings, astar},
            distance::Distance,
        },
    },
    domain::{
        OverworldZone, PrefabId, RiverType, SpawnConfig, Terrain, ZoneConstraintType, ZoneGrid,
    },
    rendering::zone_local_to_world,
};

use super::road_builder::{RoadBuilder, RoadCategory, RoadConnection};

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
        entities: &mut Grid<Vec<SpawnConfig>>,
        locked: &mut Grid<bool>,
        road_builder: &mut RoadBuilder,
        road_terrain: Terrain,
    ) {
        // Apply roads first
        Self::apply_road_constraints(ozone, terrain, locked, road_builder, road_terrain);
        // Apply rivers second (they can override roads)
        Self::apply_river_constraints(ozone, terrain, locked);
        // Create river flows through the zone interior
        Self::create_river_flows(ozone, terrain, locked);
        // Apply other constraints
        Self::apply_rock_constraints(ozone, entities, locked, road_builder);
        Self::apply_vertical_constraints(ozone, entities, locked, road_builder);
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

    fn place_river_tiles(
        x: usize,
        y: usize,
        river_type: RiverType,
        terrain: &mut Grid<Terrain>,
        locked: &mut Grid<bool>,
    ) {
        // Place river water (overrides any existing terrain, including roads)
        terrain.set(x, y, Terrain::River);
        locked.set(x, y, true);

        // Add bank erosion for wider rivers
        if river_type.width() >= 2 {
            Self::add_river_banks(x, y, terrain);
        }
    }

    fn add_river_banks(x: usize, y: usize, terrain: &mut Grid<Terrain>) {
        // Add sandy/dirt banks adjacent to rivers
        let bank_positions = [
            (x.saturating_sub(1), y),
            (x + 1, y),
            (x, y.saturating_sub(1)),
            (x, y + 1),
        ];

        for (bx, by) in bank_positions {
            if bx < ZONE_SIZE.0 && by < ZONE_SIZE.1 {
                // Only place banks if the terrain is grass (don't override other features)
                if let Some(&current_terrain) = terrain.get(bx, by) {
                    if current_terrain == Terrain::Grass {
                        terrain.set(bx, by, Terrain::Sand); // River banks
                    }
                }
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

                if let ZoneConstraintType::River(river_type) = constraint {
                    Self::place_river_tiles(x, y, *river_type, terrain, locked);
                }
            }
        }
    }

    fn apply_rock_constraints(
        ozone: &OverworldZone,
        entities: &mut Grid<Vec<SpawnConfig>>,
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

    fn create_river_flows(
        ozone: &OverworldZone,
        terrain: &mut Grid<Terrain>,
        locked: &mut Grid<bool>,
    ) {
        // Find river entry and exit points
        let river_points = Self::find_river_edge_points(ozone);

        // Connect river points with flowing water
        for connection in Self::plan_river_connections(&river_points) {
            Self::create_river_path(connection, terrain, locked);
        }
    }

    fn find_river_edge_points(ozone: &OverworldZone) -> Vec<RiverPoint> {
        let mut points = Vec::new();

        // Check north edge - group consecutive river tiles
        let mut river_start = None;
        let mut river_type = RiverType::Creek;
        for (index, constraint) in ozone.constraints.north.0.iter().enumerate() {
            if let ZoneConstraintType::River(rt) = constraint {
                if river_start.is_none() {
                    river_start = Some(index);
                    river_type = *rt;
                }
            } else if let Some(start) = river_start {
                // End of river segment - use middle point
                let middle = start + (index - start) / 2;
                points.push(RiverPoint {
                    position: (middle, 0),
                    river_type,
                    edge: EdgeDirection::North,
                });
                river_start = None;
            }
        }
        // Handle river at end of edge
        if let Some(start) = river_start {
            let middle = start + (ZONE_SIZE.0 - start) / 2;
            points.push(RiverPoint {
                position: (middle, 0),
                river_type,
                edge: EdgeDirection::North,
            });
        }

        // Check south edge - group consecutive river tiles
        river_start = None;
        for (index, constraint) in ozone.constraints.south.0.iter().enumerate() {
            if let ZoneConstraintType::River(rt) = constraint {
                if river_start.is_none() {
                    river_start = Some(index);
                    river_type = *rt;
                }
            } else if let Some(start) = river_start {
                let middle = start + (index - start) / 2;
                points.push(RiverPoint {
                    position: (middle, ZONE_SIZE.1 - 1),
                    river_type,
                    edge: EdgeDirection::South,
                });
                river_start = None;
            }
        }
        if let Some(start) = river_start {
            let middle = start + (ZONE_SIZE.0 - start) / 2;
            points.push(RiverPoint {
                position: (middle, ZONE_SIZE.1 - 1),
                river_type,
                edge: EdgeDirection::South,
            });
        }

        // Check east edge - group consecutive river tiles
        river_start = None;
        for (index, constraint) in ozone.constraints.east.0.iter().enumerate() {
            if let ZoneConstraintType::River(rt) = constraint {
                if river_start.is_none() {
                    river_start = Some(index);
                    river_type = *rt;
                }
            } else if let Some(start) = river_start {
                let middle = start + (index - start) / 2;
                points.push(RiverPoint {
                    position: (ZONE_SIZE.0 - 1, middle),
                    river_type,
                    edge: EdgeDirection::East,
                });
                river_start = None;
            }
        }
        if let Some(start) = river_start {
            let middle = start + (ZONE_SIZE.1 - start) / 2;
            points.push(RiverPoint {
                position: (ZONE_SIZE.0 - 1, middle),
                river_type,
                edge: EdgeDirection::East,
            });
        }

        // Check west edge - group consecutive river tiles
        river_start = None;
        for (index, constraint) in ozone.constraints.west.0.iter().enumerate() {
            if let ZoneConstraintType::River(rt) = constraint {
                if river_start.is_none() {
                    river_start = Some(index);
                    river_type = *rt;
                }
            } else if let Some(start) = river_start {
                let middle = start + (index - start) / 2;
                points.push(RiverPoint {
                    position: (0, middle),
                    river_type,
                    edge: EdgeDirection::West,
                });
                river_start = None;
            }
        }
        if let Some(start) = river_start {
            let middle = start + (ZONE_SIZE.1 - start) / 2;
            points.push(RiverPoint {
                position: (0, middle),
                river_type,
                edge: EdgeDirection::West,
            });
        }

        points
    }

    fn plan_river_connections(river_points: &[RiverPoint]) -> Vec<RiverConnection> {
        let mut connections = Vec::new();

        if river_points.is_empty() {
            return connections;
        }

        if river_points.len() == 1 {
            // Single river point - create a lake or short river segment
            let point = &river_points[0];
            let center = (ZONE_SIZE.0 / 2, ZONE_SIZE.1 / 2);
            connections.push(RiverConnection {
                start: point.position,
                end: center,
                river_type: point.river_type,
            });
        } else if river_points.len() == 2 {
            // Two points - connect them directly
            connections.push(RiverConnection {
                start: river_points[0].position,
                end: river_points[1].position,
                river_type: river_points[0].river_type.max(river_points[1].river_type),
            });
        } else {
            // Multiple points - create a river network
            // First, try to connect opposite edges
            let north_points: Vec<_> = river_points
                .iter()
                .filter(|p| p.edge == EdgeDirection::North)
                .collect();
            let south_points: Vec<_> = river_points
                .iter()
                .filter(|p| p.edge == EdgeDirection::South)
                .collect();
            let east_points: Vec<_> = river_points
                .iter()
                .filter(|p| p.edge == EdgeDirection::East)
                .collect();
            let west_points: Vec<_> = river_points
                .iter()
                .filter(|p| p.edge == EdgeDirection::West)
                .collect();

            let mut used_points = Vec::new();

            // Connect north-south pairs
            for north in &north_points {
                for south in &south_points {
                    if !used_points.contains(&north.position)
                        && !used_points.contains(&south.position)
                    {
                        connections.push(RiverConnection {
                            start: north.position,
                            end: south.position,
                            river_type: north.river_type.max(south.river_type),
                        });
                        used_points.push(north.position);
                        used_points.push(south.position);
                        break;
                    }
                }
            }

            // Connect east-west pairs
            for east in &east_points {
                for west in &west_points {
                    if !used_points.contains(&east.position)
                        && !used_points.contains(&west.position)
                    {
                        connections.push(RiverConnection {
                            start: east.position,
                            end: west.position,
                            river_type: east.river_type.max(west.river_type),
                        });
                        used_points.push(east.position);
                        used_points.push(west.position);
                        break;
                    }
                }
            }

            // Connect any remaining unconnected points to the nearest connected point or center
            for point in river_points {
                if !used_points.contains(&point.position) {
                    // Find nearest used point or use center
                    if let Some(nearest) = Self::find_nearest_point(point.position, &used_points) {
                        connections.push(RiverConnection {
                            start: point.position,
                            end: nearest,
                            river_type: point.river_type,
                        });
                    } else {
                        // Connect to zone center
                        let center = (ZONE_SIZE.0 / 2, ZONE_SIZE.1 / 2);
                        connections.push(RiverConnection {
                            start: point.position,
                            end: center,
                            river_type: point.river_type,
                        });
                    }
                    used_points.push(point.position);
                }
            }
        }

        connections
    }

    fn find_nearest_point(
        pos: (usize, usize),
        points: &[(usize, usize)],
    ) -> Option<(usize, usize)> {
        points
            .iter()
            .min_by_key(|&&(px, py)| {
                let dx = (pos.0 as i32 - px as i32).abs();
                let dy = (pos.1 as i32 - py as i32).abs();
                dx + dy // Manhattan distance
            })
            .copied()
    }

    fn create_river_path(
        connection: RiverConnection,
        terrain: &mut Grid<Terrain>,
        locked: &mut Grid<bool>,
    ) {
        // Use a seeded random grid for natural river meandering
        let seed = (connection.start.0 * 1000
            + connection.start.1 * 100
            + connection.end.0 * 10
            + connection.end.1) as u32;
        let path = Self::calculate_river_path_astar(connection.start, connection.end, locked, seed);
        let width = connection.river_type.width();

        for (x, y) in path {
            // Place main river channel
            Self::place_single_river_tile(x, y, terrain, locked);

            // Add width for larger rivers
            if width > 1 {
                Self::add_river_width_at_position(x, y, width, terrain, locked);
            }
        }
    }

    fn calculate_river_path_astar(
        start: (usize, usize),
        end: (usize, usize),
        locked: &Grid<bool>,
        seed: u32,
    ) -> Vec<(usize, usize)> {
        // Create a random grid for natural meandering (rivers prefer certain paths)
        let grid_bool = ZoneGrid::rand_bool(seed);

        let result = astar(AStarSettings {
            start: [start.0, start.1],
            is_goal: |p| p[0] == end.0 && p[1] == end.1,
            cost: |_, [x, y]| {
                // Check if position is already locked (has terrain)
                if *locked.get(x, y).unwrap_or(&true) {
                    return 50.0; // Rivers can flow through locked tiles but prefer not to
                }

                // Use random grid to create meandering effect
                let r = grid_bool.get(x, y).unwrap_or(&false);
                let rand_cost = if *r {
                    3.0 // Higher cost for true cells (creates meandering)
                } else {
                    1.0 // Lower cost for false cells (preferred path)
                };

                1.0 + rand_cost
            },
            heuristic: |[x, y]| {
                // Use Manhattan distance with small weight to encourage exploration
                0.5 * Distance::manhattan([x as i32, y as i32, 0], [end.0 as i32, end.1 as i32, 0])
            },
            neighbors: |[x, y]| Self::get_river_neighbors((x, y)),
            max_depth: 10000,
            max_cost: None,
        });

        if result.is_success {
            result.path.into_iter().map(|[x, y]| (x, y)).collect()
        } else {
            // Fallback to direct line if pathfinding fails
            vec![start, end]
        }
    }

    fn get_river_neighbors(pos: (usize, usize)) -> Vec<[usize; 2]> {
        let mut neighbors = vec![];
        let (x, y) = pos;

        // Cardinals and diagonals for smoother river flow
        if x > 0 {
            neighbors.push([x - 1, y]);
            if y > 0 {
                neighbors.push([x - 1, y - 1]);
            }
            if y < ZONE_SIZE.1 - 1 {
                neighbors.push([x - 1, y + 1]);
            }
        }

        if x < ZONE_SIZE.0 - 1 {
            neighbors.push([x + 1, y]);
            if y > 0 {
                neighbors.push([x + 1, y - 1]);
            }
            if y < ZONE_SIZE.1 - 1 {
                neighbors.push([x + 1, y + 1]);
            }
        }

        if y > 0 {
            neighbors.push([x, y - 1]);
        }

        if y < ZONE_SIZE.1 - 1 {
            neighbors.push([x, y + 1]);
        }

        neighbors
    }

    fn place_single_river_tile(
        x: usize,
        y: usize,
        terrain: &mut Grid<Terrain>,
        locked: &mut Grid<bool>,
    ) {
        if x < ZONE_SIZE.0 && y < ZONE_SIZE.1 {
            terrain.set(x, y, Terrain::River);
            locked.set(x, y, true);
        }
    }

    fn add_river_width_at_position(
        center_x: usize,
        center_y: usize,
        width: usize,
        terrain: &mut Grid<Terrain>,
        locked: &mut Grid<bool>,
    ) {
        let half_width = width / 2;

        for dx in 0..width {
            for dy in 0..width {
                let x = center_x.saturating_sub(half_width) + dx;
                let y = center_y.saturating_sub(half_width) + dy;

                if x < ZONE_SIZE.0 && y < ZONE_SIZE.1 {
                    // Only place river tiles within manhattan distance
                    let distance = dx.abs_diff(half_width) + dy.abs_diff(half_width);
                    if distance <= half_width {
                        Self::place_single_river_tile(x, y, terrain, locked);
                    }
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct RiverPoint {
    position: (usize, usize),
    river_type: RiverType,
    edge: EdgeDirection,
}

impl PartialEq<EdgeDirection> for RiverPoint {
    fn eq(&self, other: &EdgeDirection) -> bool {
        self.edge == *other
    }
}

struct RiverConnection {
    start: (usize, usize),
    end: (usize, usize),
    river_type: RiverType,
}

impl RiverType {
    fn max(self, other: RiverType) -> RiverType {
        match (self, other) {
            (RiverType::MightyRiver, _) | (_, RiverType::MightyRiver) => RiverType::MightyRiver,
            (RiverType::River, _) | (_, RiverType::River) => RiverType::River,
            (RiverType::Stream, _) | (_, RiverType::Stream) => RiverType::Stream,
            (RiverType::Creek, RiverType::Creek) => RiverType::Creek,
        }
    }
}
