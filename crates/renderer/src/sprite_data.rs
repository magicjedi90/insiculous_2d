//! Sprite data structures and management

use glam::{Vec2, Vec3, Vec4};
use std::sync::Arc;
use wgpu::{Device, Queue, Texture, TextureView, Sampler, Buffer};
use crate::texture::SamplerConfig;

// Re-export Camera2D and CameraUniform from common crate
pub use common::{Camera, camera::CameraUniform};

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
    /// Layer depth for sorting (and depth-test once HDR pipeline is enabled)
    pub depth: f32,
    /// Emissive intensity — 0.0 = no glow, >0.0 amplifies RGB above 1.0 so bloom picks it up
    pub emissive: f32,
}

impl SpriteInstance {
    /// Create a new sprite instance with no emission
    pub fn new(
        position: Vec2,
        rotation: f32,
        scale: Vec2,
        tex_region: [f32; 4],
        color: Vec4,
        depth: f32,
    ) -> Self {
        Self::with_emissive(position, rotation, scale, tex_region, color, depth, 0.0)
    }

    /// Create a new sprite instance with explicit emissive intensity
    pub fn with_emissive(
        position: Vec2,
        rotation: f32,
        scale: Vec2,
        tex_region: [f32; 4],
        color: Vec4,
        depth: f32,
        emissive: f32,
    ) -> Self {
        Self {
            position: position.to_array(),
            rotation,
            scale: scale.to_array(),
            tex_region,
            color: color.to_array(),
            depth,
            emissive,
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
                // Emissive intensity
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 14]>() as wgpu::BufferAddress,
                    shader_location: 9,
                    format: wgpu::VertexFormat::Float32,
                },
            ],
        }
    }
}

// Note: Camera2D and CameraUniform are now re-exported from common crate
// This eliminates ~100 lines of duplicated code

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
        let sampler = SamplerConfig::default().create_sampler(device, Some("Texture Sampler"));

        let size = texture.size();
        
        Self {
            texture,
            view,
            sampler,
            width: size.width,
            height: size.height,
        }
    }

}

/// Dynamic buffer for sprite data.
///
/// Grows on demand: when [`update`](Self::update) receives more elements than
/// the current capacity, the GPU buffer is recreated at the next power of two
/// and the capacity updated. It never shrinks.
pub struct DynamicBuffer<T> {
    buffer: Buffer,
    capacity: usize,
    usage: wgpu::BufferUsages,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: bytemuck::Pod> DynamicBuffer<T> {
    /// Create a new dynamic buffer with the given initial capacity (in elements)
    pub fn new(device: &Device, capacity: usize, usage: wgpu::BufferUsages) -> Self {
        Self {
            buffer: Self::create_buffer(device, capacity, usage),
            capacity,
            usage,
            _phantom: std::marker::PhantomData,
        }
    }

    fn create_buffer(device: &Device, capacity: usize, usage: wgpu::BufferUsages) -> Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&format!("Dynamic Buffer<{}>", std::any::type_name::<T>())),
            size: (capacity * std::mem::size_of::<T>()) as u64,
            usage: usage | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    /// Update buffer data, growing the GPU buffer if `data` exceeds capacity
    pub fn update(&mut self, device: &Device, queue: &Queue, data: &[T]) {
        if data.len() > self.capacity {
            let new_capacity = data.len().next_power_of_two();
            log::debug!(
                "Growing Dynamic Buffer<{}> from {} to {} elements",
                std::any::type_name::<T>(),
                self.capacity,
                new_capacity
            );
            self.buffer = Self::create_buffer(device, new_capacity, self.usage);
            self.capacity = new_capacity;
        }

        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(data));
    }

    /// Get buffer slice
    pub fn slice(&self) -> wgpu::BufferSlice<'_> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use glam::{Vec2, Vec3, Vec4, Mat4};

    // ==================== SpriteVertex Tests ====================

    #[test]
    fn test_sprite_vertex_new() {
        let vertex = SpriteVertex::new(
            Vec3::new(1.0, 2.0, 3.0),
            Vec2::new(0.5, 0.75),
            Vec4::new(1.0, 0.0, 0.0, 1.0),
        );

        assert_eq!(vertex.position, [1.0, 2.0, 3.0]);
        assert_eq!(vertex.tex_coords, [0.5, 0.75]);
        assert_eq!(vertex.color, [1.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn test_sprite_vertex_bytemuck_cast() {
        let vertex = SpriteVertex::new(Vec3::ZERO, Vec2::ZERO, Vec4::ONE);
        // Verify bytemuck traits work
        let _bytes: &[u8] = bytemuck::bytes_of(&vertex);
        assert_eq!(std::mem::size_of::<SpriteVertex>(), 36); // 3*4 + 2*4 + 4*4 = 36 bytes
    }

    #[test]
    fn test_sprite_vertex_desc_attributes() {
        let desc = SpriteVertex::desc();
        assert_eq!(desc.step_mode, wgpu::VertexStepMode::Vertex);
        assert_eq!(desc.attributes.len(), 3); // position, tex_coords, color
        assert_eq!(desc.array_stride, 36);
    }

    // ==================== SpriteInstance Tests ====================

    #[test]
    fn test_sprite_instance_new() {
        let instance = SpriteInstance::new(
            Vec2::new(100.0, 200.0),
            1.5,
            Vec2::new(2.0, 3.0),
            [0.0, 0.0, 1.0, 1.0],
            Vec4::new(1.0, 0.5, 0.0, 1.0),
            5.0,
        );

        assert_eq!(instance.position, [100.0, 200.0]);
        assert!((instance.rotation - 1.5).abs() < 0.0001);
        assert_eq!(instance.scale, [2.0, 3.0]);
        assert_eq!(instance.tex_region, [0.0, 0.0, 1.0, 1.0]);
        assert_eq!(instance.color, [1.0, 0.5, 0.0, 1.0]);
        assert_eq!(instance.depth, 5.0);
        assert_eq!(instance.emissive, 0.0);
    }

    #[test]
    fn test_sprite_instance_with_emissive() {
        let instance = SpriteInstance::with_emissive(
            Vec2::new(10.0, 20.0),
            0.0,
            Vec2::ONE,
            [0.0, 0.0, 1.0, 1.0],
            Vec4::ONE,
            0.0,
            2.5,
        );
        assert_eq!(instance.emissive, 2.5);
    }

    #[test]
    fn test_sprite_instance_bytemuck_cast() {
        let instance = SpriteInstance::new(
            Vec2::ZERO,
            0.0,
            Vec2::ONE,
            [0.0, 0.0, 1.0, 1.0],
            Vec4::ONE,
            0.0,
        );
        let _bytes: &[u8] = bytemuck::bytes_of(&instance);
        // 2*4 + 1*4 + 2*4 + 4*4 + 4*4 + 1*4 + 1*4 = 60 bytes
        assert_eq!(std::mem::size_of::<SpriteInstance>(), 60);
    }

    #[test]
    fn test_sprite_instance_desc_attributes() {
        let desc = SpriteInstance::desc();
        assert_eq!(desc.step_mode, wgpu::VertexStepMode::Instance);
        assert_eq!(desc.attributes.len(), 7); // + emissive
        assert_eq!(desc.array_stride, 60);
    }

    // ==================== Camera2D Tests ====================

    #[test]
    fn test_camera2d_default() {
        let camera = Camera::default();
        assert_eq!(camera.position, Vec2::ZERO);
        assert_eq!(camera.rotation, 0.0);
        assert_eq!(camera.zoom, 1.0);
        assert_eq!(camera.viewport_size, Vec2::new(800.0, 600.0));
        assert_eq!(camera.near, -1000.0);
        assert_eq!(camera.far, 1000.0);
    }

    #[test]
    fn test_camera2d_new() {
        let camera = Camera::new(Vec2::new(100.0, 200.0), Vec2::new(1920.0, 1080.0));
        assert_eq!(camera.position, Vec2::new(100.0, 200.0));
        assert_eq!(camera.viewport_size, Vec2::new(1920.0, 1080.0));
        // Other fields should be default
        assert_eq!(camera.rotation, 0.0);
        assert_eq!(camera.zoom, 1.0);
    }

    #[test]
    fn test_camera2d_view_matrix_identity_at_origin() {
        let camera = Camera::default();
        let view = camera.view_matrix();
        // At origin with no rotation and zoom 1.0, view should be identity
        let identity = Mat4::IDENTITY;
        for i in 0..4 {
            for j in 0..4 {
                assert!((view.col(i)[j] - identity.col(i)[j]).abs() < 0.0001,
                    "Mismatch at [{},{}]: {} vs {}", i, j, view.col(i)[j], identity.col(i)[j]);
            }
        }
    }

    #[test]
    fn test_camera2d_view_matrix_with_position() {
        let camera = Camera {
            position: Vec2::new(100.0, 50.0),
            ..Camera::default()
        };
        let view = camera.view_matrix();

        // Transform a point at (100, 50) - should become (0, 0) in view space
        let point = Vec4::new(100.0, 50.0, 0.0, 1.0);
        let transformed = view * point;
        assert!((transformed.x).abs() < 0.0001);
        assert!((transformed.y).abs() < 0.0001);
    }

    #[test]
    fn test_camera2d_view_matrix_with_zoom() {
        let camera = Camera {
            zoom: 2.0, // 2x zoom in
            ..Camera::default()
        };
        let view = camera.view_matrix();

        // A point at (10, 10) should appear at (20, 20) after zoom
        let point = Vec4::new(10.0, 10.0, 0.0, 1.0);
        let transformed = view * point;
        assert!((transformed.x - 20.0).abs() < 0.0001);
        assert!((transformed.y - 20.0).abs() < 0.0001);
    }

    #[test]
    fn test_camera2d_projection_matrix() {
        let camera = Camera::default();
        let proj = camera.projection_matrix();

        // For an 800x600 viewport, half extents are 400x300
        // A point at (400, 300) should map to (1, 1) in NDC (right-top)
        let point = Vec4::new(400.0, 300.0, 0.0, 1.0);
        let projected = proj * point;
        assert!((projected.x - 1.0).abs() < 0.0001);
        assert!((projected.y - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_camera2d_view_projection_combines_both() {
        let camera = Camera::default();
        let vp = camera.view_projection_matrix();
        let expected = camera.projection_matrix() * camera.view_matrix();

        for i in 0..4 {
            for j in 0..4 {
                assert!((vp.col(i)[j] - expected.col(i)[j]).abs() < 0.0001);
            }
        }
    }

    #[test]
    fn test_camera2d_screen_to_world_center() {
        let camera = Camera::default(); // 800x600 viewport, position at origin
        // Screen center (400, 300) should map to world origin (0, 0)
        let world_pos = camera.screen_to_world(Vec2::new(400.0, 300.0));
        assert!((world_pos.x).abs() < 1.0, "x: {}", world_pos.x);
        assert!((world_pos.y).abs() < 1.0, "y: {}", world_pos.y);
    }

    #[test]
    fn test_camera2d_world_to_screen_origin() {
        let camera = Camera::default(); // 800x600 viewport
        // World origin (0, 0) should map to screen center (400, 300)
        let screen_pos = camera.world_to_screen(Vec2::ZERO);
        assert!((screen_pos.x - 400.0).abs() < 1.0, "x: {}", screen_pos.x);
        assert!((screen_pos.y - 300.0).abs() < 1.0, "y: {}", screen_pos.y);
    }

    #[test]
    fn test_camera2d_world_to_screen_corners() {
        let camera = Camera::default(); // 800x600 viewport

        // World top-right corner (400, 300) should map to screen top-right (800, 0)
        let top_right = camera.world_to_screen(Vec2::new(400.0, 300.0));
        assert!((top_right.x - 800.0).abs() < 1.0, "top_right.x: {}", top_right.x);
        assert!((top_right.y - 0.0).abs() < 1.0, "top_right.y: {}", top_right.y);

        // World bottom-left corner (-400, -300) should map to screen bottom-left (0, 600)
        let bottom_left = camera.world_to_screen(Vec2::new(-400.0, -300.0));
        assert!((bottom_left.x - 0.0).abs() < 1.0, "bottom_left.x: {}", bottom_left.x);
        assert!((bottom_left.y - 600.0).abs() < 1.0, "bottom_left.y: {}", bottom_left.y);
    }

    // ==================== CameraUniform Tests ====================

    #[test]
    fn test_camera_uniform_from_camera() {
        let camera = Camera::new(Vec2::new(50.0, 100.0), Vec2::new(800.0, 600.0));
        let uniform = CameraUniform::from_camera(&camera);

        assert_eq!(uniform.position, [50.0, 100.0]);
        assert_eq!(uniform._padding, [0.0, 0.0]);

        // Verify view_projection matches camera's
        let expected_vp = camera.view_projection_matrix().to_cols_array_2d();
        for (actual_col, expected_col) in uniform.view_projection.iter().zip(expected_vp.iter()) {
            for (actual, expected) in actual_col.iter().zip(expected_col.iter()) {
                assert!((actual - expected).abs() < 0.0001);
            }
        }
    }

    #[test]
    fn test_camera_uniform_bytemuck() {
        let camera = Camera::default();
        let uniform = CameraUniform::from_camera(&camera);
        // Verify bytemuck traits work
        let _bytes: &[u8] = bytemuck::bytes_of(&uniform);
        // 16 floats for matrix (64) + 2 floats position (8) + 2 floats padding (8) = 80 bytes
        assert_eq!(std::mem::size_of::<CameraUniform>(), 80);
    }
}