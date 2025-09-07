use super::cellular_automata::{NeighborData, Rule};

pub struct CaveRule {
    pub birth_threshold: usize,
    pub survival_threshold: usize,
}

impl CaveRule {
    pub fn new(birth_threshold: usize, survival_threshold: usize) -> Self {
        Self {
            birth_threshold,
            survival_threshold,
        }
    }
}

impl Rule<bool> for CaveRule {
    fn apply(&self, _x: usize, _y: usize, current: &bool, neighbors: &NeighborData<bool>) -> bool {
        let wall_neighbors = neighbors.count_matching(&true);

        if *current {
            wall_neighbors >= self.survival_threshold
        } else {
            wall_neighbors >= self.birth_threshold
        }
    }
}

pub struct SmoothingRule {
    pub threshold: f32,
}

impl SmoothingRule {
    pub fn new(threshold: f32) -> Self {
        Self { threshold }
    }
}

impl Rule<bool> for SmoothingRule {
    fn apply(&self, _x: usize, _y: usize, current: &bool, neighbors: &NeighborData<bool>) -> bool {
        let total_neighbors = neighbors.neighbors.len();
        if total_neighbors == 0 {
            return *current;
        }

        let matching_neighbors = neighbors.count_matching(current);
        let match_ratio = matching_neighbors as f32 / total_neighbors as f32;

        if match_ratio >= self.threshold {
            *current
        } else {
            !*current
        }
    }
}

pub struct ErosionRule {
    pub min_neighbors: usize,
}

impl ErosionRule {
    pub fn new(min_neighbors: usize) -> Self {
        Self { min_neighbors }
    }
}

impl Rule<bool> for ErosionRule {
    fn apply(&self, _x: usize, _y: usize, current: &bool, neighbors: &NeighborData<bool>) -> bool {
        if *current {
            let supporting_neighbors = neighbors.count_matching(&true);
            supporting_neighbors >= self.min_neighbors
        } else {
            false
        }
    }
}
