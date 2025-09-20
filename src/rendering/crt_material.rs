use bevy_ecs::{prelude::*, system::ResMut};
use macroquad::prelude::*;

use crate::domain::GameSettings;

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
                    UniformDesc::new("u_chromatic_ab", UniformType::Int1),
                ],
                ..Default::default()
            },
        )
        .unwrap();

        CrtShader { mat }
    }
}

pub fn update_crt_uniforms(crt: ResMut<CrtShader>, settings: Res<GameSettings>) {
    crate::tracy_span!("update_crt_uniforms");
    if settings.is_changed() {
        let curve_values = settings.crt_curvature.get_values();
        crt.mat
            .set_uniform("u_crt_curve", vec2(curve_values.0, curve_values.1));
        crt.mat.set_uniform(
            "u_crt",
            if settings.crt_curvature.is_enabled() {
                1
            } else {
                0
            },
        );
        crt.mat
            .set_uniform("u_scanline", if settings.crt_scanline { 1 } else { 0 });
        crt.mat
            .set_uniform("u_film_grain", if settings.crt_film_grain { 1 } else { 0 });
        crt.mat
            .set_uniform("u_flicker", if settings.crt_flicker { 1 } else { 0 });
        crt.mat
            .set_uniform("u_vignette", if settings.crt_vignette { 1 } else { 0 });
        crt.mat.set_uniform(
            "u_chromatic_ab",
            if settings.crt_chromatic_ab { 1 } else { 0 },
        );
    }
}
