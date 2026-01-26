//! Scene viewport rendering for the editor.
//!
//! The SceneViewport handles rendering game entities within the scene view panel,
//! managing camera transforms, and converting between screen and world coordinates.

use common::{Camera, Rect};
use glam::Vec2;
use renderer::sprite::{Sprite, SpriteBatcher};
use renderer::texture::TextureHandle;

/// Manages rendering the game world within the scene view panel.
///
/// The SceneViewport coordinates:
/// - Camera viewport calculation from panel bounds
/// - World-to-screen coordinate transformations
/// - Entity sprite generation within viewport region
#[derive(Debug, Clone)]
pub struct SceneViewport {
    /// Current viewport bounds in screen space (from DockPanel)
    viewport_bounds: Rect,
    /// Camera position in world space (pan offset)
    camera_position: Vec2,
    /// Camera zoom level (1.0 = normal, 2.0 = zoomed in 2x)
    camera_zoom: f32,
    /// Target camera position for smooth interpolation
    target_camera_position: Vec2,
    /// Target zoom for smooth interpolation
    target_camera_zoom: f32,
    /// Interpolation speed (0.0-1.0, higher = snappier)
    interpolation_speed: f32,
}

impl Default for SceneViewport {
    fn default() -> Self {
        Self::new()
    }
}

impl SceneViewport {
    /// Create a new scene viewport with default settings.
    pub fn new() -> Self {
        Self {
            viewport_bounds: Rect::default(),
            camera_position: Vec2::ZERO,
            camera_zoom: 1.0,
            target_camera_position: Vec2::ZERO,
            target_camera_zoom: 1.0,
            interpolation_speed: 0.15,
        }
    }

    /// Set the viewport bounds (from the scene view panel content bounds).
    pub fn set_viewport_bounds(&mut self, bounds: Rect) {
        self.viewport_bounds = bounds;
    }

    /// Get the current viewport bounds.
    pub fn viewport_bounds(&self) -> Rect {
        self.viewport_bounds
    }

    /// Get the viewport center in screen coordinates.
    pub fn viewport_center(&self) -> Vec2 {
        Vec2::new(
            self.viewport_bounds.x + self.viewport_bounds.width * 0.5,
            self.viewport_bounds.y + self.viewport_bounds.height * 0.5,
        )
    }

    /// Get the viewport size.
    pub fn viewport_size(&self) -> Vec2 {
        Vec2::new(self.viewport_bounds.width, self.viewport_bounds.height)
    }

    // ================== Camera Methods ==================

    /// Get the current camera position.
    pub fn camera_position(&self) -> Vec2 {
        self.camera_position
    }

    /// Set the camera position directly (no interpolation).
    pub fn set_camera_position(&mut self, position: Vec2) {
        self.camera_position = position;
        self.target_camera_position = position;
    }

    /// Set the target camera position (will interpolate).
    pub fn set_target_camera_position(&mut self, position: Vec2) {
        self.target_camera_position = position;
    }

    /// Pan the camera by a delta amount (in world space).
    pub fn pan(&mut self, delta: Vec2) {
        self.target_camera_position += delta;
    }

    /// Pan the camera immediately (no interpolation).
    pub fn pan_immediate(&mut self, delta: Vec2) {
        self.camera_position += delta;
        self.target_camera_position = self.camera_position;
    }

    /// Get the current camera zoom level.
    pub fn camera_zoom(&self) -> f32 {
        self.camera_zoom
    }

    /// Set the camera zoom directly (no interpolation).
    pub fn set_camera_zoom(&mut self, zoom: f32) {
        let clamped = zoom.clamp(0.1, 10.0);
        self.camera_zoom = clamped;
        self.target_camera_zoom = clamped;
    }

    /// Set the target zoom level (will interpolate).
    pub fn set_target_zoom(&mut self, zoom: f32) {
        self.target_camera_zoom = zoom.clamp(0.1, 10.0);
    }

    /// Zoom the camera by a factor centered on a screen position.
    ///
    /// The zoom is centered on the given screen position so the world point
    /// under the cursor stays fixed.
    pub fn zoom_at(&mut self, factor: f32, screen_pos: Vec2) {
        let old_zoom = self.target_camera_zoom;
        let new_zoom = (old_zoom * factor).clamp(0.1, 10.0);

        // Calculate world position under cursor before zoom
        let world_before = self.screen_to_world(screen_pos);

        // Apply new zoom
        self.target_camera_zoom = new_zoom;

        // Calculate world position under cursor after zoom (with new zoom but old camera pos)
        let temp_zoom = self.camera_zoom;
        self.camera_zoom = new_zoom;
        let world_after = self.screen_to_world(screen_pos);
        self.camera_zoom = temp_zoom;

        // Adjust camera position to keep world_before at the same screen position
        self.target_camera_position += world_before - world_after;
    }

    /// Reset the camera to default view.
    pub fn reset_camera(&mut self) {
        self.target_camera_position = Vec2::ZERO;
        self.target_camera_zoom = 1.0;
    }

    /// Reset camera immediately (no interpolation).
    pub fn reset_camera_immediate(&mut self) {
        self.camera_position = Vec2::ZERO;
        self.camera_zoom = 1.0;
        self.target_camera_position = Vec2::ZERO;
        self.target_camera_zoom = 1.0;
    }

    /// Update camera interpolation. Call each frame.
    pub fn update(&mut self, _delta_time: f32) {
        // Simple lerp interpolation
        let t = self.interpolation_speed;
        self.camera_position = self.camera_position.lerp(self.target_camera_position, t);
        self.camera_zoom = self.camera_zoom + (self.target_camera_zoom - self.camera_zoom) * t;
    }

    /// Set interpolation speed (0.0 = no movement, 1.0 = instant).
    pub fn set_interpolation_speed(&mut self, speed: f32) {
        self.interpolation_speed = speed.clamp(0.0, 1.0);
    }

    // ================== Coordinate Conversion ==================

    /// Convert screen coordinates to world coordinates.
    ///
    /// Screen coordinates have origin at top-left of window.
    /// World coordinates have origin at camera position.
    pub fn screen_to_world(&self, screen_pos: Vec2) -> Vec2 {
        let viewport_center = self.viewport_center();

        // Convert screen position relative to viewport center
        let relative = Vec2::new(
            screen_pos.x - viewport_center.x,
            viewport_center.y - screen_pos.y, // Flip Y for world coords
        );

        // Scale by zoom and add camera offset
        relative / self.camera_zoom + self.camera_position
    }

    /// Convert world coordinates to screen coordinates.
    ///
    /// Returns screen position with origin at top-left of window.
    pub fn world_to_screen(&self, world_pos: Vec2) -> Vec2 {
        let viewport_center = self.viewport_center();

        // Convert world position relative to camera
        let relative = (world_pos - self.camera_position) * self.camera_zoom;

        // Convert to screen coordinates (flip Y)
        Vec2::new(
            viewport_center.x + relative.x,
            viewport_center.y - relative.y,
        )
    }

    /// Check if a screen position is within the viewport bounds.
    pub fn contains_screen_point(&self, screen_pos: Vec2) -> bool {
        self.viewport_bounds.contains(screen_pos)
    }

    /// Get the visible world bounds (min_x, min_y, max_x, max_y).
    pub fn visible_world_bounds(&self) -> (f32, f32, f32, f32) {
        let half_w = self.viewport_bounds.width * 0.5 / self.camera_zoom;
        let half_h = self.viewport_bounds.height * 0.5 / self.camera_zoom;

        (
            self.camera_position.x - half_w,
            self.camera_position.y - half_h,
            self.camera_position.x + half_w,
            self.camera_position.y + half_h,
        )
    }

    /// Create a Camera struct for use with the renderer.
    pub fn to_render_camera(&self) -> Camera {
        Camera::new(self.camera_position, self.viewport_size())
            .with_zoom(self.camera_zoom)
    }

    // ================== Entity Rendering ==================

    /// Generate sprites for entities within the viewport.
    ///
    /// Takes entity positions and sizes in world coordinates and generates
    /// sprites positioned correctly for rendering within the viewport.
    pub fn generate_entity_sprite(
        &self,
        world_position: Vec2,
        world_scale: Vec2,
        rotation: f32,
        texture_handle: TextureHandle,
        color: glam::Vec4,
        depth: f32,
    ) -> Sprite {
        // Sprites are positioned in world coordinates - the renderer handles
        // the camera transform. We just need to create sprites with world positions.
        Sprite::new(texture_handle)
            .with_position(world_position)
            .with_scale(world_scale)
            .with_rotation(rotation)
            .with_color(color)
            .with_depth(depth)
    }

    /// Add entity sprites to a batcher for entities with Transform2D and Sprite components.
    ///
    /// This is a convenience method that iterates entity data and generates sprites.
    pub fn batch_entities(
        &self,
        batcher: &mut SpriteBatcher,
        entities: &[(Vec2, Vec2, f32, TextureHandle, glam::Vec4, f32)], // (pos, scale, rot, tex, color, depth)
    ) {
        for (pos, scale, rot, tex, color, depth) in entities {
            let sprite = self.generate_entity_sprite(*pos, *scale, *rot, *tex, *color, *depth);
            batcher.add_sprite(&sprite);
        }
    }

    /// Focus the camera on a world position.
    pub fn focus_on(&mut self, world_pos: Vec2) {
        self.target_camera_position = world_pos;
    }

    /// Focus the camera on multiple positions (center of bounding box).
    pub fn focus_on_bounds(&mut self, positions: &[Vec2]) {
        if positions.is_empty() {
            return;
        }

        let mut min = positions[0];
        let mut max = positions[0];

        for pos in positions {
            min = min.min(*pos);
            max = max.max(*pos);
        }

        let center = (min + max) * 0.5;
        self.target_camera_position = center;

        // Optionally adjust zoom to fit bounds
        let bounds_size = max - min;
        let viewport_size = self.viewport_size();
        if bounds_size.x > 0.0 && bounds_size.y > 0.0 {
            let zoom_x = viewport_size.x / (bounds_size.x + 100.0); // Add padding
            let zoom_y = viewport_size.y / (bounds_size.y + 100.0);
            self.target_camera_zoom = zoom_x.min(zoom_y).clamp(0.1, 10.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_viewport_new() {
        let viewport = SceneViewport::new();
        assert_eq!(viewport.camera_position(), Vec2::ZERO);
        assert_eq!(viewport.camera_zoom(), 1.0);
    }

    #[test]
    fn test_viewport_set_bounds() {
        let mut viewport = SceneViewport::new();
        viewport.set_viewport_bounds(Rect::new(100.0, 50.0, 800.0, 600.0));

        assert_eq!(viewport.viewport_bounds().x, 100.0);
        assert_eq!(viewport.viewport_bounds().y, 50.0);
        assert_eq!(viewport.viewport_size(), Vec2::new(800.0, 600.0));
        assert_eq!(viewport.viewport_center(), Vec2::new(500.0, 350.0));
    }

    #[test]
    fn test_viewport_screen_to_world_no_offset() {
        let mut viewport = SceneViewport::new();
        viewport.set_viewport_bounds(Rect::new(0.0, 0.0, 800.0, 600.0));

        // Center of viewport should map to camera position (origin)
        let world = viewport.screen_to_world(Vec2::new(400.0, 300.0));
        assert!((world.x).abs() < 0.001);
        assert!((world.y).abs() < 0.001);
    }

    #[test]
    fn test_viewport_world_to_screen_no_offset() {
        let mut viewport = SceneViewport::new();
        viewport.set_viewport_bounds(Rect::new(0.0, 0.0, 800.0, 600.0));

        // World origin should map to viewport center
        let screen = viewport.world_to_screen(Vec2::ZERO);
        assert!((screen.x - 400.0).abs() < 0.001);
        assert!((screen.y - 300.0).abs() < 0.001);
    }

    #[test]
    fn test_viewport_coordinate_roundtrip() {
        let mut viewport = SceneViewport::new();
        viewport.set_viewport_bounds(Rect::new(100.0, 50.0, 800.0, 600.0));
        viewport.set_camera_position(Vec2::new(100.0, -50.0));
        viewport.set_camera_zoom(2.0);

        let original_screen = Vec2::new(350.0, 200.0);
        let world = viewport.screen_to_world(original_screen);
        let back_to_screen = viewport.world_to_screen(world);

        assert!((back_to_screen.x - original_screen.x).abs() < 0.001);
        assert!((back_to_screen.y - original_screen.y).abs() < 0.001);
    }

    #[test]
    fn test_viewport_visible_world_bounds() {
        let mut viewport = SceneViewport::new();
        viewport.set_viewport_bounds(Rect::new(0.0, 0.0, 800.0, 600.0));

        let (min_x, min_y, max_x, max_y) = viewport.visible_world_bounds();
        assert_eq!(min_x, -400.0);
        assert_eq!(min_y, -300.0);
        assert_eq!(max_x, 400.0);
        assert_eq!(max_y, 300.0);
    }

    #[test]
    fn test_viewport_visible_world_bounds_with_zoom() {
        let mut viewport = SceneViewport::new();
        viewport.set_viewport_bounds(Rect::new(0.0, 0.0, 800.0, 600.0));
        viewport.set_camera_zoom(2.0);

        let (min_x, min_y, max_x, max_y) = viewport.visible_world_bounds();
        // At 2x zoom, visible area is halved
        assert_eq!(min_x, -200.0);
        assert_eq!(min_y, -150.0);
        assert_eq!(max_x, 200.0);
        assert_eq!(max_y, 150.0);
    }

    #[test]
    fn test_viewport_pan() {
        let mut viewport = SceneViewport::new();
        viewport.pan_immediate(Vec2::new(50.0, -25.0));

        assert_eq!(viewport.camera_position(), Vec2::new(50.0, -25.0));
    }

    #[test]
    fn test_viewport_zoom_clamp() {
        let mut viewport = SceneViewport::new();

        viewport.set_camera_zoom(0.01);
        assert_eq!(viewport.camera_zoom(), 0.1); // Clamped to min

        viewport.set_camera_zoom(100.0);
        assert_eq!(viewport.camera_zoom(), 10.0); // Clamped to max
    }

    #[test]
    fn test_viewport_reset_camera() {
        let mut viewport = SceneViewport::new();
        viewport.set_camera_position(Vec2::new(100.0, 200.0));
        viewport.set_camera_zoom(3.0);

        viewport.reset_camera_immediate();

        assert_eq!(viewport.camera_position(), Vec2::ZERO);
        assert_eq!(viewport.camera_zoom(), 1.0);
    }

    #[test]
    fn test_viewport_contains_screen_point() {
        let mut viewport = SceneViewport::new();
        viewport.set_viewport_bounds(Rect::new(100.0, 50.0, 400.0, 300.0));

        assert!(viewport.contains_screen_point(Vec2::new(200.0, 150.0)));
        assert!(!viewport.contains_screen_point(Vec2::new(50.0, 150.0)));
        assert!(!viewport.contains_screen_point(Vec2::new(200.0, 400.0)));
    }

    #[test]
    fn test_viewport_to_render_camera() {
        let mut viewport = SceneViewport::new();
        viewport.set_viewport_bounds(Rect::new(0.0, 0.0, 1920.0, 1080.0));
        viewport.set_camera_position(Vec2::new(100.0, -50.0));
        viewport.set_camera_zoom(1.5);

        let camera = viewport.to_render_camera();
        assert_eq!(camera.position, Vec2::new(100.0, -50.0));
        assert_eq!(camera.zoom, 1.5);
        assert_eq!(camera.viewport_size, Vec2::new(1920.0, 1080.0));
    }

    #[test]
    fn test_viewport_focus_on() {
        let mut viewport = SceneViewport::new();
        viewport.focus_on(Vec2::new(500.0, 300.0));

        // Target should be set (actual position updates on update())
        viewport.update(0.016);

        // After interpolation, should be moving toward target
        let pos = viewport.camera_position();
        assert!(pos.x > 0.0); // Moving toward 500
        assert!(pos.y > 0.0); // Moving toward 300
    }

    #[test]
    fn test_viewport_focus_on_bounds() {
        let mut viewport = SceneViewport::new();
        viewport.set_viewport_bounds(Rect::new(0.0, 0.0, 800.0, 600.0));

        let positions = vec![
            Vec2::new(-100.0, -50.0),
            Vec2::new(100.0, 50.0),
        ];
        viewport.focus_on_bounds(&positions);

        // Target should be center of bounds
        viewport.update(0.016);
        // Camera should be moving toward (0, 0) - center of bounds
    }
}
