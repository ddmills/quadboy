use bevy_ecs::prelude::*;
use macroquad::prelude::*;

use super::{get_render_offset, GlyphMaterial, TilesetId, TilesetTextures};

pub struct Renderable {
    pub idx: usize,
    pub fg1: Color,
    pub fg2: Color,
    pub bg: Color,
    pub outline: Color,
    pub tileset_id: TilesetId,
    pub x: f32,
    pub y: f32,
}

#[derive(Resource, Default)]
pub struct Renderer {
    stack: Vec<Renderable>
}

impl Renderer {
    pub fn draw(&mut self, renderable: Renderable) {
        self.stack.push(renderable);
    }
}

pub fn render_all(
    mut renderer: ResMut<Renderer>,
    material: Res<GlyphMaterial>,
    tilesets: Res<TilesetTextures>,
) {
    gl_use_material(&material.0);

    let offset = get_render_offset();

    for r in renderer.stack.iter() {
        let texture = tilesets.get_by_id(&r.tileset_id);
        let size = tilesets.get_size(&r.tileset_id);

        material.0.set_uniform("fg1", r.fg1);
        material.0.set_uniform("fg2", r.fg2);
        material.0.set_uniform("outline", r.outline);
        material.0.set_uniform("bg", r.bg);
        material.0.set_uniform("idx", r.idx as f32);

        draw_texture_ex(
            texture,
            r.x + offset.x,
            r.y + offset.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(size),
                source: None,
                rotation: 0.,
                flip_x: false,
                flip_y: false,
                pivot: None,
            },
        );
    }

    renderer.stack.clear();
    gl_use_default_material();
}
