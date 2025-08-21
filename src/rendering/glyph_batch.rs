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

    vertices: Vec<Vertex>,
    indices: Vec<u32>,

    max_size: usize,
    size: usize,
}

#[derive(Default)]
#[repr(C)]
struct Vertex {
    pos: Vec2,
    uv: Vec2,
    idx: f32,
    tex_idx: f32,
    fg1: Vec4,
    fg2: Vec4,
    bg: Vec4,
    outline: Vec4,
}

#[repr(C)]
pub struct BaseShaderUniforms {
    pub projection: Mat4,
}

impl GlyphBatch {
    fn update_bindings(&mut self, ctx: &mut dyn RenderingBackend) {
        let vsource = BufferSource::slice(self.vertices.as_slice());
        let isource = BufferSource::slice(self.indices.as_slice());

        ctx.buffer_update(self.bindings.vertex_buffers[0], vsource);
        ctx.buffer_update(self.bindings.index_buffer, isource);
    }

    pub fn new(
        texture_id_1: TextureId,
        texture_id_2: TextureId,
        target_type: RenderTargetType,
        max_size: usize,
    ) -> GlyphBatch {
        let v_count = 4 * max_size;
        let i_count = ((v_count + 3) >> 2) * 6;

        let mut vertices = Vec::<Vertex>::with_capacity(v_count);
        let mut indices = Vec::<u32>::with_capacity(i_count);

        for i in 0..(i_count / 6) {
            let k = (i << 2) as u32;
            indices.push(k);
            indices.push(k + 1);
            indices.push(k + 2);
            indices.push(k);
            indices.push(k + 2);
            indices.push(k + 3);
        }

        for _ in 0..v_count {
            vertices.push(Vertex::default());
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
                        uniforms: vec![UniformDesc::new("projection", UniformType::Mat4)],
                    },
                },
            )
            .unwrap();

        let pipeline = ctx.new_pipeline(
            &[BufferLayout::default()],
            &[
                VertexAttribute::new("in_pos", VertexFormat::Float2),
                VertexAttribute::new("in_uv", VertexFormat::Float2),
                VertexAttribute::new("in_idx", VertexFormat::Float1),
                VertexAttribute::new("in_tex_idx", VertexFormat::Float1),
                VertexAttribute::new("in_fg1", VertexFormat::Float4),
                VertexAttribute::new("in_fg2", VertexFormat::Float4),
                VertexAttribute::new("in_bg", VertexFormat::Float4),
                VertexAttribute::new("in_outline", VertexFormat::Float4),
            ],
            shader,
            Default::default(),
        );

        let bindings = Bindings {
            vertex_buffers: vec![ctx.new_buffer(
                BufferType::VertexBuffer,
                BufferUsage::Immutable,
                BufferSource::slice(vertices.as_slice()),
            )],
            index_buffer: ctx.new_buffer(
                BufferType::IndexBuffer,
                BufferUsage::Immutable,
                BufferSource::slice(indices.as_slice()),
            ),
            images: vec![texture_id_1, texture_id_2],
        };

        GlyphBatch {
            target_type,
            bindings,
            pipeline,
            vertices,
            indices,
            max_size,
            size: 0,
        }
    }

    pub fn add(&mut self, r: Renderable) {
        if self.size >= self.max_size {
            trace!("GlyphBatch limit reached {}", self.size);
            return;
        }

        self.size += 1;

        self.vertices.push(Vertex {
            // top left
            pos: Vec2::new(r.x, r.y),
            uv: Vec2::new(0., 0.),
            idx: r.idx as f32,
            tex_idx: r.tex_idx as f32,
            fg1: r.fg1,
            fg2: r.fg2,
            bg: r.bg,
            outline: r.outline,
        });
        self.vertices.push(Vertex {
            // top right
            pos: Vec2::new(r.x + r.w, r.y),
            uv: Vec2::new(1.0, 0.),
            idx: r.idx as f32,
            tex_idx: r.tex_idx as f32,
            fg1: r.fg1,
            fg2: r.fg2,
            bg: r.bg,
            outline: r.outline,
        });
        self.vertices.push(Vertex {
            // bottom right
            pos: Vec2::new(r.x + r.w, r.y + r.h),
            uv: Vec2::new(1., 1.),
            idx: r.idx as f32,
            tex_idx: r.tex_idx as f32,
            fg1: r.fg1,
            fg2: r.fg2,
            bg: r.bg,
            outline: r.outline,
        });
        self.vertices.push(Vertex {
            // bottom left
            pos: Vec2::new(r.x, r.y + r.h),
            uv: Vec2::new(0., 1.),
            idx: r.idx as f32,
            tex_idx: r.tex_idx as f32,
            fg1: r.fg1,
            fg2: r.fg2,
            bg: r.bg,
            outline: r.outline,
        });
    }

    pub fn clear(&mut self) {
        self.vertices.clear();
        self.size = 0;
    }

    pub fn render(&mut self) {
        let mut gl = unsafe { get_internal_gl() };

        self.update_bindings(gl.quad_context);

        gl.flush();

        gl.quad_context.apply_pipeline(&self.pipeline);
        gl.quad_context.apply_bindings(&self.bindings);

        let target_size = get_render_target_size().as_vec2();
        let projection = Mat4::orthographic_rh_gl(0., target_size.x, target_size.y, 0., 0., 1.);

        gl.quad_context
            .apply_uniforms(UniformsSource::table(&BaseShaderUniforms { projection }));

        let n = self.size * 6;

        gl.quad_context.draw(0, n as i32, 1);
        gl.flush();
    }
}

pub const VERTEX: &str = "#version 100
uniform mat4 projection;

attribute float in_idx;
attribute float in_tex_idx;
attribute vec2 in_pos;
attribute vec2 in_uv;
attribute vec4 in_fg1;
attribute vec4 in_fg2;
attribute vec4 in_bg;
attribute vec4 in_outline;

varying lowp float idx;
varying lowp float tex_idx;
varying lowp vec2 uv;
varying vec4 fg1;
varying vec4 fg2;
varying vec4 bg;
varying vec4 outline;

void main() {
    gl_Position = projection * vec4(in_pos, 0.0, 1.0);
    uv = in_uv;
    idx = in_idx;
    tex_idx = in_tex_idx;
    fg1 = in_fg1;
    fg2 = in_fg2;
    bg = in_bg;
    outline = in_outline;
}";

const FRAGMENT: &str = include_str!("../assets/shaders/glyph-shader.glsl");
