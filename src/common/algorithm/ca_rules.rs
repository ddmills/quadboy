use super::cellular_automata::{NeighborData, Rule};

pub struct ConwaysLifeRule;

impl Rule<bool> for ConwaysLifeRule {
    fn apply(&self, _x: usize, _y: usize, current: &bool, neighbors: &NeighborData<bool>) -> bool {
        let alive_neighbors = neighbors.count_matching(&true);

        match (*current, alive_neighbors) {
            (true, 2) | (true, 3) => true,
            (false, 3) => true,
            _ => false,
        }
    }
}

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

    pub fn default_cavern() -> Self {
        Self::new(5, 4)
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

pub struct MajorityRule<T> {
    pub tie_breaker: T,
}

impl<T> MajorityRule<T>
where
    T: Clone,
{
    pub fn new(tie_breaker: T) -> Self {
        Self { tie_breaker }
    }
}

impl<T> Rule<T> for MajorityRule<T>
where
    T: Clone + PartialEq + Eq + std::hash::Hash,
{
    fn apply(&self, _x: usize, _y: usize, _current: &T, neighbors: &NeighborData<T>) -> T {
        if neighbors.neighbors.is_empty() {
            return self.tie_breaker.clone();
        }

        let mut counts: std::collections::HashMap<&T, usize> = std::collections::HashMap::new();
        for neighbor in &neighbors.neighbors {
            *counts.entry(neighbor).or_insert(0) += 1;
        }

        let max_count = counts.values().max().unwrap_or(&0);
        let most_common: Vec<_> = counts
            .iter()
            .filter(|&(_, count)| count == max_count)
            .map(|(value, _)| *value)
            .collect();

        if most_common.len() == 1 {
            most_common[0].clone()
        } else {
            self.tie_breaker.clone()
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

pub struct DistanceRule {
    pub inner_value: bool,
    pub outer_value: bool,
    pub transition_distance: usize,
}

impl DistanceRule {
    pub fn new(inner_value: bool, outer_value: bool, transition_distance: usize) -> Self {
        Self {
            inner_value,
            outer_value,
            transition_distance,
        }
    }
}

impl Rule<bool> for DistanceRule {
    fn apply(&self, x: usize, y: usize, _current: &bool, _neighbors: &NeighborData<bool>) -> bool {
        let min_edge_distance = x.min(y);

        if min_edge_distance < self.transition_distance {
            self.outer_value
        } else {
            self.inner_value
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

pub struct DilationRule {
    pub min_neighbors: usize,
}

impl DilationRule {
    pub fn new(min_neighbors: usize) -> Self {
        Self { min_neighbors }
    }
}

impl Rule<bool> for DilationRule {
    fn apply(&self, _x: usize, _y: usize, current: &bool, neighbors: &NeighborData<bool>) -> bool {
        if *current {
            true
        } else {
            let activating_neighbors = neighbors.count_matching(&true);
            activating_neighbors >= self.min_neighbors
        }
    }
}

pub struct EdgeBiasedCaveRule {
    pub width: usize,
    pub height: usize,
    pub edge_birth_threshold: usize,
    pub edge_survival_threshold: usize,
    pub center_birth_threshold: usize,
    pub center_survival_threshold: usize,
    pub transition_zone: f32,
}

impl EdgeBiasedCaveRule {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            edge_birth_threshold: 3,
            edge_survival_threshold: 2,
            center_birth_threshold: 4,
            center_survival_threshold: 3,
            transition_zone: 0.1,
        }
    }

    pub fn with_edge_thresholds(mut self, birth: usize, survival: usize) -> Self {
        self.edge_birth_threshold = birth;
        self.edge_survival_threshold = survival;
        self
    }

    pub fn with_center_thresholds(mut self, birth: usize, survival: usize) -> Self {
        self.center_birth_threshold = birth;
        self.center_survival_threshold = survival;
        self
    }

    pub fn with_transition_zone(mut self, transition: f32) -> Self {
        self.transition_zone = transition.clamp(0.0, 1.0);
        self
    }

    fn get_edge_distance_ratio(&self, x: usize, y: usize) -> f32 {
        let min_edge_distance = x.min(y).min(self.width - x - 1).min(self.height - y - 1);
        let max_possible_distance = (self.width.min(self.height) / 2) as f32;
        (min_edge_distance as f32 / max_possible_distance).min(1.0)
    }
}

impl Rule<bool> for EdgeBiasedCaveRule {
    fn apply(&self, x: usize, y: usize, current: &bool, neighbors: &NeighborData<bool>) -> bool {
        let wall_neighbors = neighbors.count_matching(&true);
        let distance_ratio = self.get_edge_distance_ratio(x, y);

        let birth_threshold = if distance_ratio < self.transition_zone {
            let t = distance_ratio / self.transition_zone;
            let threshold = self.edge_birth_threshold as f32 * (1.0 - t)
                + self.center_birth_threshold as f32 * t;
            threshold.round() as usize
        } else {
            self.center_birth_threshold
        };

        let survival_threshold = if distance_ratio < self.transition_zone {
            let t = distance_ratio / self.transition_zone;
            let threshold = self.edge_survival_threshold as f32 * (1.0 - t)
                + self.center_survival_threshold as f32 * t;
            threshold.round() as usize
        } else {
            self.center_survival_threshold
        };

        if *current {
            wall_neighbors >= survival_threshold
        } else {
            wall_neighbors >= birth_threshold
        }
    }
}
