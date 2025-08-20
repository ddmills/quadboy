use macroquad::prelude::warn;
use ordered_float::*;
use std::collections::HashMap;

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
}

pub struct AStarResult<T> {
    pub is_success: bool,
    pub path: Vec<T>,
    pub cost: f32,
}

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

    open.put(settings.start, OrderedFloat(0.));
    costs.insert(settings.start, OrderedFloat(0.));

    while !open.is_empty() {
        depth += 1;

        if depth >= settings.max_depth {
            warn!("astar max_depth={} exceeded", settings.max_depth);
            break;
        }

        let current = open.pop().unwrap();

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

            let new_cost = costs.get(&current).unwrap() + cost;

            if !costs.contains_key(&next) || new_cost < *costs.get(&next).unwrap() {
                costs.insert(next, new_cost);

                // todo: use a min priority queue and remove hard-coded float
                let priority = OrderedFloat(100000.0) - (new_cost + (settings.heuristic)(next));

                open.put(next, priority);
                from.insert(next, current);
            }
        }
    }

    if !result.is_success {
        return result;
    }

    let g = goal.unwrap();
    result.path.push(g);
    result.cost = **costs.get(&g).unwrap();

    let mut previous_pos = &g;

    while from.contains_key(previous_pos) {
        let f = from.get(previous_pos).unwrap();
        result.path.push(*f);
        previous_pos = f;
    }

    // note: path is returned in reverse order
    result
}
