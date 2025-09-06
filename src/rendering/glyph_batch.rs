use bevy_ecs::component::Component;
use macroquad::miniquad::*;
use macroquad::prelude::*;

use crate::rendering::RenderTargetType;

use super::Renderable;
use super::get_render_target_size;

#[derive(Component)]
pub struct GlyphBatch {
    pub target_type: RenderTargetType,

    pipeline: Pipeline,
    bindings: Bindings,

    quad_vertices: [Vertex; 4],
    quad_indices: [u32; 6],
    instances: Vec<InstanceData>,

    max_size: usize,
    size: usize,
}

#[derive(Default)]
#[repr(C)]
struct Vertex {
    pos: Vec2,
    uv: Vec2,
}

#[derive(Default)]
#[repr(C)]
struct InstanceData {
    instance_pos: Vec2,
    instance_size: Vec2,
    idx: f32,
    tex_idx: f32,
    fg1: Vec4,
    fg2: Vec4,
    bg: Vec4,
    outline: Vec4,
    is_shrouded: f32,
    light_rgba: Vec4,
    light_flicker: f32,
    ignore_lighting: f32,
}

#[repr(C)]
pub struct BaseShaderUniforms {
    pub projection: Mat4,
    pub time: f32,
    pub ambient: Vec4,
}

impl GlyphBatch {
    fn update_bindings(&mut self, ctx: &mut dyn RenderingBackend) {
        let used_instances = self.size;
        let vsource = BufferSource::slice(&self.quad_vertices);
        let isource = BufferSource::slice(&self.quad_indices);
        let instance_source = BufferSource::slice(&self.instances[..used_instances]);

        ctx.buffer_update(self.bindings.vertex_buffers[0], vsource);
        ctx.buffer_update(self.bindings.vertex_buffers[1], instance_source);
        ctx.buffer_update(self.bindings.index_buffer, isource);
    }

    pub fn new(
        texture_id_1: TextureId,
        texture_id_2: TextureId,
        target_type: RenderTargetType,
        max_size: usize,
    ) -> GlyphBatch {
        let quad_vertices = [
            Vertex {
                pos: Vec2::new(0.0, 0.0),
                uv: Vec2::new(0.0, 0.0),
            }, // top left
            Vertex {
                pos: Vec2::new(1.0, 0.0),
                uv: Vec2::new(1.0, 0.0),
            }, // top right
            Vertex {
                pos: Vec2::new(1.0, 1.0),
                uv: Vec2::new(1.0, 1.0),
            }, // bottom right
            Vertex {
                pos: Vec2::new(0.0, 1.0),
                uv: Vec2::new(0.0, 1.0),
            }, // bottom left
        ];

        let quad_indices = [0, 1, 2, 0, 2, 3];

        let mut instances = Vec::<InstanceData>::with_capacity(max_size);
        for _ in 0..max_size {
            instances.push(InstanceData::default());
        }

        let ctx = unsafe { get_internal_gl().quad_context };

        let shader = ctx
            .new_shader(
                miniquad::ShaderSource::Glsl {
                    vertex: VERTEX,
                    fragment: FRAGMENT,
                },
                ShaderMeta {
                    images: vec!["tex_1".to_string(), "tex_2".to_string()],
                    uniforms: UniformBlockLayout {
                        uniforms: vec![
                            UniformDesc::new("projection", UniformType::Mat4),
                            UniformDesc::new("time", UniformType::Float1),
                            UniformDesc::new("ambient", UniformType::Float4),
                        ],
                    },
                },
            )
            .unwrap();

        let pipeline = ctx.new_pipeline(
            &[
                BufferLayout {
                    step_func: VertexStep::PerVertex,
                    step_rate: 1,
                    stride: std::mem::size_of::<Vertex>() as i32,
                },
                BufferLayout {
                    step_func: VertexStep::PerInstance,
                    step_rate: 1,
                    stride: std::mem::size_of::<InstanceData>() as i32,
                },
            ],
            &[
                // Vertex attributes (per vertex) - buffer 0
                VertexAttribute::with_buffer("in_pos", VertexFormat::Float2, 0),
                VertexAttribute::with_buffer("in_uv", VertexFormat::Float2, 0),
                // Instance attributes (per instance) - buffer 1
                VertexAttribute::with_buffer("in_instance_pos", VertexFormat::Float2, 1),
                VertexAttribute::with_buffer("in_instance_size", VertexFormat::Float2, 1),
                VertexAttribute::with_buffer("in_idx", VertexFormat::Float1, 1),
                VertexAttribute::with_buffer("in_tex_idx", VertexFormat::Float1, 1),
                VertexAttribute::with_buffer("in_fg1", VertexFormat::Float4, 1),
                VertexAttribute::with_buffer("in_fg2", VertexFormat::Float4, 1),
                VertexAttribute::with_buffer("in_bg", VertexFormat::Float4, 1),
                VertexAttribute::with_buffer("in_outline", VertexFormat::Float4, 1),
                VertexAttribute::with_buffer("in_is_shrouded", VertexFormat::Float1, 1),
                VertexAttribute::with_buffer("in_light_rgba", VertexFormat::Float4, 1),
                VertexAttribute::with_buffer("in_light_flicker", VertexFormat::Float1, 1),
                VertexAttribute::with_buffer("in_ignore_lighting", VertexFormat::Float1, 1),
            ],
            shader,
            Default::default(),
        );

        let bindings = Bindings {
            vertex_buffers: vec![
                ctx.new_buffer(
                    BufferType::VertexBuffer,
                    BufferUsage::Immutable,
                    BufferSource::slice(&quad_vertices),
                ),
                ctx.new_buffer(
                    BufferType::VertexBuffer,
                    BufferUsage::Stream,
                    BufferSource::slice(instances.as_slice()),
                ),
            ],
            index_buffer: ctx.new_buffer(
                BufferType::IndexBuffer,
                BufferUsage::Immutable,
                BufferSource::slice(&quad_indices),
            ),
            images: vec![texture_id_1, texture_id_2],
        };

        GlyphBatch {
            target_type,
            bindings,
            pipeline,
            quad_vertices,
            quad_indices,
            instances,
            max_size,
            size: 0,
        }
    }

    pub fn add(&mut self, r: Renderable) {
        if self.size >= self.max_size {
            trace!("GlyphBatch limit reached {}", self.size);
            return;
        }

        self.instances[self.size] = InstanceData {
            instance_pos: Vec2::new(r.x, r.y),
            instance_size: Vec2::new(r.w, r.h),
            idx: r.idx as f32,
            tex_idx: r.tex_idx as f32,
            fg1: r.fg1,
            fg2: r.fg2,
            bg: r.bg,
            outline: r.outline,
            is_shrouded: r.is_shrouded as f32,
            light_rgba: r.light_rgba,
            light_flicker: r.light_flicker,
            ignore_lighting: r.ignore_lighting,
        };

        self.size += 1;
    }

    pub fn clear(&mut self) {
        self.size = 0;
    }

    pub fn render(&mut self, time: f32, ambient: Vec4) {
        if self.size == 0 {
            return;
        }

        let mut gl = unsafe { get_internal_gl() };

        self.update_bindings(gl.quad_context);

        gl.flush();

        gl.quad_context.apply_pipeline(&self.pipeline);
        gl.quad_context.apply_bindings(&self.bindings);

        let target_size = get_render_target_size().as_vec2();
        let projection = Mat4::orthographic_rh_gl(0., target_size.x, target_size.y, 0., 0., 1.);

        gl.quad_context
            .apply_uniforms(UniformsSource::table(&BaseShaderUniforms {
                projection,
                time,
                ambient,
            }));

        gl.quad_context.draw(0, 6, self.size as i32);
        gl.flush();
    }
}

pub const VERTEX: &str = "#version 100
uniform mat4 projection;

// Vertex attributes (per vertex)
attribute vec2 in_pos;
attribute vec2 in_uv;

// Instance attributes (per glyph)
attribute vec2 in_instance_pos;
attribute vec2 in_instance_size;
attribute float in_idx;
attribute float in_tex_idx;
attribute vec4 in_fg1;
attribute vec4 in_fg2;
attribute vec4 in_bg;
attribute vec4 in_outline;
attribute float in_is_shrouded;
attribute vec4 in_light_rgba;
attribute float in_light_flicker;
attribute float in_ignore_lighting;

varying lowp float idx;
varying lowp float tex_idx;
varying lowp vec2 uv;
varying vec4 fg1;
varying vec4 fg2;
varying vec4 bg;
varying vec4 outline;
varying lowp float is_shrouded;
varying vec4 light_rgba;
varying float light_flicker;
varying float ignore_lighting;

void main() {
    // Calculate world position by scaling unit quad and translating
    vec2 world_pos = in_instance_pos + in_pos * in_instance_size;
    gl_Position = projection * vec4(world_pos, 0.0, 1.0);
    
    uv = in_uv;
    idx = in_idx;
    tex_idx = in_tex_idx;
    fg1 = in_fg1;
    fg2 = in_fg2;
    bg = in_bg;
    outline = in_outline;
    is_shrouded = in_is_shrouded;
    light_rgba = in_light_rgba;
    light_flicker = in_light_flicker;
    ignore_lighting = in_ignore_lighting;
}";

const FRAGMENT: &str = include_str!("../assets/shaders/glyph-shader.glsl");
