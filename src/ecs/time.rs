use std::collections::VecDeque;

use bevy_ecs::prelude::*;
use macroquad::prelude::*;

use crate::rendering::Text;

#[derive(Resource, Default)]
pub struct Time {
    pub dt: f32,
    pub fps: i32,
    pub elapsed: f64,
    pub frames: VecDeque<i32>,
}

impl Time {
    pub fn get_smooth_avg(&self) -> i32
    {
        self.frames.iter().sum::<i32>() / 60
    }
    
    pub fn get_min_fps(&self) -> i32
    {
        *self.frames.iter().min().unwrap_or(&0)
    }
}

pub fn update_time(mut time: ResMut<Time>, mut fixed_time: ResMut<TimeFixed>) {
    let dt = get_frame_time();
    let fps = get_fps();
    time.dt = dt;
    time.fps = fps;
    time.elapsed = get_time();

    if time.frames.len() >= 60 {
        time.frames.pop_front();
    }

    time.frames.push_back(fps);

    fixed_time.overstep += dt;

    while fixed_time.overstep >= fixed_time.timestep {
        fixed_time.t += fixed_time.timestep;
        fixed_time.overstep -= fixed_time.timestep;
    }
}

#[derive(Component)]
pub struct FpsDisplay;

pub fn render_fps(time: Res<Time>, mut q_fps: Query<&mut Text, With<FpsDisplay>>) {
    let smoothed = time.get_smooth_avg().to_string();
    let min_fps = time.get_min_fps().to_string();

    for mut text in q_fps.iter_mut() {
        text.value = format!("{} (min {})", smoothed, min_fps);
    }
}

#[derive(Resource)]
pub struct TimeFixed {
    pub t: f32,
    pub timestep: f32,
    pub overstep: f32,
}

impl Default for TimeFixed {
    fn default() -> Self {
        Self {
            t: 0.,
            timestep: 1. / 60., // 60Hz
            overstep: 0.,
        }
    }
}

impl TimeFixed {
    #[inline]
    pub fn overstep_fraction(&self) -> f32 {
        self.overstep / self.timestep
    }
}
