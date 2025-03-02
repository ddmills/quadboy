use bevy_ecs::{component::Component, system::Query};
use macroquad::shapes::draw_rectangle;

use crate::{cfg::TILE_SIZE_F32, rendering::{get_render_offset, Position}};

use super::MacroquadColorable;


#[derive(Component)]
pub struct Rectangle {
    width: usize,
    height: usize,
    color: u32,
}

impl Rectangle {
    pub fn new<T: Into<u32>>(width: usize, height: usize, color: T) -> Self {
        Self {
            width,
            height,
            color: color.into(),
        }
    }
}

pub fn render_shapes(q_shapes: Query<(&Rectangle, &Position)>)
{
    let offset = get_render_offset();
    for (shape, position) in q_shapes.iter() {
        draw_rectangle(
            (position.x * TILE_SIZE_F32.0) + offset.x,
            (position.y * TILE_SIZE_F32.1) + offset.y,
            shape.width as f32,
            shape.height as f32,
            shape.color.to_macroquad_color(),
        );
    }
}
