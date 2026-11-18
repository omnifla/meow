use super::Painter;
use crate::base::{Rect, Size};
use crate::style::Color;

const SHADER: &str = include_str!("rect.wgsl");
const FLOATS_PER_VERTEX: usize = 6;
const INITIAL_CAPACITY: u64 = 4096;

pub struct WgpuRenderer {
    pipeline: wgpu::RenderPipeline,
    buffer: wgpu::Buffer,
    capacity: u64,
    vertices: Vec<f32>,
    viewport: Size,
}

impl WgpuRenderer {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("meow rect"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("meow rect"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: (FLOATS_PER_VERTEX * 4) as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x4],
                })],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        Self {
            pipeline,
            buffer: Self::create_buffer(device, INITIAL_CAPACITY),
            capacity: INITIAL_CAPACITY,
            vertices: Vec::new(),
            viewport: Size::default(),
        }
    }

    pub fn begin(&mut self, viewport: Size) {
        self.vertices.clear();
        self.viewport = viewport;
    }

    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        clear: Color,
    ) {
        let bytes: Vec<u8> = self.vertices.iter().flat_map(|v| v.to_ne_bytes()).collect();
        let needed = bytes.len() as u64;

        if needed > self.capacity {
            self.capacity = needed.next_power_of_two();
            self.buffer = Self::create_buffer(device, self.capacity);
        }
        if needed > 0 {
            queue.write_buffer(&self.buffer, 0, &bytes);
        }

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("meow"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: clear.r as f64,
                        g: clear.g as f64,
                        b: clear.b as f64,
                        a: clear.a as f64,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        if needed > 0 {
            pass.set_pipeline(&self.pipeline);
            pass.set_vertex_buffer(0, self.buffer.slice(0..needed));
            pass.draw(0..(self.vertices.len() / FLOATS_PER_VERTEX) as u32, 0..1);
        }
    }

    fn create_buffer(device: &wgpu::Device, size: u64) -> wgpu::Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("meow vertices"),
            size,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }
}

impl Painter for WgpuRenderer {
    fn fill_rect(&mut self, rect: Rect, color: Color) {
        let Size { width, height } = self.viewport;
        let x0 = rect.origin.x / width * 2.0 - 1.0;
        let y0 = 1.0 - rect.origin.y / height * 2.0;
        let x1 = (rect.origin.x + rect.size.width) / width * 2.0 - 1.0;
        let y1 = 1.0 - (rect.origin.y + rect.size.height) / height * 2.0;

        for (x, y) in [(x0, y0), (x1, y0), (x0, y1), (x1, y0), (x1, y1), (x0, y1)] {
            self.vertices
                .extend_from_slice(&[x, y, color.r, color.g, color.b, color.a]);
        }
    }
}
