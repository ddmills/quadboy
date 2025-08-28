use super::cellular_automata::{Rule, NeighborData};

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
        Self { birth_threshold, survival_threshold }
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
        let most_common: Vec<_> = counts.iter()
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
        Self { inner_value, outer_value, transition_distance }
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