use macroquad::prelude::*;
use bevy_ecs::prelude::*;

use super::alpha_blend;

const GLYPH_FRAGMENT_SHADER: &str = include_str!("../assets/shaders/glyph-shader.glsl");
const GLYPH_VERTEX_SHADER: &str = "#version 400
attribute vec3 position;
attribute vec2 texcoord;

varying lowp vec2 uv;

uniform mat4 Model;
uniform mat4 Projection;

void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    uv = texcoord;
}
";

#[derive(Resource)]
pub struct GlyphMaterial(pub Material);

impl FromWorld for GlyphMaterial {
    fn from_world(_: &mut World) -> Self {
        let mat = load_material(
            ShaderSource::Glsl {
                vertex: GLYPH_VERTEX_SHADER,
                fragment: GLYPH_FRAGMENT_SHADER,
            },
            MaterialParams {
                uniforms: vec![
                    UniformDesc::new("fg1", UniformType::Float4),
                    UniformDesc::new("fg2", UniformType::Float4),
                    UniformDesc::new("bg", UniformType::Float4),
                    UniformDesc::new("outline", UniformType::Float4),
                    UniformDesc::new("idx", UniformType::Float1),
                ],
                pipeline_params: PipelineParams {
                    color_blend: Some(alpha_blend()),
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .unwrap();

        GlyphMaterial(mat)
    }
}
