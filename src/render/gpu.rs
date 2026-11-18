use std::collections::HashMap;

use super::{GlyphImage, Painter};
use crate::base::{Point, Rect, Size};
use crate::style::Color;

const SHADER: &str = include_str!("rect.wgsl");
const FLOATS_PER_VERTEX: usize = 8;
const INITIAL_CAPACITY: u64 = 4096;
const ATLAS_SIZE: u32 = 2048;
const ATLAS_PADDING: u32 = 1;
const WHITE_SIZE: u32 = 2;

#[derive(Clone, Copy)]
struct Slot {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    left: i32,
    top: i32,
}

struct Upload {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    data: Vec<u8>,
}

struct Packer {
    x: u32,
    y: u32,
    row: u32,
}

impl Packer {
    fn new() -> Self {
        Self {
            x: WHITE_SIZE + ATLAS_PADDING,
            y: 0,
            row: 0,
        }
    }

    fn alloc(&mut self, width: u32, height: u32) -> Option<(u32, u32)> {
        if width + ATLAS_PADDING > ATLAS_SIZE {
            return None;
        }
        if self.x + width + ATLAS_PADDING > ATLAS_SIZE {
            self.x = 0;
            self.y += self.row + ATLAS_PADDING;
            self.row = 0;
        }
        if self.y + height > ATLAS_SIZE {
            return None;
        }
        let position = (self.x, self.y);
        self.x += width + ATLAS_PADDING;
        self.row = self.row.max(height);
        Some(position)
    }
}

pub struct WgpuRenderer {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    texture: wgpu::Texture,
    buffer: wgpu::Buffer,
    capacity: u64,
    vertices: Vec<f32>,
    scratch: Vec<u8>,
    viewport: Size,
    packer: Packer,
    glyphs: HashMap<u64, Option<Slot>>,
    uploads: Vec<Upload>,
}

impl WgpuRenderer {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("meow rect"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("meow atlas"),
            size: wgpu::Extent3d {
                width: ATLAS_SIZE,
                height: ATLAS_SIZE,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("meow atlas layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("meow atlas"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[Some(&bind_group_layout)],
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
                    attributes: &wgpu::vertex_attr_array![
                        0 => Float32x2,
                        1 => Float32x2,
                        2 => Float32x4
                    ],
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
            bind_group,
            texture,
            buffer: Self::create_buffer(device, INITIAL_CAPACITY),
            capacity: INITIAL_CAPACITY,
            vertices: Vec::new(),
            scratch: Vec::new(),
            viewport: Size::default(),
            packer: Packer::new(),
            glyphs: HashMap::new(),
            uploads: vec![Upload {
                x: 0,
                y: 0,
                width: WHITE_SIZE,
                height: WHITE_SIZE,
                data: vec![255; (WHITE_SIZE * WHITE_SIZE) as usize],
            }],
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
        for upload in self.uploads.drain(..) {
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &self.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: upload.x,
                        y: upload.y,
                        z: 0,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                &upload.data,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(upload.width),
                    rows_per_image: None,
                },
                wgpu::Extent3d {
                    width: upload.width,
                    height: upload.height,
                    depth_or_array_layers: 1,
                },
            );
        }

        self.scratch.clear();
        for value in &self.vertices {
            self.scratch.extend_from_slice(&value.to_ne_bytes());
        }
        let needed = self.scratch.len() as u64;

        if needed > self.capacity {
            self.capacity = needed.next_power_of_two();
            self.buffer = Self::create_buffer(device, self.capacity);
        }
        if needed > 0 {
            queue.write_buffer(&self.buffer, 0, &self.scratch);
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
            pass.set_bind_group(0, &self.bind_group, &[]);
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

    fn reserve(&mut self, image: GlyphImage) -> Option<Slot> {
        if image.width == 0 || image.height == 0 {
            return None;
        }
        let (x, y) = self.packer.alloc(image.width, image.height)?;
        self.uploads.push(Upload {
            x,
            y,
            width: image.width,
            height: image.height,
            data: image.coverage,
        });
        Some(Slot {
            x,
            y,
            width: image.width,
            height: image.height,
            left: image.left,
            top: image.top,
        })
    }

    fn push_quad(&mut self, rect: Rect, uv: [f32; 4], color: Color) {
        let Size { width, height } = self.viewport;
        let x0 = rect.origin.x / width * 2.0 - 1.0;
        let y0 = 1.0 - rect.origin.y / height * 2.0;
        let x1 = (rect.origin.x + rect.size.width) / width * 2.0 - 1.0;
        let y1 = 1.0 - (rect.origin.y + rect.size.height) / height * 2.0;
        let [u0, v0, u1, v1] = uv;

        let corners = [
            (x0, y0, u0, v0),
            (x1, y0, u1, v0),
            (x0, y1, u0, v1),
            (x1, y0, u1, v0),
            (x1, y1, u1, v1),
            (x0, y1, u0, v1),
        ];
        for (x, y, u, v) in corners {
            self.vertices
                .extend_from_slice(&[x, y, u, v, color.r, color.g, color.b, color.a]);
        }
    }
}

impl Painter for WgpuRenderer {
    fn fill_rect(&mut self, rect: Rect, color: Color) {
        let texel = 1.0 / ATLAS_SIZE as f32;
        self.push_quad(rect, [texel, texel, texel, texel], color);
    }

    fn draw_glyph(
        &mut self,
        key: u64,
        origin: Point,
        color: Color,
        image: &mut dyn FnMut() -> Option<GlyphImage>,
    ) {
        let slot = match self.glyphs.get(&key).copied() {
            Some(slot) => slot,
            None => {
                let slot = image().and_then(|image| self.reserve(image));
                self.glyphs.insert(key, slot);
                slot
            }
        };
        let Some(slot) = slot else {
            return;
        };

        let size = ATLAS_SIZE as f32;
        let rect = Rect::new(
            (origin.x + slot.left as f32).round(),
            (origin.y - slot.top as f32).round(),
            slot.width as f32,
            slot.height as f32,
        );
        let uv = [
            slot.x as f32 / size,
            slot.y as f32 / size,
            (slot.x + slot.width) as f32 / size,
            (slot.y + slot.height) as f32 / size,
        ];
        self.push_quad(rect, uv, color);
    }
}
