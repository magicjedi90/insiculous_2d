//! 2D sprite rendering module with batching and camera support

use glam::{Vec2, Vec3, Vec4};
use std::collections::HashMap;
use std::sync::Arc;
use wgpu::{Device, Queue, RenderPipeline, PipelineLayout, BindGroupLayout, Buffer, Sampler, TextureView, CommandEncoder};
use wgpu::util::DeviceExt;

use crate::sprite_data::{SpriteVertex, SpriteInstance, Camera2D, CameraUniform, TextureResource, DynamicBuffer};
use crate::texture::TextureHandle;

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

/// A batch of sprites using the same texture
#[derive(Debug, Clone)]
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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    /// The pipeline layout
    layout: PipelineLayout,
    /// Vertex buffer for quad geometry
    vertex_buffer: Buffer,
    #[allow(dead_code)]
    /// Instance buffer for sprite data
    instance_buffer: DynamicBuffer<SpriteInstance>,
    /// Index buffer for quad geometry
    index_buffer: Buffer,
    /// Camera uniform buffer
    camera_buffer: Buffer,
    /// Texture bind group layout
    texture_bind_group_layout: BindGroupLayout,
    /// Camera bind group layout
    camera_bind_group_layout: BindGroupLayout,
    #[allow(dead_code)]
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

        // Create index buffer for quad (2 triangles, 6 indices)
        let indices: [u16; 6] = [
            0, 1, 2, // First triangle
            0, 2, 3, // Second triangle
        ];
        
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sprite Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
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
            index_buffer,
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

    /// Update instance buffer with sprite data and return the number of instances
    pub fn update_instance_buffer(&mut self, queue: &Queue, instances: &[SpriteInstance]) -> usize {
        if instances.is_empty() {
            return 0;
        }
        
        let instance_count = instances.len();
        
        // Update the instance buffer
        self.instance_buffer.update(queue, instances);
        
        instance_count
    }

    /// Draw sprites using this pipeline
    pub fn draw(
        &self,
        encoder: &mut CommandEncoder,
        _camera: &Camera2D,
        texture_resources: &HashMap<TextureHandle, TextureResource>,
        batches: &[&SpriteBatch],
        target: &TextureView,
        clear_color: wgpu::Color,
    ) {
        log::info!("SPRITE DRAW: batches={}, clear_color={:?}", batches.len(), clear_color);
        // Begin render pass with clearing
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Sprite Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(clear_color),
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
        log::info!("✓ Pipeline set");

        // Set vertex buffers
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice());
        log::info!("✓ Vertex buffers set: vertex_buffer_size={}, instance_buffer_size={}", 
                   self.vertex_buffer.size(), self.instance_buffer.buffer().size());

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
        let mut instance_offset = 0u32;
        for batch in batches {
            if batch.is_empty() {
                continue;
            }

            // Get texture resource - the renderer should have provided a white texture for colored sprites
            let texture_bind_group = if let Some(texture_resource) = texture_resources.get(&batch.texture_handle) {
                // Use provided texture resource (should include white texture for colored sprites)
                self.device.create_bind_group(&wgpu::BindGroupDescriptor {
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
                })
            } else {
                // This should not happen if the renderer is working correctly
                log::warn!("No texture resource found for texture handle {:?}, sprite may not render correctly", batch.texture_handle);
                
                // Create a fallback bind group - this is a last resort
                self.create_fallback_texture_bind_group()
            };

            // Set texture bind group (set 1)
            render_pass.set_bind_group(1, &texture_bind_group, &[]);

            // Draw the instances for this batch
            let instance_count = batch.len() as u32;
            
            // Draw indexed triangles with instancing
            // 6 indices per quad (2 triangles), draw instances from instance_offset to instance_offset + instance_count
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..6, 0, instance_offset..(instance_offset + instance_count));
            
            // Update offset for next batch
            instance_offset += instance_count;
        }
    }

    /// Get maximum sprites per batch
    pub fn max_sprites_per_batch(&self) -> usize {
        self.max_sprites_per_batch
    }

    /// Get the render pipeline
    pub fn pipeline(&self) -> &RenderPipeline {
        &self.pipeline
    }

    /// Get the camera bind group layout
    pub fn camera_bind_group_layout(&self) -> &BindGroupLayout {
        &self.camera_bind_group_layout
    }

    /// Get the texture bind group layout
    pub fn texture_bind_group_layout(&self) -> &BindGroupLayout {
        &self.texture_bind_group_layout
    }

    /// Create a fallback texture bind group for emergency cases
    /// This should rarely be used if the renderer is working correctly
    fn create_fallback_texture_bind_group(&self) -> wgpu::BindGroup {
        // Create a simple 1x1 texture as a last resort
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Fallback Texture"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Fallback Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Fallback Texture Bind Group"),
            layout: &self.texture_bind_group_layout,
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
        })
    }

    /// Prepare sprite data for rendering by updating instance buffer and creating batches
    pub fn prepare_sprites(&mut self, queue: &Queue, batches: &[&SpriteBatch]) {
        let mut all_instances = Vec::new();

        // Collect all instances from batches
        for batch in batches {
            for instance in &batch.instances {
                all_instances.push(*instance);
            }
        }

        // Update the instance buffer
        if !all_instances.is_empty() {
            log::debug!("Preparing {} sprite instances for GPU upload", all_instances.len());
            self.update_instance_buffer(queue, &all_instances);
        }
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
    #[allow(clippy::too_many_arguments)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use glam::{Vec2, Vec4};

    // ==================== Sprite Tests ====================

    #[test]
    fn test_sprite_default() {
        let sprite = Sprite::default();
        assert_eq!(sprite.position, Vec2::ZERO);
        assert_eq!(sprite.rotation, 0.0);
        assert_eq!(sprite.scale, Vec2::ONE);
        assert_eq!(sprite.tex_region, [0.0, 0.0, 1.0, 1.0]);
        assert_eq!(sprite.color, Vec4::ONE);
        assert_eq!(sprite.depth, 0.0);
        assert_eq!(sprite.texture_handle.id, 0);
    }

    #[test]
    fn test_sprite_new_with_texture() {
        let handle = TextureHandle::new(42);
        let sprite = Sprite::new(handle);
        assert_eq!(sprite.texture_handle.id, 42);
        // Other fields should be default
        assert_eq!(sprite.position, Vec2::ZERO);
        assert_eq!(sprite.scale, Vec2::ONE);
    }

    #[test]
    fn test_sprite_builder_position() {
        let sprite = Sprite::new(TextureHandle::default())
            .with_position(Vec2::new(100.0, 200.0));
        assert_eq!(sprite.position, Vec2::new(100.0, 200.0));
    }

    #[test]
    fn test_sprite_builder_rotation() {
        let sprite = Sprite::new(TextureHandle::default())
            .with_rotation(std::f32::consts::PI);
        assert!((sprite.rotation - std::f32::consts::PI).abs() < 0.0001);
    }

    #[test]
    fn test_sprite_builder_scale() {
        let sprite = Sprite::new(TextureHandle::default())
            .with_scale(Vec2::new(2.0, 3.0));
        assert_eq!(sprite.scale, Vec2::new(2.0, 3.0));
    }

    #[test]
    fn test_sprite_builder_tex_region() {
        let sprite = Sprite::new(TextureHandle::default())
            .with_tex_region(0.25, 0.5, 0.5, 0.25);
        assert_eq!(sprite.tex_region, [0.25, 0.5, 0.5, 0.25]);
    }

    #[test]
    fn test_sprite_builder_color() {
        let color = Vec4::new(1.0, 0.0, 0.0, 0.5);
        let sprite = Sprite::new(TextureHandle::default())
            .with_color(color);
        assert_eq!(sprite.color, color);
    }

    #[test]
    fn test_sprite_builder_depth() {
        let sprite = Sprite::new(TextureHandle::default())
            .with_depth(5.0);
        assert_eq!(sprite.depth, 5.0);
    }

    #[test]
    fn test_sprite_builder_chaining() {
        let sprite = Sprite::new(TextureHandle::new(1))
            .with_position(Vec2::new(10.0, 20.0))
            .with_rotation(1.5)
            .with_scale(Vec2::new(2.0, 2.0))
            .with_color(Vec4::new(1.0, 0.0, 0.0, 1.0))
            .with_depth(10.0);

        assert_eq!(sprite.texture_handle.id, 1);
        assert_eq!(sprite.position, Vec2::new(10.0, 20.0));
        assert!((sprite.rotation - 1.5).abs() < 0.0001);
        assert_eq!(sprite.scale, Vec2::new(2.0, 2.0));
        assert_eq!(sprite.color, Vec4::new(1.0, 0.0, 0.0, 1.0));
        assert_eq!(sprite.depth, 10.0);
    }

    #[test]
    fn test_sprite_to_instance() {
        let sprite = Sprite::new(TextureHandle::default())
            .with_position(Vec2::new(100.0, 200.0))
            .with_rotation(0.5)
            .with_scale(Vec2::new(2.0, 3.0))
            .with_color(Vec4::new(1.0, 0.5, 0.25, 1.0))
            .with_depth(5.0);

        let instance = sprite.to_instance();
        assert_eq!(instance.position, [100.0, 200.0]);
        assert!((instance.rotation - 0.5).abs() < 0.0001);
        assert_eq!(instance.scale, [2.0, 3.0]);
        assert_eq!(instance.color, [1.0, 0.5, 0.25, 1.0]);
        assert_eq!(instance.depth, 5.0);
    }

    // ==================== SpriteBatch Tests ====================

    #[test]
    fn test_sprite_batch_new() {
        let handle = TextureHandle::new(5);
        let batch = SpriteBatch::new(handle);
        assert_eq!(batch.texture_handle.id, 5);
        assert!(batch.instances.is_empty());
        assert!(!batch.sorted);
    }

    #[test]
    fn test_sprite_batch_add_instance() {
        let mut batch = SpriteBatch::new(TextureHandle::default());
        let instance = SpriteInstance::new(
            Vec2::new(10.0, 20.0),
            0.0,
            Vec2::ONE,
            [0.0, 0.0, 1.0, 1.0],
            Vec4::ONE,
            0.0,
        );
        batch.add_instance(instance);
        assert_eq!(batch.len(), 1);
        assert!(!batch.sorted);
    }

    #[test]
    fn test_sprite_batch_add_instances() {
        let mut batch = SpriteBatch::new(TextureHandle::default());
        let instances = vec![
            SpriteInstance::new(Vec2::ZERO, 0.0, Vec2::ONE, [0.0, 0.0, 1.0, 1.0], Vec4::ONE, 0.0),
            SpriteInstance::new(Vec2::ONE, 0.0, Vec2::ONE, [0.0, 0.0, 1.0, 1.0], Vec4::ONE, 1.0),
            SpriteInstance::new(Vec2::new(2.0, 2.0), 0.0, Vec2::ONE, [0.0, 0.0, 1.0, 1.0], Vec4::ONE, 2.0),
        ];
        batch.add_instances(&instances);
        assert_eq!(batch.len(), 3);
    }

    #[test]
    fn test_sprite_batch_sort_by_depth() {
        let mut batch = SpriteBatch::new(TextureHandle::default());
        batch.add_instance(SpriteInstance::new(Vec2::ZERO, 0.0, Vec2::ONE, [0.0, 0.0, 1.0, 1.0], Vec4::ONE, 3.0));
        batch.add_instance(SpriteInstance::new(Vec2::ZERO, 0.0, Vec2::ONE, [0.0, 0.0, 1.0, 1.0], Vec4::ONE, 1.0));
        batch.add_instance(SpriteInstance::new(Vec2::ZERO, 0.0, Vec2::ONE, [0.0, 0.0, 1.0, 1.0], Vec4::ONE, 2.0));

        assert!(!batch.sorted);
        batch.sort_by_depth();
        assert!(batch.sorted);

        // Verify sorted order (ascending)
        assert_eq!(batch.instances[0].depth, 1.0);
        assert_eq!(batch.instances[1].depth, 2.0);
        assert_eq!(batch.instances[2].depth, 3.0);
    }

    #[test]
    fn test_sprite_batch_sort_idempotent() {
        let mut batch = SpriteBatch::new(TextureHandle::default());
        batch.add_instance(SpriteInstance::new(Vec2::ZERO, 0.0, Vec2::ONE, [0.0, 0.0, 1.0, 1.0], Vec4::ONE, 2.0));
        batch.add_instance(SpriteInstance::new(Vec2::ZERO, 0.0, Vec2::ONE, [0.0, 0.0, 1.0, 1.0], Vec4::ONE, 1.0));

        batch.sort_by_depth();
        assert!(batch.sorted);

        // Sorting again should be a no-op since already sorted
        batch.sort_by_depth();
        assert!(batch.sorted);
        assert_eq!(batch.instances[0].depth, 1.0);
        assert_eq!(batch.instances[1].depth, 2.0);
    }

    #[test]
    fn test_sprite_batch_len_and_is_empty() {
        let mut batch = SpriteBatch::new(TextureHandle::default());
        assert!(batch.is_empty());
        assert_eq!(batch.len(), 0);

        batch.add_instance(SpriteInstance::new(Vec2::ZERO, 0.0, Vec2::ONE, [0.0, 0.0, 1.0, 1.0], Vec4::ONE, 0.0));
        assert!(!batch.is_empty());
        assert_eq!(batch.len(), 1);
    }

    #[test]
    fn test_sprite_batch_clear() {
        let mut batch = SpriteBatch::new(TextureHandle::default());
        batch.add_instance(SpriteInstance::new(Vec2::ZERO, 0.0, Vec2::ONE, [0.0, 0.0, 1.0, 1.0], Vec4::ONE, 0.0));
        batch.add_instance(SpriteInstance::new(Vec2::ONE, 0.0, Vec2::ONE, [0.0, 0.0, 1.0, 1.0], Vec4::ONE, 1.0));
        batch.sort_by_depth();

        assert_eq!(batch.len(), 2);
        assert!(batch.sorted);

        batch.clear();
        assert!(batch.is_empty());
        assert!(!batch.sorted);
    }

    #[test]
    fn test_sprite_batch_sorted_flag_reset_on_add() {
        let mut batch = SpriteBatch::new(TextureHandle::default());
        batch.add_instance(SpriteInstance::new(Vec2::ZERO, 0.0, Vec2::ONE, [0.0, 0.0, 1.0, 1.0], Vec4::ONE, 0.0));
        batch.sort_by_depth();
        assert!(batch.sorted);

        // Adding should reset sorted flag
        batch.add_instance(SpriteInstance::new(Vec2::ONE, 0.0, Vec2::ONE, [0.0, 0.0, 1.0, 1.0], Vec4::ONE, 1.0));
        assert!(!batch.sorted);
    }

    // ==================== SpriteBatcher Tests ====================

    #[test]
    fn test_sprite_batcher_new() {
        let batcher = SpriteBatcher::new(1000);
        assert_eq!(batcher.sprite_count(), 0);
        assert!(batcher.batches().is_empty());
    }

    #[test]
    fn test_sprite_batcher_add_sprite() {
        let mut batcher = SpriteBatcher::new(1000);
        let sprite = Sprite::new(TextureHandle::new(1));
        batcher.add_sprite(&sprite);
        assert_eq!(batcher.sprite_count(), 1);
    }

    #[test]
    fn test_sprite_batcher_add_sprites() {
        let mut batcher = SpriteBatcher::new(1000);
        let sprites = vec![
            Sprite::new(TextureHandle::new(1)),
            Sprite::new(TextureHandle::new(1)),
            Sprite::new(TextureHandle::new(2)),
        ];
        batcher.add_sprites(&sprites);
        assert_eq!(batcher.sprite_count(), 3);
    }

    #[test]
    fn test_sprite_batcher_groups_by_texture() {
        let mut batcher = SpriteBatcher::new(1000);

        // Add sprites with different textures
        batcher.add_sprite(&Sprite::new(TextureHandle::new(1)));
        batcher.add_sprite(&Sprite::new(TextureHandle::new(1)));
        batcher.add_sprite(&Sprite::new(TextureHandle::new(2)));
        batcher.add_sprite(&Sprite::new(TextureHandle::new(2)));
        batcher.add_sprite(&Sprite::new(TextureHandle::new(3)));

        let batches = batcher.batches();
        assert_eq!(batches.len(), 3); // 3 different textures

        assert_eq!(batches.get(&TextureHandle::new(1)).unwrap().len(), 2);
        assert_eq!(batches.get(&TextureHandle::new(2)).unwrap().len(), 2);
        assert_eq!(batches.get(&TextureHandle::new(3)).unwrap().len(), 1);
    }

    #[test]
    fn test_sprite_batcher_sprite_count() {
        let mut batcher = SpriteBatcher::new(1000);
        assert_eq!(batcher.sprite_count(), 0);

        batcher.add_sprite(&Sprite::new(TextureHandle::new(1)));
        assert_eq!(batcher.sprite_count(), 1);

        batcher.add_sprite(&Sprite::new(TextureHandle::new(2)));
        assert_eq!(batcher.sprite_count(), 2);

        batcher.add_sprite(&Sprite::new(TextureHandle::new(1)));
        assert_eq!(batcher.sprite_count(), 3);
    }

    #[test]
    fn test_sprite_batcher_sort_all_batches() {
        let mut batcher = SpriteBatcher::new(1000);

        batcher.add_sprite(&Sprite::new(TextureHandle::new(1)).with_depth(3.0));
        batcher.add_sprite(&Sprite::new(TextureHandle::new(1)).with_depth(1.0));
        batcher.add_sprite(&Sprite::new(TextureHandle::new(2)).with_depth(5.0));
        batcher.add_sprite(&Sprite::new(TextureHandle::new(2)).with_depth(2.0));

        batcher.sort_all_batches();

        let batch1 = batcher.batches().get(&TextureHandle::new(1)).unwrap();
        assert!(batch1.sorted);
        assert_eq!(batch1.instances[0].depth, 1.0);
        assert_eq!(batch1.instances[1].depth, 3.0);

        let batch2 = batcher.batches().get(&TextureHandle::new(2)).unwrap();
        assert!(batch2.sorted);
        assert_eq!(batch2.instances[0].depth, 2.0);
        assert_eq!(batch2.instances[1].depth, 5.0);
    }

    #[test]
    fn test_sprite_batcher_clear() {
        let mut batcher = SpriteBatcher::new(1000);
        batcher.add_sprite(&Sprite::new(TextureHandle::new(1)));
        batcher.add_sprite(&Sprite::new(TextureHandle::new(2)));
        assert_eq!(batcher.sprite_count(), 2);

        batcher.clear();
        assert_eq!(batcher.sprite_count(), 0);

        // Batches still exist but are empty
        assert!(!batcher.batches().is_empty());
        for batch in batcher.batches().values() {
            assert!(batch.is_empty());
        }
    }

    #[test]
    fn test_sprite_batcher_batches_mutable() {
        let mut batcher = SpriteBatcher::new(1000);
        batcher.add_sprite(&Sprite::new(TextureHandle::new(1)));

        // Verify we can get mutable access
        let batches = batcher.batches_mut();
        if let Some(batch) = batches.get_mut(&TextureHandle::new(1)) {
            batch.clear();
        }

        assert_eq!(batcher.sprite_count(), 0);
    }
}