use bevy_ecs::prelude::*;
use macroquad::math::Vec3;

use crate::common::MacroquadColorable;

#[derive(Resource, Default)]
pub struct Clock {
    tick: u32,
    tick_delta: u32,
    force_update: bool,
}

impl Clock {
    pub fn new(tick: u32) -> Self {
        Self {
            tick,
            tick_delta: 0,
            force_update: true,
        }
    }

    pub fn is_frozen(&self) -> bool {
        self.tick_delta == 0 && !self.force_update
    }

    pub fn increment_tick(&mut self, amount: u32) {
        self.tick += amount;
        self.tick_delta += amount;
    }

    pub fn tick_delta(&self) -> u32 {
        self.tick_delta
    }

    pub fn current_tick(&self) -> u32 {
        self.tick
    }

    pub fn clear_tick_delta(&mut self) {
        self.tick_delta = 0;
        self.force_update = false;
    }

    pub fn force_update(&mut self) {
        self.force_update = true;
    }

    pub fn set_tick(&mut self, value: u32) {
        self.tick = value;
        self.force_update = true;
    }

    pub fn current_turn(&self) -> u32 {
        self.tick / 1000
    }

    pub fn sub_turn(&self) -> u32 {
        self.tick % 1000
    }

    pub fn get_minute(&self) -> u32 {
        self.tick / 100
    }

    pub fn get_hour(&self) -> u32 {
        (self.get_minute() / 60) % 24
    }

    pub fn get_day(&self) -> u32 {
        self.get_minute() / 1440 // 24 * 60 minutes per day
    }

    pub fn get_minute_of_day(&self) -> u32 {
        self.get_minute() % 1440
    }

    pub fn get_day_progress(&self) -> f32 {
        self.get_minute_of_day() as f32 / 1440.0
    }

    pub fn get_daylight(&self) -> DaylightInfo {
        let progress = self.get_day_progress();

        let phases = [
            DaylightPhase {
                progress: 0.0,
                color: 0x14556E,
                intensity: 0.3,
            },
            DaylightPhase {
                progress: 0.21,
                color: 0x6AA5A7,
                intensity: 0.5,
            },
            DaylightPhase {
                progress: 0.5,
                color: 0xFFFFFF,
                intensity: 1.0,
            },
            DaylightPhase {
                progress: 0.83,
                color: 0x6AA5A7,
                intensity: 0.5,
            },
            DaylightPhase {
                progress: 1.0,
                color: 0x14556E,
                intensity: 0.3,
            },
        ];

        let mut before_idx = 0;
        let mut after_idx = 1;

        for i in 0..(phases.len() - 1) {
            if progress >= phases[i].progress && progress <= phases[i + 1].progress {
                before_idx = i;
                after_idx = i + 1;
                break;
            }
        }

        let before = &phases[before_idx];
        let after = &phases[after_idx];

        let range = after.progress - before.progress;
        let t = if range > 0.0 {
            (progress - before.progress) / range
        } else {
            0.0
        };

        let before_rgba = before.color.to_rgba(1.0);
        let after_rgba = after.color.to_rgba(1.0);

        let color = Vec3::new(
            before_rgba[0] + (after_rgba[0] - before_rgba[0]) * t,
            before_rgba[1] + (after_rgba[1] - before_rgba[1]) * t,
            before_rgba[2] + (after_rgba[2] - before_rgba[2]) * t,
        );

        let intensity = before.intensity + (after.intensity - before.intensity) * t;

        DaylightInfo { color, intensity }
    }
}

#[derive(Clone, Copy)]
struct DaylightPhase {
    progress: f32,
    color: u32,
    intensity: f32,
}

#[derive(Clone, Copy)]
pub struct DaylightInfo {
    pub color: Vec3,
    pub intensity: f32,
}
