use crate::{
    cfg::ZONE_SIZE,
    common::{Grid, Perlin, Rand},
};

pub struct ZoneGrid;

impl ZoneGrid {
    pub fn edge_gradient(buffer: usize, pow: f32) -> Grid<f32> {
        let mut g = Grid::init(ZONE_SIZE.0, ZONE_SIZE.1, 1.);

        for x in 0..ZONE_SIZE.0 {
            for z in 0..buffer {
                let v = (z as f32 / buffer as f32).powf(pow);

                if z < x && z < (ZONE_SIZE.0 - x) {
                    g.set(x, z, v);
                    g.set(x, ZONE_SIZE.1 - z - 1, v);
                }
            }
        }

        for y in 0..ZONE_SIZE.1 {
            for z in 0..buffer {
                let v = (z as f32 / buffer as f32).powf(pow);

                if z <= y && z < (ZONE_SIZE.1 - y) {
                    g.set(z, y, v);
                    g.set(ZONE_SIZE.0 - z - 1, y, v);
                }
            }
        }

        g
    }

    pub fn perlin(seed: u32, frequency: f32, octaves: u32, lacunarity: f32) -> Grid<f32> {
        let mut nz = Perlin::new(seed, frequency, octaves, lacunarity);

        Grid::init_fill(ZONE_SIZE.0, ZONE_SIZE.1, |x, y| nz.get(x as f32, y as f32))
    }

    pub fn rand(seed: u32) -> Grid<f32> {
        let mut rand = Rand::seed(seed);

        Grid::init_fill(ZONE_SIZE.0, ZONE_SIZE.1, |_, _| rand.random())
    }

    pub fn bool(seed: u32) -> Grid<bool> {
        let mut rand = Rand::seed(seed);

        Grid::init_fill(ZONE_SIZE.0, ZONE_SIZE.1, |_, _| rand.bool(0.5))
    }
}
