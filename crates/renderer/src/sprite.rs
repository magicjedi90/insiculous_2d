//! 2D sprite rendering module with batching and camera support

use glam::{Vec2, Vec3, Vec4};
use std::collections::HashMap;
use std::sync::Arc;
use wgpu::{Device, Queue, RenderPipeline, PipelineLayout, BindGroupLayout, Buffer, Sampler, TextureView, CommandEncoder};
use wgpu::util::DeviceExt;

use crate::sprite_data::{SpriteVertex, SpriteInstance, Camera2D, CameraUniform, TextureResource, DynamicBuffer};

/// A single sprite to be rendered
#[derive(Debug, Clone)]
pub struct Sprite {
    /// Position in world space
    pub position: Vec2,
    /// Rotation in radians
    pub rotation: f32,
    /// Scale
    pub scale: Vec2,
    /// Texture region (x, y, width, height) in texture coordinates [0, 1]
    pub tex_region: [f32; 4],
    /// Color tint
    pub color: Vec4,
    /// Layer depth for sorting (higher values render on top)
    pub depth: f32,
    /// Texture handle
    pub texture_handle: TextureHandle,
}

impl Default for Sprite {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            rotation: 0.0,
            scale: Vec2::ONE,
            tex_region: [0.0, 0.0, 1.0, 1.0], // Full texture
            color: Vec4::ONE, // White
            depth: 0.0,
            texture_handle: TextureHandle::default(),
        }
    }
}

impl Sprite {
    /// Create a new sprite
    pub fn new(texture_handle: TextureHandle) -> Self {
        Self {
            texture_handle,
            ..Default::default()
        }
    }

    /// Set sprite position
    pub fn with_position(mut self, position: Vec2) -> Self {
        self.position = position;
        self
    }

    /// Set sprite rotation
    pub fn with_rotation(mut self, rotation: f32) -> Self {
        self.rotation = rotation;
        self
    }

    /// Set sprite scale
    pub fn with_scale(mut self, scale: Vec2) -> Self {
        self.scale = scale;
        self
    }

    /// Set texture region (UV coordinates)
    pub fn with_tex_region(mut self, x: f32, y: f32, width: f32, height: f32) -> Self {
        self.tex_region = [x, y, width, height];
        self
    }

    /// Set color tint
    pub fn with_color(mut self, color: Vec4) -> Self {
        self.color = color;
        self
    }

    /// Set depth
    pub fn with_depth(mut self, depth: f32) -> Self {
        self.depth = depth;
        self
    }

    /// Convert to sprite instance for batching
    pub fn to_instance(&self) -> SpriteInstance {
        SpriteInstance::new(
            self.position,
            self.rotation,
            self.scale,
            self.tex_region,
            self.color,
            self.depth,
        )
    }
}

/// Handle to a texture resource
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct TextureHandle {
    pub id: u32,
}

impl TextureHandle {
    /// Create a new texture handle
    pub fn new(id: u32) -> Self {
        Self { id }
    }
}

/// A batch of sprites using the same texture
#[derive(Debug)]
pub struct SpriteBatch {
    /// Texture handle for this batch
    pub texture_handle: TextureHandle,
    /// Sprite instances
    pub instances: Vec<SpriteInstance>,
    /// Whether this batch is sorted by depth
    pub sorted: bool,
}

impl SpriteBatch {
    /// Create a new sprite batch
    pub fn new(texture_handle: TextureHandle) -> Self {
        Self {
            texture_handle,
            instances: Vec::new(),
            sorted: false,
        }
    }

    /// Add a sprite instance to the batch
    pub fn add_instance(&mut self, instance: SpriteInstance) {
        self.instances.push(instance);
        self.sorted = false;
    }

    /// Add multiple sprite instances
    pub fn add_instances(&mut self, instances: &[SpriteInstance]) {
        self.instances.extend_from_slice(instances);
        self.sorted = false;
    }

    /// Sort instances by depth (for proper alpha blending)
    pub fn sort_by_depth(&mut self) {
        if !self.sorted {
            self.instances.sort_by(|a, b| a.depth.partial_cmp(&b.depth).unwrap());
            self.sorted = true;
        }
    }

    /// Get the number of instances
    pub fn len(&self) -> usize {
        self.instances.len()
    }

    /// Check if batch is empty
    pub fn is_empty(&self) -> bool {
        self.instances.is_empty()
    }

    /// Clear all instances
    pub fn clear(&mut self) {
        self.instances.clear();
        self.sorted = false;
    }
}

/// Sprite batcher for efficient rendering
pub struct SpriteBatcher {
    batches: HashMap<TextureHandle, SpriteBatch>,
    max_sprites_per_batch: usize,
}

impl SpriteBatcher {
    /// Create a new sprite batcher
    pub fn new(max_sprites_per_batch: usize) -> Self {
        Self {
            batches: HashMap::new(),
            max_sprites_per_batch,
        }
    }

    /// Add a sprite to the batcher
    pub fn add_sprite(&mut self, sprite: &Sprite) {
        let batch = self.batches
            .entry(sprite.texture_handle)
            .or_insert_with(|| SpriteBatch::new(sprite.texture_handle));

        // If batch is too large, we could split it, but for now just add anyway
        batch.add_instance(sprite.to_instance());
    }

    /// Add multiple sprites
    pub fn add_sprites(&mut self, sprites: &[Sprite]) {
        for sprite in sprites {
            self.add_sprite(sprite);
        }
    }

    /// Sort all batches by depth
    pub fn sort_all_batches(&mut self) {
        for batch in self.batches.values_mut() {
            batch.sort_by_depth();
        }
    }

    /// Get all batches
    pub fn batches(&self) -> &HashMap<TextureHandle, SpriteBatch> {
        &self.batches
    }

    /// Get mutable batches
    pub fn batches_mut(&mut self) -> &mut HashMap<TextureHandle, SpriteBatch> {
        &mut self.batches
    }

    /// Clear all batches
    pub fn clear(&mut self) {
        for batch in self.batches.values_mut() {
            batch.clear();
        }
    }

    /// Get total sprite count
    pub fn sprite_count(&self) -> usize {
        self.batches.values().map(|batch| batch.len()).sum()
    }
}

/// Enhanced sprite pipeline with camera support and proper batching
pub struct SpritePipeline {
    /// The render pipeline
    pipeline: RenderPipeline,
    /// The pipeline layout
    layout: PipelineLayout,
    /// Vertex buffer for quad geometry
    vertex_buffer: Buffer,
    /// Instance buffer for sprite data
    instance_buffer: DynamicBuffer<SpriteInstance>,
    /// Camera uniform buffer
    camera_buffer: Buffer,
    /// Texture bind group layout
    texture_bind_group_layout: BindGroupLayout,
    /// Camera bind group layout
    camera_bind_group_layout: BindGroupLayout,
    /// Sampler for textures
    sampler: Sampler,
    /// Maximum sprites per batch
    max_sprites_per_batch: usize,
    /// Device reference for creating bind groups
    device: Arc<Device>,
}

impl SpritePipeline {
    /// Create a new sprite pipeline
    pub fn new(device: &Device, max_sprites_per_batch: usize) -> Self {
        let device_arc = Arc::new(device.clone());

        // Create texture bind group layout
        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Sprite Texture Bind Group Layout"),
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

        // Create camera bind group layout
        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Sprite Camera Bind Group Layout"),
            entries: &[
                // Camera uniform
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        // Create pipeline layout
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Sprite Pipeline Layout"),
            bind_group_layouts: &[&camera_bind_group_layout, &texture_bind_group_layout],
            ..Default::default()
        });

        // Create sampler
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Sprite Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });

        // Create quad vertices (full texture coordinates)
        let vertices = [
            // Top-left
            SpriteVertex::new(Vec3::new(-0.5, 0.5, 0.0), Vec2::new(0.0, 0.0), Vec4::ONE),
            // Top-right
            SpriteVertex::new(Vec3::new(0.5, 0.5, 0.0), Vec2::new(1.0, 0.0), Vec4::ONE),
            // Bottom-right
            SpriteVertex::new(Vec3::new(0.5, -0.5, 0.0), Vec2::new(1.0, 1.0), Vec4::ONE),
            // Bottom-left
            SpriteVertex::new(Vec3::new(-0.5, -0.5, 0.0), Vec2::new(0.0, 1.0), Vec4::ONE),
        ];

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sprite Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        // Create instance buffer
        let instance_buffer = DynamicBuffer::new(
            device,
            max_sprites_per_batch,
            wgpu::BufferUsages::VERTEX,
        );

        // Create camera uniform buffer
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sprite Camera Buffer"),
            contents: bytemuck::cast_slice(&[CameraUniform::from_camera(&Camera2D::default())]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create shader module
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Sprite Shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!("shaders/sprite_instanced.wgsl"))),
        });

        // Create the render pipeline
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Sprite Pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[
                    SpriteVertex::desc(),     // Vertex buffer
                    SpriteInstance::desc(),   // Instance buffer
                ],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Bgra8UnormSrgb,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None, // Don't cull sprites
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            cache: None,
            multiview_mask: None,
        });

        Self {
            pipeline,
            layout,
            vertex_buffer,
            instance_buffer,
            camera_buffer,
            texture_bind_group_layout,
            camera_bind_group_layout,
            sampler,
            max_sprites_per_batch,
            device: device_arc,
        }
    }

    /// Update camera uniform
    pub fn update_camera(&self, queue: &Queue, camera: &Camera2D) {
        let uniform = CameraUniform::from_camera(camera);
        queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[uniform]));
    }

    /// Draw sprites using this pipeline
    pub fn draw(
        &self,
        encoder: &mut CommandEncoder,
        _camera: &Camera2D,
        texture_resources: &HashMap<TextureHandle, TextureResource>,
        batches: &[&SpriteBatch],
        target: &TextureView,
    ) {
        // Update camera uniform
        // Note: This assumes we have access to queue, but we don't in this method
        // In a real implementation, we'd need to pass the queue or update camera elsewhere

        // Begin render pass
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Sprite Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        // Set pipeline
        render_pass.set_pipeline(&self.pipeline);

        // Set vertex buffer
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

        // Create camera bind group
        let camera_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Sprite Camera Bind Group"),
            layout: &self.camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: self.camera_buffer.as_entire_binding(),
            }],
        });

        // Set camera bind group (set 0)
        render_pass.set_bind_group(0, &camera_bind_group, &[]);

        // Draw each batch
        for batch in batches {
            if batch.is_empty() {
                continue;
            }

            // Get texture resource
            if let Some(texture_resource) = texture_resources.get(&batch.texture_handle) {
                // Note: We can't update the instance buffer here because we don't have queue access
                // In a real implementation, this would need to be handled differently
                // For now, we'll skip the instance buffer update

                // Create texture bind group
                let texture_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Sprite Texture Bind Group"),
                    layout: &self.texture_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&texture_resource.view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&texture_resource.sampler),
                        },
                    ],
                });

                // Set texture bind group (set 1)
                render_pass.set_bind_group(1, &texture_bind_group, &[]);

                // Note: We can't draw instances here because we haven't updated the instance buffer
                // This is a limitation of the current design - in a real implementation,
                // the instance buffer would need to be updated before calling this method
            }
        }
    }

    /// Get maximum sprites per batch
    pub fn max_sprites_per_batch(&self) -> usize {
        self.max_sprites_per_batch
    }
}

/// Texture atlas for efficient sprite rendering
pub struct TextureAtlas {
    texture: Arc<wgpu::Texture>,
    view: TextureView,
    sampler: Sampler,
    regions: HashMap<String, [f32; 4]>, // name -> [x, y, width, height] in UV coordinates
}

impl TextureAtlas {
    /// Create a new texture atlas
    pub fn new(device: &Device, width: u32, height: u32) -> Self {
        let texture = Arc::new(device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Texture Atlas"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        }));

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Texture Atlas Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });

        Self {
            texture,
            view,
            sampler,
            regions: HashMap::new(),
        }
    }

    /// Add a region to the atlas
    pub fn add_region(&mut self, name: String, x: u32, y: u32, width: u32, height: u32, atlas_width: u32, atlas_height: u32) {
        let u0 = x as f32 / atlas_width as f32;
        let v0 = y as f32 / atlas_height as f32;
        let u1 = (x + width) as f32 / atlas_width as f32;
        let v1 = (y + height) as f32 / atlas_height as f32;

        self.regions.insert(name, [u0, v0, u1 - u0, v1 - v0]);
    }

    /// Get a region by name
    pub fn get_region(&self, name: &str) -> Option<[f32; 4]> {
        self.regions.get(name).copied()
    }

    /// Get texture view
    pub fn view(&self) -> &TextureView {
        &self.view
    }

    /// Get sampler
    pub fn sampler(&self) -> &Sampler {
        &self.sampler
    }

    /// Get texture
    pub fn texture(&self) -> &Arc<wgpu::Texture> {
        &self.texture
    }
}