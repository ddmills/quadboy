use bevy_ecs::resource::Resource;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

#[derive(Resource)]
pub struct Rand {
    r: SmallRng,
}

#[allow(dead_code)]
impl Rand {
    pub fn seed(seed: u32) -> Self {
        Self {
            r: SmallRng::seed_from_u64(seed as u64),
        }
    }

    pub fn new() -> Self {
        Self {
            r: SmallRng::from_os_rng(),
        }
    }

    pub fn pick<T>(&mut self, v: &[T]) -> T
    where
        T: Copy,
    {
        let idx = self.pick_idx(v);
        v[idx]
    }

    pub fn pick_idx<T>(&mut self, v: &[T]) -> usize {
        self.range_n(0, v.len() as i32) as usize
    }

    pub fn range_n(&mut self, min: i32, max: i32) -> i32 {
        (self.random() * (max as f32 - min as f32)) as i32 + min
    }

    pub fn d12(&mut self) -> i32 {
        self.range_n(1, 13)
    }

    #[inline]
    pub fn random(&mut self) -> f32 {
        self.r.random()
    }

    pub fn bool(&mut self, chance: f32) -> bool {
        self.random() < chance
    }
}

impl Default for Rand {
    fn default() -> Self {
        Self::new()
    }
}
