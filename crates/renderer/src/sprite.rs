//! 2D sprite rendering module.

use wgpu::{
    BindGroup, BindGroupLayout, Buffer, Device, PipelineLayout, RenderPipeline, Sampler, TextureView,
};

/// A 2D camera for orthographic projection
#[derive(Debug)]
pub struct Camera2D {
    /// The position of the camera
    pub position: [f32; 2],
    /// The zoom level of the camera
    pub zoom: f32,
    /// The aspect ratio of the camera
    pub aspect_ratio: f32,
}

impl Default for Camera2D {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0],
            zoom: 1.0,
            aspect_ratio: 1.0,
        }
    }
}

/// A batch of sprites to be rendered together
#[derive(Debug)]
pub struct SpriteBatch {
    /// The texture to use for this batch
    pub texture: TextureView,
    /// The bind group for this batch
    pub bind_group: BindGroup,
    /// The number of sprites in this batch
    pub count: u32,
}

/// A pipeline for rendering 2D sprites
pub struct SpritePipeline {
    /// The render pipeline
    pipeline: RenderPipeline,
    /// The pipeline layout
    layout: PipelineLayout,
    /// The vertex buffer
    vertex_buffer: Buffer,
    /// The index buffer
    index_buffer: Buffer,
    /// The texture atlas sampler
    sampler: Sampler,
    /// The bind group layout
    bind_group_layout: BindGroupLayout,
}

impl SpritePipeline {
    /// Create a new sprite pipeline
    pub fn new(device: &Device) -> Self {
        // Create the bind group layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Sprite Bind Group Layout"),
            entries: &[
                // Texture
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // Sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        // Create the pipeline layout
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Sprite Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create the sampler
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Sprite Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        // Create vertex and index buffers for 1000 quads
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Sprite Vertex Buffer"),
            size: 4 * 1000 * std::mem::size_of::<[f32; 5]>() as u64, // pos[2] + uv[2] + color[1]
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Sprite Index Buffer"),
            size: 6 * 1000 * std::mem::size_of::<u16>() as u64, // 2 triangles per quad
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create the shader module
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Sprite Shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!("shaders/sprite.wgsl"))),
        });

        // Create the render pipeline
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Sprite Pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<[f32; 5]>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        // Position
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 0,
                        },
                        // UV
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 2 * 4,
                            shader_location: 1,
                        },
                        // Color
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32,
                            offset: 4 * 4,
                            shader_location: 2,
                        },
                    ],
                }],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Bgra8UnormSrgb,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        Self {
            pipeline,
            layout,
            vertex_buffer,
            index_buffer,
            sampler,
            bind_group_layout,
        }
    }

    /// Draw sprites using this pipeline
    pub fn draw(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        _camera: &Camera2D,
        sprite_batches: &[SpriteBatch],
        target: &wgpu::TextureView,
    ) {
        // Begin render pass
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Sprite Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load, // Assume the target is already cleared
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        // Set pipeline
        render_pass.set_pipeline(&self.pipeline);

        // Set vertex and index buffers
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

        // Draw each batch
        for batch in sprite_batches {
            render_pass.set_bind_group(0, &batch.bind_group, &[]);
            render_pass.draw_indexed(0..6 * batch.count, 0, 0..1);
        }
    }

    /// Create a bind group for a texture
    pub fn create_bind_group(&self, device: &Device, texture_view: &TextureView) -> BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Sprite Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        })
    }
}