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

    pub fn roll(&mut self, dice_notation: &str) -> Result<i32, String> {
        let dice_notation = dice_notation.trim();

        if !dice_notation.contains('d') {
            return Err("Invalid dice notation: must contain 'd'".to_string());
        }

        let (dice_part, modifier) = if dice_notation.contains('+') {
            let parts: Vec<&str> = dice_notation.split('+').collect();
            if parts.len() != 2 {
                return Err("Invalid modifier format".to_string());
            }
            (
                parts[0],
                parts[1]
                    .parse::<i32>()
                    .map_err(|_| "Invalid modifier number")?,
            )
        } else if dice_notation.contains('-') {
            let parts: Vec<&str> = dice_notation.split('-').collect();
            if parts.len() != 2 {
                return Err("Invalid modifier format".to_string());
            }
            (
                parts[0],
                -(parts[1]
                    .parse::<i32>()
                    .map_err(|_| "Invalid modifier number")?),
            )
        } else {
            (dice_notation, 0)
        };

        let dice_parts: Vec<&str> = dice_part.split('d').collect();
        if dice_parts.len() != 2 {
            return Err("Invalid dice format: use XdY".to_string());
        }

        let count = dice_parts[0]
            .parse::<u32>()
            .map_err(|_| "Invalid dice count")?;
        let sides = dice_parts[1]
            .parse::<i32>()
            .map_err(|_| "Invalid die size")?;

        if count == 0 {
            return Err("Dice count must be greater than 0".to_string());
        }
        if sides <= 0 {
            return Err("Die size must be greater than 0".to_string());
        }

        let mut total = 0;
        for _ in 0..count {
            total += self.range_n(1, sides + 1);
        }

        Ok(total + modifier)
    }
}

impl Default for Rand {
    fn default() -> Self {
        Self::new()
    }
}
