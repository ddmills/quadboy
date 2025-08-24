use macroquad::prelude::warn;
use ordered_float::*;
use std::collections::{HashMap, HashSet};

use crate::common::PriorityQueue;

pub struct AStarSettings<T, H, C, N, G>
where
    T: std::cmp::Eq + std::hash::Hash + Copy,
    H: Fn(T) -> f32,
    C: Fn(T, T) -> f32,
    N: Fn(T) -> Vec<T>,
    G: Fn(T) -> bool,
{
    pub start: T,
    pub is_goal: G,
    pub cost: C,
    // Heuristic should not exceed the real cost, or bad path is returned!
    pub heuristic: H,
    pub neighbors: N,
    pub max_depth: u32,
    pub max_cost: Option<f32>,
}

pub struct AStarResult<T> {
    pub is_success: bool,
    pub path: Vec<T>,
    pub cost: f32,
}

#[allow(dead_code)]
pub fn astar<T, H, C, N, G>(settings: AStarSettings<T, H, C, N, G>) -> AStarResult<T>
where
    H: Fn(T) -> f32,
    T: std::cmp::Eq + std::hash::Hash + Copy,
    C: Fn(T, T) -> f32,
    N: Fn(T) -> Vec<T>,
    G: Fn(T) -> bool,
{
    let mut depth = 0;
    let mut open = PriorityQueue::new();
    let mut from = HashMap::new();
    let mut costs = HashMap::new();
    let mut closed = HashSet::new();
    let mut goal: Option<T> = None;

    let mut result = AStarResult {
        is_success: false,
        path: vec![],
        cost: 0.,
    };

    if (settings.is_goal)(settings.start) {
        result.is_success = true;
        return result;
    }

    open.put(settings.start, OrderedFloat(0.0));
    costs.insert(settings.start, OrderedFloat(0.0));

    while !open.is_empty() {
        depth += 1;

        if depth >= settings.max_depth {
            warn!("astar max_depth={} exceeded", settings.max_depth);
            break;
        }

        let current = match open.pop() {
            Some(node) => node,
            None => break,
        };

        if closed.contains(&current) {
            continue;
        }
        closed.insert(current);

        if (settings.is_goal)(current) {
            result.is_success = true;
            goal = Some(current);
            break;
        }

        let neighbors = (settings.neighbors)(current);

        for next in neighbors {
            let cost = if (settings.is_goal)(next) {
                0.
            } else {
                (settings.cost)(current, next)
            };

            if cost == f32::INFINITY {
                continue;
            }

            let current_cost = match costs.get(&current) {
                Some(cost) => *cost,
                None => continue,
            };
            let new_cost = current_cost + cost;

            let should_update = match costs.get(&next) {
                Some(existing_cost) => new_cost < *existing_cost,
                None => true,
            };

            if should_update {
                costs.insert(next, new_cost);
                let f_cost = *new_cost + (settings.heuristic)(next);

                if let Some(max_cost) = settings.max_cost
                    && f_cost > max_cost
                {
                    continue;
                }

                open.put(next, OrderedFloat(f_cost));
                from.insert(next, current);
            }
        }
    }

    if !result.is_success {
        return result;
    }

    let g = match goal {
        Some(goal_node) => goal_node,
        None => return result,
    };

    result.path.push(g);
    result.cost = match costs.get(&g) {
        Some(cost) => **cost,
        None => 0.0,
    };

    let mut previous_pos = &g;

    while let Some(parent) = from.get(previous_pos) {
        result.path.push(*parent);
        previous_pos = parent;
    }

    // note: path is returned in reverse order
    result
}
