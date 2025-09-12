use std::slice::{Iter, IterMut};

use serde::{Deserialize, Serialize};

// A Column-major 2D grid
#[allow(dead_code)]
#[derive(Default, Clone, Deserialize, Serialize)]
pub struct Grid<T> {
    data: Vec<T>,
    width: usize,
    height: usize,
}

#[allow(dead_code)]
impl<T> Grid<T> {
    pub fn init(width: usize, height: usize, value: T) -> Self
    where
        T: Clone,
    {
        Self {
            data: vec![value; width * height],
            width,
            height,
        }
    }

    pub fn init_fill<F>(width: usize, height: usize, mut fill_fn: F) -> Self
    where
        F: FnMut(usize, usize) -> T,
    {
        let mut data = Vec::with_capacity(width * height);

        for x in 0..width {
            for y in 0..height {
                data.push(fill_fn(x, y));
            }
        }

        Self {
            data,
            width,
            height,
        }
    }

    pub fn init_from_vec(width: usize, height: usize, data: Vec<T>) -> Self {
        Self {
            data,
            width,
            height,
        }
    }

    #[inline]
    pub fn xy(&self, idx: usize) -> (usize, usize) {
        (idx / self.height, idx % self.height)
    }

    #[inline]
    pub fn idx(&self, x: usize, y: usize) -> usize {
        x * self.height + y
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
    pub fn get(&self, x: usize, y: usize) -> Option<&T> {
        self.data.get(self.idx(x, y))
    }

    #[inline]
    pub fn get_at(&self, idx: usize) -> Option<&T> {
        self.data.get(idx)
    }

    #[inline]
    pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut T> {
        self.data.get_mut(x * self.height + y)
    }

    #[inline]
    pub fn get_at_mut(&mut self, idx: usize) -> Option<&mut T> {
        self.data.get_mut(idx)
    }

    #[inline]
    pub fn insert(&mut self, x: usize, y: usize, value: T) {
        self.data[x * self.height + y] = value;
    }

    #[inline]
    pub fn set(&mut self, x: usize, y: usize, value: T) {
        self.data[x * self.height + y] = value;
    }

    #[inline]
    pub fn set_at(&mut self, idx: usize, value: T) {
        self.data[idx] = value;
    }

    #[inline]
    pub fn clear(&mut self, value: T)
    where
        T: Clone,
    {
        self.data.fill(value);
    }

    #[inline]
    pub fn iter(&'_ self) -> Iter<'_, T> {
        self.data.iter()
    }

    #[inline]
    pub fn iter_mut(&'_ mut self) -> IterMut<'_, T> {
        self.data.iter_mut()
    }

    pub fn iter_xy(&self) -> impl Iterator<Item = (usize, usize, &T)> {
        self.data.iter().enumerate().map(move |(idx, value)| {
            let (x, y) = self.xy(idx);
            (x, y, value)
        })
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

    pub fn is_oob(&self, x: usize, y: usize) -> bool {
        x > 0 && y > 0 && x < self.width && y < self.height
    }

    pub fn is_on_edge(&self, x: usize, y: usize) -> bool {
        x == 0 || x == self.width - 1 || y == 0 || y == self.height - 1
    }

    // map from one grid of values to a new grid, of the same size
    pub fn map<F, V>(&self, mut map_fn: F) -> Grid<V>
    where
        F: FnMut(usize, usize, &T) -> V,
    {
        let mut data = Vec::with_capacity(self.width * self.height);

        for x in 0..self.width {
            for y in 0..self.height {
                let v = self.get(x, y).unwrap();
                data.push(map_fn(x, y, v));
            }
        }

        Grid {
            data,
            width: self.width,
            height: self.height,
        }
    }
}
