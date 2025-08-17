use std::collections::VecDeque;

use bevy_ecs::prelude::*;
use macroquad::prelude::*;

use crate::rendering::Text;

#[derive(Resource)]
pub struct Time {
    pub dt: f32,
    pub fps: i32,
    pub elapsed: f64,
    pub frames: VecDeque<i32>,
    pub frames_count: usize,
    pub fixed_t: f32,
    pub fixed_overstep: f32,
    pub fixed_timestep: f32,
}

impl Default for Time {
    fn default() -> Self {
        Self {
            dt: 0.,
            fps: 0,
            elapsed: 0.,
            frames: VecDeque::new(),
            frames_count: 60,
            fixed_t: 0.,
            fixed_overstep: 0.,
            fixed_timestep: 1. / 60., // 60Hz
        }
    }
}

#[allow(dead_code)]
impl Time {
    pub fn get_smooth_avg(&self) -> i32 {
        self.frames.iter().sum::<i32>() / (self.frames_count as i32)
    }

    pub fn get_min_fps(&self) -> i32 {
        *self.frames.iter().min().unwrap_or(&0)
    }

    #[inline]
    pub fn overstep_fraction(&self) -> f32 {
        self.fixed_overstep / self.fixed_timestep
    }
}

pub fn update_time(mut time: ResMut<Time>) {
    let dt = get_frame_time();
    let fps = get_fps();

    time.dt = dt;
    time.fps = fps;
    time.elapsed = get_time();

    if time.frames.len() >= time.frames_count {
        time.frames.pop_front();
    }
    time.frames.push_back(fps);
    time.fixed_overstep += dt;

    while time.fixed_overstep >= time.fixed_timestep {
        time.fixed_t += time.fixed_timestep;
        time.fixed_overstep -= time.fixed_timestep;
    }
}

#[derive(Component)]
pub struct FpsDisplay;

pub fn render_fps(time: Res<Time>, mut q_fps: Query<&mut Text, With<FpsDisplay>>) {
    let smoothed = time.get_smooth_avg().to_string();

    for mut text in q_fps.iter_mut() {
        text.value = format!("{{R-O-Y-G-B-P stretch|QUADBOY}} {}", smoothed);
    }
}

#[allow(dead_code)]
pub fn render_profiler() {
    macroquad_profiler::profiler(macroquad_profiler::ProfilerParams {
        fps_counter_pos: vec2(0., 0.),
    });
}
