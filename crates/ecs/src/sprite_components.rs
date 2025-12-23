//! Sprite components for ECS integration

use glam::{Vec2, Vec4};

// Re-export types from renderer that we need
use renderer::{Sprite as RendererSprite, Camera2D as RendererCamera2D};

/// Sprite component that defines visual appearance
#[derive(Debug, Clone)]
pub struct Sprite {
    /// Position offset from entity position
    pub offset: Vec2,
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
    /// Texture handle (from renderer::TextureHandle)
    pub texture_handle: u32,
}

impl Default for Sprite {
    fn default() -> Self {
        Self {
            offset: Vec2::ZERO,
            rotation: 0.0,
            scale: Vec2::ONE,
            tex_region: [0.0, 0.0, 1.0, 1.0], // Full texture
            color: Vec4::ONE, // White
            depth: 0.0,
            texture_handle: 0,
        }
    }
}

impl Sprite {
    /// Create a new sprite
    pub fn new(texture_handle: u32) -> Self {
        Self {
            texture_handle,
            ..Default::default()
        }
    }

    /// Set sprite offset
    pub fn with_offset(mut self, offset: Vec2) -> Self {
        self.offset = offset;
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
}



/// Transform component for entity position, rotation, and scale
#[derive(Debug, Clone)]
pub struct Transform2D {
    /// Position in world space
    pub position: Vec2,
    /// Rotation in radians
    pub rotation: f32,
    /// Scale
    pub scale: Vec2,
}

impl Default for Transform2D {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            rotation: 0.0,
            scale: Vec2::ONE,
        }
    }
}

impl Transform2D {
    /// Create a new transform
    pub fn new(position: Vec2) -> Self {
        Self {
            position,
            ..Default::default()
        }
    }

    /// Set rotation
    pub fn with_rotation(mut self, rotation: f32) -> Self {
        self.rotation = rotation;
        self
    }

    /// Set scale
    pub fn with_scale(mut self, scale: Vec2) -> Self {
        self.scale = scale;
        self
    }

    /// Get the transformation matrix
    pub fn matrix(&self) -> glam::Mat3 {
        let cos_r = self.rotation.cos();
        let sin_r = self.rotation.sin();
        
        // Rotation matrix
        let rot = glam::Mat3::from_cols_array(&[
            cos_r, sin_r, 0.0,
            -sin_r, cos_r, 0.0,
            0.0, 0.0, 1.0,
        ]);
        
        // Scale matrix
        let scale = glam::Mat3::from_diagonal(glam::Vec3::new(self.scale.x, self.scale.y, 1.0));
        
        // Translation matrix
        let translate = glam::Mat3::from_cols_array(&[
            1.0, 0.0, 0.0,
            0.0, 1.0, 0.0,
            self.position.x, self.position.y, 1.0,
        ]);
        
        // Combine: T * R * S
        translate * rot * scale
    }

    /// Transform a point by this transform
    pub fn transform_point(&self, point: Vec2) -> Vec2 {
        let transformed = self.matrix() * glam::Vec3::new(point.x, point.y, 1.0);
        Vec2::new(transformed.x, transformed.y)
    }

    /// Get the inverse transformation matrix
    pub fn inverse_matrix(&self) -> glam::Mat3 {
        self.matrix().inverse()
    }
}



/// Camera component for 2D rendering
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
    /// Whether this is the main camera
    pub is_main_camera: bool,
}

impl Default for Camera2D {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            rotation: 0.0,
            zoom: 1.0,
            viewport_size: Vec2::new(800.0, 600.0),
            is_main_camera: false,
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

    /// Set as main camera
    pub fn as_main_camera(mut self) -> Self {
        self.is_main_camera = true;
        self
    }

    /// Set rotation
    pub fn with_rotation(mut self, rotation: f32) -> Self {
        self.rotation = rotation;
        self
    }

    /// Set zoom
    pub fn with_zoom(mut self, zoom: f32) -> Self {
        self.zoom = zoom;
        self
    }

    /// Build view matrix
    pub fn view_matrix(&self) -> glam::Mat4 {
        let mut view = glam::Mat4::IDENTITY;
        
        // Apply zoom
        view *= glam::Mat4::from_scale(glam::Vec3::new(self.zoom, self.zoom, 1.0));
        
        // Apply rotation
        view *= glam::Mat4::from_rotation_z(self.rotation);
        
        // Apply translation (note: we negate position for view matrix)
        view *= glam::Mat4::from_translation(glam::Vec3::new(-self.position.x, -self.position.y, 0.0));
        
        view
    }

    /// Build orthographic projection matrix
    pub fn projection_matrix(&self) -> glam::Mat4 {
        let half_width = self.viewport_size.x * 0.5;
        let half_height = self.viewport_size.y * 0.5;
        
        glam::Mat4::orthographic_rh(
            -half_width,
            half_width,
            -half_height,
            half_height,
            -1000.0,
            1000.0,
        )
    }

    /// Build view-projection matrix
    pub fn view_projection_matrix(&self) -> glam::Mat4 {
        self.projection_matrix() * self.view_matrix()
    }
}



/// Sprite animation component
#[derive(Debug, Clone)]
pub struct SpriteAnimation {
    /// Current frame index
    pub current_frame: usize,
    /// Animation speed (frames per second)
    pub fps: f32,
    /// Whether the animation is playing
    pub playing: bool,
    /// Whether the animation should loop
    pub loop_animation: bool,
    /// Time accumulator for frame timing
    pub time_accumulator: f32,
    /// Texture regions for each frame [x, y, width, height]
    pub frames: Vec<[f32; 4]>,
}

impl Default for SpriteAnimation {
    fn default() -> Self {
        Self {
            current_frame: 0,
            fps: 10.0,
            playing: true,
            loop_animation: true,
            time_accumulator: 0.0,
            frames: vec![[0.0, 0.0, 1.0, 1.0]], // Single frame covering entire texture
        }
    }
}

impl SpriteAnimation {
    /// Create a new sprite animation
    pub fn new(fps: f32, frames: Vec<[f32; 4]>) -> Self {
        Self {
            fps,
            frames,
            ..Default::default()
        }
    }

    /// Set whether to loop
    pub fn with_loop(mut self, loop_animation: bool) -> Self {
        self.loop_animation = loop_animation;
        self
    }

    /// Start playing the animation
    pub fn play(&mut self) {
        self.playing = true;
    }

    /// Pause the animation
    pub fn pause(&mut self) {
        self.playing = false;
    }

    /// Reset to first frame
    pub fn reset(&mut self) {
        self.current_frame = 0;
        self.time_accumulator = 0.0;
    }

    /// Update the animation (should be called every frame)
    pub fn update(&mut self, delta_time: f32) {
        if !self.playing || self.frames.is_empty() {
            return;
        }

        self.time_accumulator += delta_time;
        let frame_duration = 1.0 / self.fps;

        while self.time_accumulator >= frame_duration {
            self.time_accumulator -= frame_duration;
            self.current_frame += 1;

            if self.current_frame >= self.frames.len() {
                if self.loop_animation {
                    self.current_frame = 0;
                } else {
                    self.current_frame = self.frames.len() - 1;
                    self.playing = false;
                }
            }
        }
    }

    /// Get the current frame's texture region
    pub fn current_frame_region(&self) -> [f32; 4] {
        if self.frames.is_empty() {
            [0.0, 0.0, 1.0, 1.0]
        } else {
            self.frames[self.current_frame.min(self.frames.len() - 1)]
        }
    }

    /// Check if animation is complete (for non-looping animations)
    pub fn is_complete(&self) -> bool {
        !self.loop_animation && !self.playing && self.current_frame == self.frames.len().saturating_sub(1)
    }
}



/// Sprite renderer system data
#[derive(Debug, Default)]
pub struct SpriteRenderData {
    /// Sprites to render this frame
    pub sprites: Vec<RendererSprite>,
    /// Camera data
    pub camera: Option<RendererCamera2D>,
}

impl SpriteRenderData {
    /// Create new sprite render data
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a sprite to render
    pub fn add_sprite(&mut self, sprite: RendererSprite) {
        self.sprites.push(sprite);
    }

    /// Set camera
    pub fn set_camera(&mut self, camera: RendererCamera2D) {
        self.camera = Some(camera);
    }

    /// Clear all sprites
    pub fn clear(&mut self) {
        self.sprites.clear();
    }

    /// Get sprite count
    pub fn sprite_count(&self) -> usize {
        self.sprites.len()
    }
}