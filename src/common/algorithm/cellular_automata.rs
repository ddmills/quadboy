use crate::common::{Grid, Rand};

pub trait Rule<T> {
    fn apply(&self, x: usize, y: usize, current: &T, neighbors: &NeighborData<T>) -> T;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Neighborhood {
    Moore,
    VonNeumann,
}

#[derive(Debug, Clone)]
pub enum BoundaryBehavior<T> {
    Wrap,
    Clamp,
    Constant(T),
}

pub struct NeighborData<T> {
    pub neighbors: Vec<T>,
    pub alive_count: usize,
}

impl<T> NeighborData<T>
where
    T: Clone + PartialEq,
{
    pub fn count_matching(&self, value: &T) -> usize {
        self.neighbors.iter().filter(|&n| n == value).count()
    }
}

pub struct CellularAutomata<T> {
    current: Grid<T>,
    next: Grid<T>,
    neighborhood: Neighborhood,
    boundary: BoundaryBehavior<T>,
    constraints: Option<Grid<bool>>,
}

impl<T> CellularAutomata<T>
where
    T: Clone + Default,
{
    pub fn new(width: usize, height: usize, initial_value: T) -> Self {
        Self {
            current: Grid::init(width, height, initial_value.clone()),
            next: Grid::init(width, height, initial_value.clone()),
            neighborhood: Neighborhood::Moore,
            boundary: BoundaryBehavior::Constant(initial_value),
            constraints: None,
        }
    }

    pub fn from_grid(grid: Grid<T>) -> Self {
        let width = grid.width();
        let height = grid.height();
        let default_value = T::default();

        Self {
            current: grid,
            next: Grid::init(width, height, default_value.clone()),
            neighborhood: Neighborhood::Moore,
            boundary: BoundaryBehavior::Constant(default_value),
            constraints: None,
        }
    }

    pub fn from_seed(
        width: usize,
        height: usize,
        density: f32,
        rand: &mut Rand,
        alive_value: T,
        dead_value: T,
    ) -> Self
    where
        T: Clone,
    {
        let grid = Grid::init_fill(width, height, |_, _| {
            if rand.bool(density) {
                alive_value.clone()
            } else {
                dead_value.clone()
            }
        });

        Self::from_grid(grid)
    }

    pub fn with_neighborhood(mut self, neighborhood: Neighborhood) -> Self {
        self.neighborhood = neighborhood;
        self
    }

    pub fn with_boundary(mut self, boundary: BoundaryBehavior<T>) -> Self {
        self.boundary = boundary;
        self
    }

    pub fn with_constraints(mut self, constraints: Grid<bool>) -> Self {
        self.constraints = Some(constraints);
        self
    }

    pub fn set_constraint(&mut self, x: usize, y: usize, locked: bool) {
        if self.constraints.is_none() {
            let width = self.current.width();
            let height = self.current.height();
            self.constraints = Some(Grid::init(width, height, false));
        }

        if let Some(ref mut constraints) = self.constraints {
            constraints.set(x, y, locked);
        }
    }

    pub fn get(&self, x: usize, y: usize) -> Option<&T> {
        self.current.get(x, y)
    }

    pub fn set(&mut self, x: usize, y: usize, value: T) {
        self.current.set(x, y, value);
    }

    pub fn grid(&self) -> &Grid<T> {
        &self.current
    }

    pub fn step<R>(&mut self, rule: &R)
    where
        R: Rule<T>,
    {
        let width = self.current.width();
        let height = self.current.height();

        for x in 0..width {
            for y in 0..height {
                let current_cell = self.current.get(x, y).unwrap();
                let neighbors = self.get_neighbors(x, y);
                let next_value = rule.apply(x, y, current_cell, &neighbors);
                self.next.set(x, y, next_value);
            }
        }

        if let Some(ref constraints) = self.constraints {
            for x in 0..width {
                for y in 0..height {
                    if let Some(&is_locked) = constraints.get(x, y)
                        && is_locked
                    {
                        let current_value = self.current.get(x, y).unwrap().clone();
                        self.next.set(x, y, current_value);
                    }
                }
            }
        }

        std::mem::swap(&mut self.current, &mut self.next);
    }

    pub fn evolve_steps<R>(&mut self, rule: &R, steps: usize)
    where
        R: Rule<T>,
    {
        for _ in 0..steps {
            self.step(rule);
        }
    }

    pub fn evolve_until_stable<R>(&mut self, rule: &R, max_steps: usize) -> usize
    where
        T: PartialEq,
        R: Rule<T>,
    {
        for step in 0..max_steps {
            let before = self.current.clone();
            self.step(rule);

            if self.grids_equal(&before, &self.current) {
                return step + 1;
            }
        }
        max_steps
    }

    fn grids_equal(&self, a: &Grid<T>, b: &Grid<T>) -> bool
    where
        T: PartialEq,
    {
        if a.width() != b.width() || a.height() != b.height() {
            return false;
        }

        for x in 0..a.width() {
            for y in 0..a.height() {
                if a.get(x, y) != b.get(x, y) {
                    return false;
                }
            }
        }
        true
    }

    fn get_neighbors(&self, x: usize, y: usize) -> NeighborData<T> {
        let mut neighbors = Vec::new();

        let offsets = match self.neighborhood {
            Neighborhood::Moore => vec![
                (-1, -1),
                (-1, 0),
                (-1, 1),
                (0, -1),
                (0, 1),
                (1, -1),
                (1, 0),
                (1, 1),
            ],
            Neighborhood::VonNeumann => vec![(-1, 0), (0, -1), (0, 1), (1, 0)],
        };

        for (dx, dy) in offsets {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;

            let neighbor_value = self.get_cell_with_boundary(nx, ny);
            neighbors.push(neighbor_value);
        }

        NeighborData {
            neighbors,
            alive_count: 0,
        }
    }

    fn get_cell_with_boundary(&self, x: i32, y: i32) -> T {
        let width = self.current.width() as i32;
        let height = self.current.height() as i32;

        if x >= 0 && y >= 0 && x < width && y < height {
            return self.current.get(x as usize, y as usize).unwrap().clone();
        }

        match &self.boundary {
            BoundaryBehavior::Constant(value) => value.clone(),
            BoundaryBehavior::Clamp => {
                let clamped_x = (x.max(0).min(width - 1)) as usize;
                let clamped_y = (y.max(0).min(height - 1)) as usize;
                self.current.get(clamped_x, clamped_y).unwrap().clone()
            }
            BoundaryBehavior::Wrap => {
                let wrapped_x = ((x % width + width) % width) as usize;
                let wrapped_y = ((y % height + height) % height) as usize;
                self.current.get(wrapped_x, wrapped_y).unwrap().clone()
            }
        }
    }
}
