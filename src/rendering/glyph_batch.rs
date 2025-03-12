use bevy_ecs::component::Component;
use macroquad::prelude::*;
use macroquad::miniquad::*;

use super::get_render_target_size;
use super::Renderable;

struct Stage {
    pipeline: Pipeline,
    bindings: Bindings,
}

#[derive(Component)]
pub struct GlyphBatch {
    texture_id: TextureId,
    stage: Option<Stage>,

    vertices: Vec<Vertex>,
    indices: Vec<u32>,

    max_size: usize,
    size: usize,
}

#[repr(C)]
struct Vertex {
    pos: Vec2,
    uv: Vec2,
    idx: f32,
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
    fn update_bindings(&mut self, ctx: &mut dyn RenderingBackend)
    {
        let vsource = BufferSource::slice(self.vertices.as_slice());
        let isource = BufferSource::slice(self.indices.as_slice());
        let n_elements = self.vertices.len() >> 2;

        if let Some(stage) = self.stage.as_mut() {
            if n_elements > self.size {
                ctx.delete_buffer(stage.bindings.vertex_buffers[0]);
                ctx.delete_buffer(stage.bindings.index_buffer);
                stage.bindings = Bindings {
                    vertex_buffers: vec![ctx.new_buffer(
                        BufferType::VertexBuffer,
                        BufferUsage::Immutable,
                        vsource,
                    )],
                    index_buffer: ctx.new_buffer(
                        BufferType::IndexBuffer,
                        BufferUsage::Immutable,
                        isource,
                    ),
                    images: vec![self.texture_id],
                };
                self.size = n_elements;
            } else {
                ctx.buffer_update(stage.bindings.vertex_buffers[0], vsource);
                ctx.buffer_update(stage.bindings.index_buffer, isource);
            }
        } else {
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
                    VertexAttribute::new("in_fg1", VertexFormat::Float4),
                    VertexAttribute::new("in_fg2", VertexFormat::Float4),
                    VertexAttribute::new("in_bg", VertexFormat::Float4),
                    VertexAttribute::new("in_outline", VertexFormat::Float4),
                ],
                shader,
                Default::default(),
            );

            self.size = n_elements;

            let bindings = Bindings {
                vertex_buffers: vec![ctx.new_buffer(
                    BufferType::VertexBuffer,
                    BufferUsage::Immutable,
                    BufferSource::slice(self.vertices.as_slice()),
                )],
                index_buffer: ctx.new_buffer(
                    BufferType::IndexBuffer,
                    BufferUsage::Immutable,
                    BufferSource::slice(self.indices.as_slice()),
                ),
                images: vec![self.texture_id],
            };

            self.stage = Some(Stage {
                bindings,
                pipeline,
            });
        }
    }

    pub fn new(texture_id: TextureId, max_size: usize) -> GlyphBatch {
        let v_count = 4 * max_size;
        let i_count = ((v_count + 3) >> 2) * 6;

        let vertices = Vec::<Vertex>::with_capacity(v_count);
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

        GlyphBatch {
            texture_id,
            stage: None,
            vertices,
            indices,
            max_size,
            size: 0,
        }
    }

    pub fn set_glyphs<I>(&mut self, renderables: I) where I: Iterator<Item = Renderable> {
        self.vertices.clear();

        for (i, r) in renderables.enumerate() {
            if i >= self.max_size {
                trace!("LIMIT REACHED");
                break;
            }

            self.vertices.push(Vertex {// top left
                pos : Vec2::new(r.x, r.y),
                uv: Vec2::new(0., 0.),
                idx: r.idx as f32,
                fg1: r.fg1,
                fg2: r.fg2,
                bg: r.bg,
                outline: r.outline,
            });
            self.vertices.push(Vertex { // top right
                pos : Vec2::new(r.x + r.w, r.y),
                uv: Vec2::new(1.0, 0.),
                idx: r.idx as f32,
                fg1: r.fg1,
                fg2: r.fg2,
                bg: r.bg,
                outline: r.outline,
            });
            self.vertices.push(Vertex { // bottom right
                pos : Vec2::new(r.x + r.w, r.y + r.h),
                uv: Vec2::new(1., 1.),
                idx: r.idx as f32,
                fg1: r.fg1,
                fg2: r.fg2,
                bg: r.bg,
                outline: r.outline,
            });
            self.vertices.push(Vertex { // bottom left
                pos : Vec2::new(r.x, r.y + r.h),
                uv: Vec2::new(0., 1.),
                idx: r.idx as f32,
                fg1: r.fg1,
                fg2: r.fg2,
                bg: r.bg,
                outline: r.outline,
            });
        };
    }

    pub fn render(&mut self) {
        let mut gl = unsafe { get_internal_gl() };

        self.update_bindings(gl.quad_context);

        let Some(ref stage) = self.stage else {
            return;
        };

        gl.flush();

        gl.quad_context.apply_pipeline(&stage.pipeline);
        gl.quad_context.apply_bindings(&stage.bindings);

        let target_size = get_render_target_size().as_vec2();
        let projection = Mat4::orthographic_rh_gl(0., target_size.x, target_size.y, 0., 0., 1.);

        gl.quad_context.apply_uniforms(UniformsSource::table(
            &BaseShaderUniforms {
                projection,
            },
        ));

        let n = ((self.vertices.len() + 3) >> 2) * 6;

        gl.quad_context.draw(0, n as i32, 1);
    }
}

pub const VERTEX: &str = "#version 400
uniform mat4 projection;

attribute float in_idx;
attribute vec2 in_pos;
attribute vec2 in_uv;
attribute vec4 in_fg1;
attribute vec4 in_fg2;
attribute vec4 in_bg;
attribute vec4 in_outline;

varying lowp float idx;
varying lowp vec2 uv;
varying vec4 fg1;
varying vec4 fg2;
varying vec4 bg;
varying vec4 outline;

void main() {
    gl_Position = projection * vec4(in_pos, 0, 1);
    uv = in_uv;
    idx = in_idx;
    fg1 = in_fg1;
    fg2 = in_fg2;
    bg = in_bg;
    outline = in_outline;
}";

pub const FRAGMENT: &str = "#version 400
precision lowp float;

varying lowp vec2 uv;
varying float idx;
varying vec4 fg1;
varying vec4 fg2;
varying vec4 bg;
varying vec4 outline;

uniform sampler2D tex;

void main() {
    vec2 uv_scaled = uv / 16.0; // atlas is 16x16
    float x = float(uint(idx) % 16u);
    float y = float(uint(idx) / 16u);
    vec2 uv_offset = vec2(x, y) / 16.0;

    vec2 tex_uv = uv_offset + uv_scaled;

    vec4 v = texture2D(tex, tex_uv);

    if (v.a == 0) { // transparent (background)
        gl_FragColor.a = 0.0;
    } else if (v.r == 0 && v.g == 0 && v.b == 0 && fg1.a > 0) { // Black (Primary)
        gl_FragColor = fg1;
    } else if (v.r == 1 && v.g == 1 && v.b == 1 && fg2.a > 0) { // White (Secondary)
        gl_FragColor = fg2;
    } else if (v.r == 1 && v.g == 0 && v.b == 0 && outline.a > 0) { // Red (Outline)
        gl_FragColor = outline;
    } else { // debug
        gl_FragColor = bg;
        // gl_FragColor = vec4(1.0, 1.0, 0.0, 1.0);
    }

    // if (gl_FragColor.a == 0) {
    //     discard;
    // }
}";
