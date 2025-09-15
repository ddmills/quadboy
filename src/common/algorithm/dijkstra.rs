use std::cmp::Ordering;
use std::collections::BinaryHeap;

use crate::common::{Grid, Rand};

#[derive(Copy, Clone, PartialEq)]
struct DijkstraNode {
    cost: f32,
    position: (usize, usize),
}

impl Eq for DijkstraNode {}

impl Ord for DijkstraNode {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .cost
            .partial_cmp(&self.cost)
            .unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for DijkstraNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct DijkstraMap {
    costs: Grid<f32>,
    dirty: bool,
}

impl DijkstraMap {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            costs: Grid::init(width, height, f32::INFINITY),
            dirty: true,
        }
    }

    pub fn width(&self) -> usize {
        self.costs.width()
    }

    pub fn height(&self) -> usize {
        self.costs.height()
    }

    pub fn get_cost(&self, x: usize, y: usize) -> Option<f32> {
        self.costs.get(x, y).copied()
    }

    pub fn set_blocked(&mut self, x: usize, y: usize) {
        if let Some(cost) = self.costs.get_mut(x, y) {
            *cost = f32::NEG_INFINITY;
            self.dirty = true;
        }
    }

    pub fn set_passable(&mut self, x: usize, y: usize) {
        if let Some(cost) = self.costs.get_mut(x, y) {
            if *cost == f32::NEG_INFINITY {
                *cost = f32::INFINITY;
                self.dirty = true;
            }
        }
    }

    pub fn is_blocked(&self, x: usize, y: usize) -> bool {
        self.costs
            .get(x, y)
            .map_or(true, |&cost| cost == f32::NEG_INFINITY)
    }

    pub fn calculate<F>(&mut self, goals: &[(usize, usize)], cost_fn: F)
    where
        F: Fn(usize, usize, usize, usize) -> f32,
    {
        // Clear all costs to INFINITY, but preserve blocked tiles (NEG_INFINITY)
        for x in 0..self.width() {
            for y in 0..self.height() {
                if let Some(cost) = self.costs.get_mut(x, y) {
                    if *cost != f32::NEG_INFINITY {
                        *cost = f32::INFINITY;
                    }
                }
            }
        }

        let mut heap = BinaryHeap::new();

        for &(x, y) in goals {
            if let Some(cost) = self.costs.get_mut(x, y) {
                *cost = 0.0;
                heap.push(DijkstraNode {
                    cost: 0.0,
                    position: (x, y),
                });
            }
        }

        while let Some(DijkstraNode {
            cost,
            position: (x, y),
        }) = heap.pop()
        {
            if let Some(&current_cost) = self.costs.get(x, y) {
                if cost > current_cost {
                    continue;
                }
            }

            for (dx, dy) in &[
                (-1, 0),
                (1, 0),
                (0, -1),
                (0, 1),
                (-1, -1),
                (-1, 1),
                (1, -1),
                (1, 1),
            ] {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;

                if nx < 0 || ny < 0 {
                    continue;
                }

                let (nx, ny) = (nx as usize, ny as usize);

                if nx >= self.width() || ny >= self.height() {
                    continue;
                }

                if self.is_blocked(nx, ny) {
                    continue;
                }

                let move_cost = cost_fn(x, y, nx, ny);
                let new_cost = cost + move_cost;

                if let Some(neighbor_cost) = self.costs.get_mut(nx, ny) {
                    if new_cost < *neighbor_cost {
                        *neighbor_cost = new_cost;
                        heap.push(DijkstraNode {
                            cost: new_cost,
                            position: (nx, ny),
                        });
                    }
                }
            }
        }

        self.dirty = false;
    }

    pub fn calculate_uniform(&mut self, goals: &[(usize, usize)]) {
        self.calculate(goals, |_, _, _, _| 1.0);
    }

    pub fn get_best_direction(&self, x: usize, y: usize) -> Option<(i32, i32)> {
        let current_cost = self.get_cost(x, y)?;

        if current_cost.is_infinite() {
            return None;
        }

        let mut best_direction = None;
        let mut best_cost = current_cost;

        for (dx, dy) in &[(-1, 0), (1, 0), (0, -1), (0, 1)] {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;

            if nx < 0 || ny < 0 {
                continue;
            }

            let (nx, ny) = (nx as usize, ny as usize);

            if let Some(neighbor_cost) = self.get_cost(nx, ny) {
                if neighbor_cost < best_cost && !neighbor_cost.is_infinite() {
                    best_cost = neighbor_cost;
                    best_direction = Some((*dx, *dy));
                }
            }
        }

        best_direction
    }

    pub fn get_neighbors_by_cost(&self, x: usize, y: usize) -> Vec<((usize, usize), f32)> {
        let mut neighbors = Vec::new();

        for (dx, dy) in &[(-1, 0), (1, 0), (0, -1), (0, 1)] {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;

            if nx < 0 || ny < 0 {
                continue;
            }

            let (nx, ny) = (nx as usize, ny as usize);

            if let Some(cost) = self.get_cost(nx, ny) {
                if !cost.is_infinite() && cost != f32::NEG_INFINITY {
                    neighbors.push(((nx, ny), cost));
                }
            }
        }

        neighbors.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal));
        neighbors
    }

    pub fn get_opposite_direction(&self, x: usize, y: usize) -> Option<(i32, i32)> {
        let current_cost = self.get_cost(x, y)?;

        if current_cost.is_infinite() {
            return None;
        }

        let mut worst_direction = None;
        let mut worst_cost = current_cost;

        for (dx, dy) in &[(-1, 0), (1, 0), (0, -1), (0, 1)] {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;

            if nx < 0 || ny < 0 {
                continue;
            }

            let (nx, ny) = (nx as usize, ny as usize);

            if let Some(neighbor_cost) = self.get_cost(nx, ny) {
                if neighbor_cost > worst_cost && neighbor_cost.is_finite() {
                    worst_cost = neighbor_cost;
                    worst_direction = Some((*dx, *dy));
                }
            }
        }

        worst_direction
    }

    pub fn get_perpendicular_direction(&self, x: usize, y: usize) -> Option<(i32, i32)> {
        let current_cost = self.get_cost(x, y)?;

        if current_cost.is_infinite() {
            return None;
        }

        // First, find the best direction toward goal
        let best_direction = self.get_best_direction(x, y);

        if let Some((best_dx, best_dy)) = best_direction {
            // Find perpendicular directions to the best path
            let perpendicular_dirs = if best_dx != 0 {
                // If moving horizontally, perpendicular is vertical
                vec![(0, -1), (0, 1)]
            } else if best_dy != 0 {
                // If moving vertically, perpendicular is horizontal
                vec![(-1, 0), (1, 0)]
            } else {
                // Should not happen, but return all directions as fallback
                vec![(-1, 0), (1, 0), (0, -1), (0, 1)]
            };

            // Choose the perpendicular direction with the lowest cost
            let mut best_perp_direction = None;
            let mut best_perp_cost = f32::INFINITY;

            for (dx, dy) in perpendicular_dirs {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;

                if nx < 0 || ny < 0 {
                    continue;
                }

                let (nx, ny) = (nx as usize, ny as usize);

                if let Some(neighbor_cost) = self.get_cost(nx, ny) {
                    if neighbor_cost < best_perp_cost && neighbor_cost.is_finite() {
                        best_perp_cost = neighbor_cost;
                        best_perp_direction = Some((dx, dy));
                    }
                }
            }

            best_perp_direction
        } else {
            None
        }
    }

    pub fn get_weighted_random_direction(
        &self,
        x: usize,
        y: usize,
        rand: &mut Rand,
    ) -> Option<(i32, i32)> {
        let current_cost = self.get_cost(x, y)?;

        if current_cost.is_infinite() {
            return None;
        }

        // Collect all valid directions with their costs
        let mut directions = Vec::new();

        for (dx, dy) in &[(-1, 0), (1, 0), (0, -1), (0, 1)] {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;

            if nx < 0 || ny < 0 {
                continue;
            }

            let (nx, ny) = (nx as usize, ny as usize);

            if let Some(neighbor_cost) = self.get_cost(nx, ny) {
                if neighbor_cost.is_finite() {
                    // Weight inversely proportional to cost (lower cost = higher weight)
                    let weight = if neighbor_cost == 0.0 {
                        1000.0 // Very high weight for goal
                    } else {
                        1.0 / neighbor_cost
                    };
                    directions.push(((*dx, *dy), weight));
                }
            }
        }

        if directions.is_empty() {
            return None;
        }

        // Calculate total weight
        let total_weight: f32 = directions.iter().map(|(_, weight)| weight).sum();

        if total_weight <= 0.0 {
            return None;
        }

        // Pick random direction based on weights
        let mut target = rand.random() * total_weight;

        for (direction, weight) in &directions {
            target -= weight;
            if target <= 0.0 {
                return Some(*direction);
            }
        }

        // Fallback to last direction if rounding errors occur
        directions.last().map(|(direction, _)| *direction)
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn clear(&mut self) {
        self.costs.clear(f32::INFINITY);
        self.dirty = true;
    }

    pub fn iter_costs(&self) -> impl Iterator<Item = (usize, usize, f32)> + '_ {
        self.costs.iter_xy().map(|(x, y, &cost)| (x, y, cost))
    }

    pub fn find_path_to_goal(
        &self,
        start: (usize, usize),
        max_steps: usize,
    ) -> Vec<(usize, usize)> {
        let mut path = Vec::new();
        let mut current = start;

        for _ in 0..max_steps {
            if let Some((dx, dy)) = self.get_best_direction(current.0, current.1) {
                let next = (
                    (current.0 as i32 + dx) as usize,
                    (current.1 as i32 + dy) as usize,
                );

                if let Some(cost) = self.get_cost(next.0, next.1) {
                    if cost == 0.0 {
                        path.push(next);
                        break;
                    }
                    path.push(next);
                    current = next;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dijkstra_basic() {
        let mut map = DijkstraMap::new(5, 5);
        let goals = vec![(2, 2)];

        map.calculate_uniform(&goals);

        assert_eq!(map.get_cost(2, 2), Some(0.0));
        assert_eq!(map.get_cost(2, 1), Some(1.0));
        assert_eq!(map.get_cost(1, 2), Some(1.0));
        assert_eq!(map.get_cost(0, 0), Some(4.0));
    }

    #[test]
    fn test_dijkstra_with_obstacles() {
        let mut map = DijkstraMap::new(5, 5);

        map.set_blocked(2, 1);
        map.set_blocked(2, 3);
        map.set_blocked(1, 2);
        map.set_blocked(3, 2);

        let goals = vec![(2, 2)];
        map.calculate_uniform(&goals);

        assert_eq!(map.get_cost(2, 2), Some(0.0));
        assert!(map.get_cost(2, 1).unwrap().is_infinite());
        assert!(map.get_cost(1, 2).unwrap().is_infinite());
    }

    #[test]
    fn test_best_direction() {
        let mut map = DijkstraMap::new(3, 3);
        let goals = vec![(2, 2)];

        map.calculate_uniform(&goals);

        let direction = map.get_best_direction(0, 0);
        assert!(direction.is_some());

        let (dx, dy) = direction.unwrap();
        assert!(dx == 1 || dy == 1);
    }

    #[test]
    fn test_opposite_direction() {
        let mut map = DijkstraMap::new(5, 5);
        let goals = vec![(2, 2)];

        map.calculate_uniform(&goals);

        // From corner, opposite direction should point away from goal
        if let Some((dx, dy)) = map.get_opposite_direction(0, 0) {
            // Should move away from (2,2), so dx should be negative and/or dy should be negative
            // In this case, from (0,0) the highest cost neighbor would be toward the edges
            assert!(dx <= 0 || dy <= 0);
        }

        // From goal position, should have no opposite direction (all neighbors are higher cost)
        // Actually, from goal all neighbors have cost 1, so any direction works
        let opposite_from_goal = map.get_opposite_direction(2, 2);
        assert!(opposite_from_goal.is_some());
    }

    #[test]
    fn test_perpendicular_direction() {
        let mut map = DijkstraMap::new(5, 5);
        let goals = vec![(4, 2)]; // Goal on the right side

        map.calculate_uniform(&goals);

        // From (0,2) the best direction should be horizontal (1,0)
        // So perpendicular should be vertical (0,-1) or (0,1)
        if let Some((dx, dy)) = map.get_perpendicular_direction(0, 2) {
            // Should be perpendicular to the horizontal movement
            assert!(dx == 0 && (dy == -1 || dy == 1));
        }

        // Test edge cases
        let perp_from_goal = map.get_perpendicular_direction(4, 2);
        // From goal, there should be no best direction, so no perpendicular either
        assert!(perp_from_goal.is_none());
    }

    #[test]
    fn test_weighted_random_direction() {
        let mut map = DijkstraMap::new(5, 5);
        let goals = vec![(2, 2)];
        let mut rand = Rand::new();

        map.calculate_uniform(&goals);

        // Test multiple times to ensure it returns valid directions
        for _ in 0..10 {
            if let Some((dx, dy)) = map.get_weighted_random_direction(0, 0, &mut rand) {
                // Should be a valid cardinal direction
                assert!(
                    (dx == -1 && dy == 0)
                        || (dx == 1 && dy == 0)
                        || (dx == 0 && dy == -1)
                        || (dx == 0 && dy == 1)
                );
            }
        }

        // Test from blocked position
        map.set_blocked(0, 0);
        map.calculate_uniform(&goals);

        let blocked_result = map.get_weighted_random_direction(0, 0, &mut rand);
        assert!(blocked_result.is_none());
    }

    #[test]
    fn test_edge_cases() {
        let mut map = DijkstraMap::new(3, 3);

        // Test with no goals calculated
        assert!(map.get_best_direction(1, 1).is_none());
        assert!(map.get_opposite_direction(1, 1).is_none());
        assert!(map.get_perpendicular_direction(1, 1).is_none());

        let mut rand = Rand::new();
        assert!(map.get_weighted_random_direction(1, 1, &mut rand).is_none());

        // Test with blocked position
        map.set_blocked(1, 1);
        let goals = vec![(2, 2)];
        map.calculate_uniform(&goals);

        assert!(map.get_best_direction(1, 1).is_none());
        assert!(map.get_opposite_direction(1, 1).is_none());
        assert!(map.get_perpendicular_direction(1, 1).is_none());
        assert!(map.get_weighted_random_direction(1, 1, &mut rand).is_none());
    }
}
