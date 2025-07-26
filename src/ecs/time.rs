use bevy_ecs::prelude::*;
use macroquad::prelude::*;

use crate::rendering::Text;

#[derive(Resource, Default)]
pub struct Time {
    pub dt: f32,
    pub fps: i32,
    pub elapsed: f64,
}

pub fn update_time(mut time: ResMut<Time>, mut fixed_time: ResMut<TimeFixed>) {
    time.dt = get_frame_time();
    time.fps = get_fps();
    time.elapsed = get_time();

    fixed_time.accumulate(time.dt);

    while fixed_time.overstep >= fixed_time.timestep {
        fixed_time.t += fixed_time.timestep;
        fixed_time.overstep -= fixed_time.timestep;
    }
}

#[derive(Component)]
pub struct FpsDisplay;

pub fn render_fps(time: Res<Time>, fixed_time: Res<TimeFixed>, mut q_fps: Query<&mut Text, With<FpsDisplay>>) {
    let binding_fps = time.fps.to_string();
    let binding_over = fixed_time.overstep_fraction().to_string();

    for mut text in q_fps.iter_mut() {
        text.value = format!("{} ({})", binding_fps, binding_over);
    }
}

#[derive(Resource)]
pub struct TimeFixed {
    pub t: f32,
    pub timestep: f32,
    pub overstep: f32,
}

impl TimeFixed {
    pub fn new() -> TimeFixed{
        TimeFixed {
            t: 0.,
            timestep: 1.,
            overstep: 0.,
        }
    }

    pub fn overstep_fraction(&self) -> f32 {
        self.overstep / self.timestep
    }

    pub fn accumulate(&mut self, dt: f32)
    {
        self.overstep += dt;
    }
}
