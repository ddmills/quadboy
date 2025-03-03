use macroquad::prelude::*;
use macroquad::miniquad::*;

use super::Renderable;

pub struct Stage {
    pub pipeline: Pipeline,
    pub bindings: Bindings,

    vertices: Vec<Vertex>,
    pub indices: Vec<u32>,

    // vbuffer: BufferId,
    // ibuffer: BufferId,

    pub i: u32,
}

#[repr(C)]
struct Vertex {
    pos: Vec2,
    uv: Vec2,
    tile_idx: f32,
}

#[repr(C)]
pub struct BaseShaderUniforms {
    pub projection: Mat4,
}

impl Stage {
    pub fn add(&mut self, r: Renderable) {
        let x = r.x;
        let y = r.y;
        let w = r.w;
        let h = r.h;
        let tile_idx = r.idx as f32;

        self.vertices.push(Vertex {// top left
            pos : Vec2::new(x, y),
            uv: Vec2::new(0., 0.),
            tile_idx,
        });
        self.vertices.push(Vertex { // top right
            pos : Vec2::new(x + w, y),
            uv: Vec2::new(1.0, 0.),
            tile_idx,
        });
        self.vertices.push(Vertex { // bottom right
            pos : Vec2::new(x + w, y + h),
            uv: Vec2::new(1., 1.),
            tile_idx,
        });
        self.vertices.push(Vertex { // bottom left
            pos : Vec2::new(x, y + h),
            uv: Vec2::new(0., 1.),
            tile_idx,
        });

        self.indices.push(self.i);
        self.indices.push(self.i + 1);
        self.indices.push(self.i + 2);
        self.indices.push(self.i);
        self.indices.push(self.i + 2);
        self.indices.push(self.i + 3);

        self.i += 4; // increment index counter

    }

    pub fn update_buffers(&mut self, ctx: &mut dyn RenderingBackend)
    {
        let vbuffer = ctx.new_buffer(
            BufferType::VertexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(self.vertices.as_slice()),
        );
        let ibuffer = ctx.new_buffer(
            BufferType::IndexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(self.indices.as_slice()),
        );

        self.bindings = Bindings {
            vertex_buffers: vec![vbuffer],
            index_buffer: ibuffer,
            images: self.bindings.images.clone(),
        };
    }

    pub fn new(ctx: &mut dyn RenderingBackend, texture: TextureId) -> Stage {
        let shader = ctx
            .new_shader(
                miniquad::ShaderSource::Glsl {
                    vertex: VERTEX,
                    fragment: FRAGMENT,
                },
                ShaderMeta {
                    images: vec!["tex".to_string()],
                    uniforms: UniformBlockLayout {
                        uniforms: vec![
                            UniformDesc::new("projection", UniformType::Mat4)
                        ],
                    },
                },
            )
            .unwrap();

        let pipeline = ctx.new_pipeline(
            &[
                BufferLayout::default(),
            ],
            &[
                VertexAttribute::new("in_pos", VertexFormat::Float2),
                VertexAttribute::new("in_uv", VertexFormat::Float2),
                VertexAttribute::new("in_idx", VertexFormat::Float1),
            ],
            shader,
            Default::default(),
        );

        let vbuffer = ctx.new_buffer(
            BufferType::VertexBuffer,
            BufferUsage::Dynamic,
            BufferSource::empty::<Vertex>(0),
        );
        let ibuffer = ctx.new_buffer(
            BufferType::IndexBuffer,
            BufferUsage::Dynamic,
            BufferSource::empty::<u32>(0),
        );
        
        let bindings = Bindings {
            vertex_buffers: vec![vbuffer],
            index_buffer: ibuffer,
            images: vec![texture],
        };

        Stage {
            pipeline,
            bindings,
            vertices: vec![],
            indices: vec![],
            i: 0,
        }
    }
}

pub const VERTEX: &str = "#version 400
attribute vec2 in_pos;
attribute vec2 in_uv;
attribute float in_idx;

uniform mat4 projection;

varying lowp vec2 uv;
varying lowp float idx;

void main() {
    gl_Position = projection * vec4(in_pos, 0, 1);
    uv = in_uv;
    idx = in_idx;
}";

pub const FRAGMENT: &str = "#version 400
precision lowp float;

varying lowp vec2 uv;
varying float idx;

uniform sampler2D tex;

void main() {
    vec2 uv_scaled = uv / 16.0; // atlas is 16x16
    float x = float(uint(idx) % 16u);
    float y = float(uint(idx) / 16u);
    vec2 uv_offset = vec2(x, y) / 16.0;

    vec2 tex_uv = uv_offset + uv_scaled;

    vec4 v = texture2D(tex, tex_uv);

    gl_FragColor = v;
}";
