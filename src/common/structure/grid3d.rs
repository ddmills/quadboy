// A Column-major 3D grid
#[allow(dead_code)]
#[derive(Clone)]
pub struct Grid3d<T> {
    data: Vec<T>,
    width: usize,
    height: usize,
    depth: usize,
}

#[allow(dead_code)]
impl<T> Grid3d<T> {
    pub fn new(width: usize, height: usize, depth: usize) -> Self {
        Self {
            data: Vec::with_capacity(width * height * depth),
            width,
            height,
            depth,
        }
    }

    pub fn init(width: usize, height: usize, depth: usize, value: T) -> Self
    where
        T: Clone,
    {
        Self {
            data: vec![value; width * height],
            width,
            height,
            depth,
        }
    }

    #[inline]
    pub fn xyz(&self, idx: usize) -> (usize, usize, usize) {
        (
            idx / (self.height * self.depth),
            (idx / self.depth) % self.height,
            idx % self.depth,
        )
    }

    #[inline]
    pub fn idx(&self, x: usize, y: usize, z: usize) -> usize {
        x * self.height * self.depth + y * self.depth + z
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
    pub fn depth(&self) -> usize {
        self.depth
    }

    #[inline]
    pub fn get(&self, x: usize, y: usize, z: usize) -> Option<&T> {
        self.data.get(self.idx(x, y, z))
    }

    #[inline]
    pub fn get_mut(&mut self, x: usize, y: usize, z: usize) -> Option<&mut T> {
        self.data
            .get_mut(x * self.height * self.depth + y * self.depth + z)
    }

    #[inline]
    pub fn set(&mut self, x: usize, y: usize, z: usize, value: T) {
        self.data[x * self.height * self.depth + y * self.depth + z] = value;
    }

    #[inline]
    pub fn clear(&mut self, value: T)
    where
        T: Clone,
    {
        self.data.fill(value);
    }

    pub fn fill<F>(&mut self, fill_fn: F)
    where
        F: Fn(usize, usize, usize) -> T,
    {
        for x in 0..self.width {
            for y in 0..self.height {
                for z in 0..self.depth {
                    self.set(x, y, z, fill_fn(x, y, z));
                }
            }
        }
    }

    pub fn is_oob(&self, x: usize, y: usize, z: usize) -> bool {
        x >= self.width || y >= self.height || z >= self.depth
        // !(x > 0 && y > 0 && z > 0 && x < self.width - 1 && y < self.height - 1 && z < self.depth - 1)
    }

    pub fn is_on_edge(&self, x: usize, y: usize, z: usize) -> bool {
        x == 0
            || x == self.width - 1
            || y == 0
            || y == self.height - 1
            || z == 0
            || z == self.depth - 1
    }
}
