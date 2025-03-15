use bevy_ecs::prelude::*;

#[derive(Resource, Default)]
pub struct GameCamera {
    pub x: f32,
    pub y: f32,
}
