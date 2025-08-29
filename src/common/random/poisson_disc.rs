use crate::common::{Grid, Rand};

pub struct PoissonDiscSettings {
    pub width: usize,
    pub height: usize,
    pub radius: f32,
    pub seed: u32,
}

pub struct PoissonDiscSampler {
    settings: PoissonDiscSettings,
    rng: Rand,
    grid: Grid<Option<(usize, usize)>>,
    cell_size: f32,
    active_list: Vec<(usize, usize)>,
    samples: Vec<(usize, usize)>,
}

impl PoissonDiscSampler {
    pub fn new(settings: PoissonDiscSettings) -> Self {
        let cell_size = settings.radius / (2.0_f32).sqrt();
        let grid_width = (settings.width as f32 / cell_size).ceil() as usize;
        let grid_height = (settings.height as f32 / cell_size).ceil() as usize;

        let mut grid = Grid::init(grid_width, grid_height, None);
        let mut rng = Rand::seed(settings.seed);
        let mut active_list = Vec::new();
        let mut samples = Vec::new();

        let initial_x = rng.range_n(0, settings.width as i32) as usize;
        let initial_y = rng.range_n(0, settings.height as i32) as usize;

        let grid_x = (initial_x as f32 / cell_size) as usize;
        let grid_y = (initial_y as f32 / cell_size) as usize;

        grid.set(grid_x, grid_y, Some((initial_x, initial_y)));
        active_list.push((initial_x, initial_y));
        samples.push((initial_x, initial_y));

        Self {
            settings,
            rng,
            grid,
            cell_size,
            active_list,
            samples,
        }
    }

    pub fn sample(&mut self) -> Option<(usize, usize)> {
        while !self.active_list.is_empty() {
            let idx = self.rng.range_n(0, self.active_list.len() as i32) as usize;
            let current = self.active_list[idx];

            for _ in 0..30 {
                let angle = self.rng.random() * 2.0 * std::f32::consts::PI;
                let distance = self.settings.radius + self.rng.random() * self.settings.radius;

                let new_x = current.0 as f32 + angle.cos() * distance;
                let new_y = current.1 as f32 + angle.sin() * distance;

                if new_x >= 0.0
                    && new_x < self.settings.width as f32
                    && new_y >= 0.0
                    && new_y < self.settings.height as f32
                {
                    let new_point = (new_x as usize, new_y as usize);

                    if self.is_valid_point(new_point) {
                        let grid_x = (new_x / self.cell_size) as usize;
                        let grid_y = (new_y / self.cell_size) as usize;

                        self.grid.set(grid_x, grid_y, Some(new_point));
                        self.active_list.push(new_point);
                        self.samples.push(new_point);
                        return Some(new_point);
                    }
                }
            }

            // No valid point found after 30 attempts, remove from active list
            self.active_list.remove(idx);
        }

        None
    }

    pub fn all(&mut self) -> Vec<(usize, usize)> {
        while self.sample().is_some() {}
        self.samples.clone()
    }

    fn is_valid_point(&self, point: (usize, usize)) -> bool {
        let grid_x = (point.0 as f32 / self.cell_size) as usize;
        let grid_y = (point.1 as f32 / self.cell_size) as usize;

        let search_radius = 2;

        for x in grid_x.saturating_sub(search_radius)
            ..=(grid_x + search_radius).min(self.grid.width() - 1)
        {
            for y in grid_y.saturating_sub(search_radius)
                ..=(grid_y + search_radius).min(self.grid.height() - 1)
            {
                if let Some(Some(existing_point)) = self.grid.get(x, y) {
                    let dx = point.0 as f32 - existing_point.0 as f32;
                    let dy = point.1 as f32 - existing_point.1 as f32;
                    let distance = (dx * dx + dy * dy).sqrt();

                    if distance < self.settings.radius {
                        return false;
                    }
                }
            }
        }

        true
    }
}
