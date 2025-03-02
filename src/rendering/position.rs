use bevy_ecs::prelude::*;
use macroquad::prelude::*;

#[derive(Default, PartialEq)]
pub enum PositionSpace {
    #[default]
    World,
    Screen,
}

#[derive(Component)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

impl Position {
    pub fn new(x: usize, y: usize) -> Self {
        Self {
            x: x as f32,
            y: y as f32,
        }
    }

    pub fn screen(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
        }
    }

    pub fn new_f32(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
        }
    }
}
