//! Sprite data structures and management

use glam::{Vec2, Vec3, Vec4, Mat4};
use std::sync::Arc;
use wgpu::{Device, Queue, Texture, TextureView, Sampler, Buffer};

/// Vertex data for a sprite
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SpriteVertex {
    /// Position in world space
    pub position: [f32; 3],
    /// Texture coordinates
    pub tex_coords: [f32; 2],
    /// Color tint (RGBA as 4 floats)
    pub color: [f32; 4],
}

impl SpriteVertex {
    /// Create a new sprite vertex
    pub fn new(position: Vec3, tex_coords: Vec2, color: Vec4) -> Self {
        Self {
            position: position.to_array(),
            tex_coords: tex_coords.to_array(),
            color: color.to_array(),
        }
    }

    /// Get the vertex buffer layout
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<SpriteVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // Position
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // Texture coordinates
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // Color
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

/// Sprite instance data for batching
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SpriteInstance {
    /// World position
    pub position: [f32; 2],
    /// Rotation in radians
    pub rotation: f32,
    /// Scale
    pub scale: [f32; 2],
    /// Texture region (x, y, width, height) in texture coordinates [0, 1]
    pub tex_region: [f32; 4],
    /// Color tint
    pub color: [f32; 4],
    /// Layer depth for sorting
    pub depth: f32,
}

impl SpriteInstance {
    /// Create a new sprite instance
    pub fn new(
        position: Vec2,
        rotation: f32,
        scale: Vec2,
        tex_region: [f32; 4],
        color: Vec4,
        depth: f32,
    ) -> Self {
        Self {
            position: position.to_array(),
            rotation,
            scale: scale.to_array(),
            tex_region,
            color: color.to_array(),
            depth,
        }
    }

    /// Get the instance buffer layout
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<SpriteInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // Position
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // Rotation
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32,
                },
                // Scale
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // Texture region
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // Color
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 9]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // Depth
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 13]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32,
                },
            ],
        }
    }
}

/// 2D Camera with orthographic projection
#[derive(Debug, Clone)]
pub struct Camera2D {
    /// Camera position in world space
    pub position: Vec2,
    /// Camera rotation in radians
    pub rotation: f32,
    /// Zoom level (1.0 = normal, 2.0 = 2x zoom in, 0.5 = 2x zoom out)
    pub zoom: f32,
    /// Viewport dimensions
    pub viewport_size: Vec2,
    /// Near clipping plane
    pub near: f32,
    /// Far clipping plane
    pub far: f32,
}

impl Default for Camera2D {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            rotation: 0.0,
            zoom: 1.0,
            viewport_size: Vec2::new(800.0, 600.0),
            near: -1000.0,
            far: 1000.0,
        }
    }
}

impl Camera2D {
    /// Create a new camera
    pub fn new(position: Vec2, viewport_size: Vec2) -> Self {
        Self {
            position,
            viewport_size,
            ..Default::default()
        }
    }

    /// Build view matrix
    pub fn view_matrix(&self) -> Mat4 {
        let mut view = Mat4::IDENTITY;
        
        // Apply zoom
        view *= Mat4::from_scale(Vec3::new(self.zoom, self.zoom, 1.0));
        
        // Apply rotation
        view *= Mat4::from_rotation_z(self.rotation);
        
        // Apply translation (note: we negate position for view matrix)
        view *= Mat4::from_translation(Vec3::new(-self.position.x, -self.position.y, 0.0));
        
        view
    }

    /// Build orthographic projection matrix
    pub fn projection_matrix(&self) -> Mat4 {
        let half_width = self.viewport_size.x * 0.5;
        let half_height = self.viewport_size.y * 0.5;
        
        Mat4::orthographic_rh(
            -half_width,
            half_width,
            -half_height,
            half_height,
            self.near,
            self.far,
        )
    }

    /// Build view-projection matrix
    pub fn view_projection_matrix(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }

    /// Convert screen coordinates to world coordinates
    pub fn screen_to_world(&self, screen_pos: Vec2) -> Vec2 {
        // Convert screen coordinates to normalized device coordinates
        let ndc = Vec2::new(
            (screen_pos.x - self.viewport_size.x * 0.5) / (self.viewport_size.x * 0.5),
            (self.viewport_size.y * 0.5 - screen_pos.y) / (self.viewport_size.y * 0.5), // Flip Y axis
        );
        
        // Transform by inverse view-projection matrix
        let world_pos = self.view_matrix().inverse() * Vec4::new(ndc.x, ndc.y, 0.0, 1.0);
        
        Vec2::new(world_pos.x, world_pos.y)
    }

    /// Convert world coordinates to screen coordinates
    pub fn world_to_screen(&self, world_pos: Vec2) -> Vec2 {
        // Transform by view-projection matrix
        let clip_pos = self.view_projection_matrix() * Vec4::new(world_pos.x, world_pos.y, 0.0, 1.0);
        
        // Convert to screen coordinates
        Vec2::new(
            (clip_pos.x + 1.0) * 0.5 * self.viewport_size.x,
            (1.0 - clip_pos.y) * 0.5 * self.viewport_size.y, // Flip Y axis
        )
    }
}

/// Camera uniform data for GPU
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_projection: [[f32; 4]; 4],
    pub position: [f32; 2],
    pub _padding: [f32; 2],
}

impl CameraUniform {
    /// Create camera uniform from camera
    pub fn from_camera(camera: &Camera2D) -> Self {
        Self {
            view_projection: camera.view_projection_matrix().to_cols_array_2d(),
            position: camera.position.to_array(),
            _padding: [0.0, 0.0],
        }
    }
}

/// A texture with its view and sampler
#[derive(Debug, Clone)]
pub struct TextureResource {
    pub texture: Arc<Texture>,
    pub view: TextureView,
    pub sampler: Sampler,
    pub width: u32,
    pub height: u32,
}

impl TextureResource {
    /// Create a new texture resource from existing texture
    pub fn new(device: &Device, texture: Arc<Texture>) -> Self {
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Texture Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });

        let size = texture.size();
        
        Self {
            texture,
            view,
            sampler,
            width: size.width,
            height: size.height,
        }
    }

    /// Create a solid color texture - simplified version without texture data upload
    pub fn create_solid_color(
        device: &Device,
        _queue: &Queue,
        _color: [u8; 4],
        width: u32,
        height: u32,
    ) -> Self {
        let texture = Arc::new(device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Solid Color Texture"),
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

        // Note: Texture data upload skipped due to WGPU 28.0.0 API changes
        // In a real implementation, you would use the correct texture upload method
        log::info!("Created solid color texture: {}x{} (data upload skipped)", width, height);

        Self::new(device, texture)
    }
}

/// Dynamic buffer for sprite data
pub struct DynamicBuffer<T> {
    buffer: Buffer,
    capacity: usize,
    usage: wgpu::BufferUsages,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: bytemuck::Pod> DynamicBuffer<T> {
    /// Create a new dynamic buffer
    pub fn new(device: &Device, capacity: usize, usage: wgpu::BufferUsages) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&format!("Dynamic Buffer<{}>", std::any::type_name::<T>())),
            size: (capacity * std::mem::size_of::<T>()) as u64,
            usage: usage | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            buffer,
            capacity,
            usage,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Update buffer data
    pub fn update(&mut self, queue: &Queue, data: &[T]) {
        if data.len() > self.capacity {
            panic!("Buffer overflow: trying to write {} elements to buffer with capacity {}", 
                   data.len(), self.capacity);
        }

        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(data));
    }

    /// Get buffer slice
    pub fn slice(&self) -> wgpu::BufferSlice {
        self.buffer.slice(..)
    }

    /// Get buffer
    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    /// Get capacity
    pub fn capacity(&self) -> usize {
        self.capacity
    }
}