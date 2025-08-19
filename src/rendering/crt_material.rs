use bevy_ecs::resource::Resource;
use macroquad::prelude::*;

#[derive(Debug, Clone, Copy)]
pub enum CrtCurvature {
    Off,
    Curve(f32, f32),
}

impl CrtCurvature {
    pub fn is_enabled(&self) -> bool {
        !matches!(self, CrtCurvature::Off)
    }

    pub fn get_values(&self) -> (f32, f32) {
        match self {
            CrtCurvature::Off => (0.0, 0.0),
            CrtCurvature::Curve(x, y) => (*x, *y),
        }
    }
}

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
                    UniformDesc::new("u_resolution", UniformType::Float2),
                    UniformDesc::new("u_time", UniformType::Float1),
                    UniformDesc::new("u_crt_curve", UniformType::Float2),
                    UniformDesc::new("u_crt", UniformType::Int1),
                    UniformDesc::new("u_scanline", UniformType::Int1),
                    UniformDesc::new("u_film_grain", UniformType::Int1),
                    UniformDesc::new("u_flicker", UniformType::Int1),
                    UniformDesc::new("u_vignette", UniformType::Int1),
                ],
                ..Default::default()
            },
        )
        .unwrap();

        CrtShader { mat }
    }
}
