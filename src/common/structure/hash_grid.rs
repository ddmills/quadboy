use std::{collections::HashMap, hash::Hash, slice::Iter};

use serde::{Deserialize, Serialize};

use crate::common::Grid;

// A Column-major 2D grid with double-lookup
#[allow(dead_code)]
#[derive(Default, Clone, Deserialize, Serialize)]
pub struct HashGrid<T>
where
    T: Hash + Eq + Copy,
{
    grid: Grid<Vec<T>>,
    hash: HashMap<T, usize>,
    width: usize,
    height: usize,
}

#[allow(dead_code)]
impl<T> HashGrid<T>
where
    T: Hash + Eq + Copy,
{
    pub fn init(width: usize, height: usize) -> Self
where {
        let g = Grid::init_fill(width, height, |_, _| vec![]);

        Self {
            grid: g,
            width,
            height,
            hash: HashMap::new(),
        }
    }

    #[inline]
    pub fn xy(&self, idx: usize) -> (usize, usize) {
        self.grid.xy(idx)
    }

    #[inline]
    pub fn idx(&self, x: usize, y: usize) -> usize {
        self.grid.idx(x, y)
    }

    #[inline]
    pub fn width(&self) -> usize {
        self.width
    }

    #[inline]
    pub fn height(&self) -> usize {
        self.height
    }

    #[inline]
    pub fn get(&self, x: usize, y: usize) -> Option<&Vec<T>> {
        self.grid.get(x, y)
    }

    #[inline]
    pub fn get_at(&self, idx: usize) -> Option<&Vec<T>> {
        self.grid.get_at(idx)
    }

    #[inline]
    pub fn insert(&mut self, x: usize, y: usize, value: T) {
        self.insert_at(self.idx(x, y), value);
    }

    pub fn insert_at(&mut self, idx: usize, value: T) {
        self.remove(&value);
        let Some(v) = self.grid.get_at_mut(idx) else {
            return;
        };

        v.push(value);
        self.hash.insert(value, idx);
    }

    #[inline]
    pub fn has(&self, value: &T) -> bool {
        self.hash.contains_key(value)
    }

    pub fn remove(&mut self, value: &T) -> bool {
        let Some(idx) = self.hash.remove(value) else {
            return false;
        };

        let Some(cell) = self.grid.get_at_mut(idx) else {
            return false;
        };

        let Some(vec_idx) = cell.iter().position(|v| v == value) else {
            return false;
        };

        cell.swap_remove(vec_idx);

        true
    }

    #[inline]
    pub fn iter(&'_ self) -> Iter<'_, Vec<T>> {
        self.grid.iter()
    }

    pub fn fill<F>(&mut self, fill_fn: F)
    where
        F: Fn(usize, usize) -> T,
    {
        for x in 0..self.width {
            for y in 0..self.height {
                self.insert(x, y, fill_fn(x, y));
            }
        }
    }

    #[inline]
    pub fn is_oob(&self, x: usize, y: usize) -> bool {
        self.grid.is_oob(x, y)
    }

    #[inline]
    pub fn is_on_edge(&self, x: usize, y: usize) -> bool {
        self.grid.is_on_edge(x, y)
    }
}
