use bevy_ecs::resource::Resource;
use macroquad::prelude::*;

const CRT_FRAGMENT_SHADER: &str = include_str!("../assets/shaders/crt-shader.glsl");
const CRT_VERTEX_SHADER: &str = "#version 100
precision highp float;

attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;

varying vec2 uv;
varying vec4 color;

uniform mat4 Model;
uniform mat4 Projection;

void main() {
    gl_Position = Projection * Model * vec4(position, 1.0);
    color = color0 / 255.0;
    uv = texcoord;
}
";

#[derive(Resource)]
pub struct CrtShader {
    pub mat: Material,
}

impl Default for CrtShader {
    fn default() -> Self {
        let mat = load_material(
            ShaderSource::Glsl {
                vertex: CRT_VERTEX_SHADER,
                fragment: CRT_FRAGMENT_SHADER,
            },
            MaterialParams {
                uniforms: vec![
                    UniformDesc::new("iResolution", UniformType::Float2),
                    UniformDesc::new("iTime", UniformType::Float1),
                ],
                ..Default::default()
            },
        )
        .unwrap();

        CrtShader { mat }
    }
}
