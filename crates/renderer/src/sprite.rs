//! 2D sprite rendering: sprite data, CPU batching, and the GPU pipeline.
//!
//! - [`Sprite`] (this module) — what a game asks to draw
//! - [`SpriteBatch`] / [`SpriteBatcher`] — group sprites by texture before upload
//! - [`SpritePipeline`] — instanced GPU rendering into the HDR target

use glam::{Vec2, Vec4};

use crate::sprite_data::SpriteInstance;
use crate::texture::TextureHandle;

mod batch;
mod instance_cache;
mod pipeline;

pub use batch::{SpriteBatch, SpriteBatcher};
pub use instance_cache::InstanceCache;
pub use pipeline::SpritePipeline;

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
    /// Emissive intensity — 0.0 disables glow, values above 0.0 produce HDR output that bloom picks up
    pub emissive: f32,
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
            emissive: 0.0,
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

    /// Set emissive intensity (0.0 disables glow, larger values bloom more strongly)
    pub fn with_emissive(mut self, emissive: f32) -> Self {
        self.emissive = emissive;
        self
    }

    /// Convert to sprite instance for batching
    pub fn to_instance(&self) -> SpriteInstance {
        SpriteInstance::with_emissive(
            self.position,
            self.rotation,
            self.scale,
            self.tex_region,
            self.color,
            self.depth,
            self.emissive,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::{Vec2, Vec4};

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
}
