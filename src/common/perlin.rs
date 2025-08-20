use fastnoise_lite::{FastNoiseLite, FractalType, NoiseType};

#[allow(dead_code)]
pub struct Perlin {
    nz: FastNoiseLite,
    seed: u32,
}

#[allow(dead_code)]
impl Perlin {
    // lacunarity is best ~2
    pub fn new(seed: u32, frequency: f32, octaves: u32, lacunarity: f32) -> Self {
        let mut nz = FastNoiseLite::with_seed(seed as i32);
        nz.set_frequency(frequency.into());
        nz.set_fractal_octaves((octaves as i32).into());
        nz.set_fractal_lacunarity(lacunarity.into());
        nz.set_noise_type(NoiseType::OpenSimplex2.into());
        nz.set_fractal_type(FractalType::FBm.into());
        Self { nz, seed }
    }

    // get 2d noise value, clamped to [0, 1)
    pub fn get(&mut self, x: f32, y: f32) -> f32 {
        (self.nz.get_noise_2d(x, y) + 1.) / 2.
    }
}
