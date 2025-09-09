use bevy_ecs::prelude::*;
use macroquad::prelude::*;

#[derive(Resource)]
pub struct AmbientTransition {
    current_ambient: Vec4,
    target_ambient: Vec4,
    transition_progress: f32,
    transition_duration: f32,
}

impl Default for AmbientTransition {
    fn default() -> Self {
        let default_ambient = Vec4::new(0.1, 0.07, 0.1, 0.1); // Default dark ambient
        Self {
            current_ambient: default_ambient,
            target_ambient: default_ambient,
            transition_progress: 1.0,
            transition_duration: 0.25,
        }
    }
}

impl AmbientTransition {
    pub fn start_transition(&mut self, target_ambient: Vec4) {
        if (self.target_ambient - target_ambient).length() < 0.001 {
            return; // Skip if target is essentially the same
        }

        self.current_ambient = self.get_interpolated_ambient();
        self.target_ambient = target_ambient;
        self.transition_progress = 0.0;
    }

    pub fn update(&mut self, dt: f32) {
        if self.transition_progress < 1.0 {
            self.transition_progress += dt / self.transition_duration;
            self.transition_progress = self.transition_progress.min(1.0);
        }
    }

    pub fn get_interpolated_ambient(&self) -> Vec4 {
        if self.transition_progress >= 1.0 {
            return self.target_ambient;
        }

        // Linear interpolation
        let t = self.transition_progress;
        self.current_ambient + (self.target_ambient - self.current_ambient) * t
    }
}
