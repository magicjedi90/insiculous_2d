//! Render manager for handling renderer lifecycle and sprite rendering.
//!
//! This module provides a focused manager for all rendering concerns,
//! following the Single Responsibility Principle.

use std::collections::HashMap;
use std::sync::Arc;

use glam::Vec2;
use winit::window::Window;

use renderer::{
    sprite::{SpriteBatch, SpriteBatcher, SpritePipeline},
    sprite_data::TextureResource,
    texture::TextureHandle,
    wgpu::{Device, Queue},
    Camera, Renderer, RendererError,
};

/// Manages the renderer lifecycle and sprite rendering pipeline.
///
/// This struct encapsulates all rendering-related responsibilities:
/// - Renderer initialization and lifecycle
/// - Sprite pipeline management
/// - Camera configuration
/// - Surface management
pub struct RenderManager {
    /// The WGPU renderer
    renderer: Option<Renderer>,
    /// The sprite rendering pipeline
    sprite_pipeline: Option<SpritePipeline>,
    /// The 2D camera for orthographic projection
    camera: Camera,
}

impl Default for RenderManager {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderManager {
    /// Create a new render manager with default settings.
    pub fn new() -> Self {
        Self {
            renderer: None,
            sprite_pipeline: None,
            camera: Camera::default(),
        }
    }

    /// Initialize the renderer with a window.
    ///
    /// # Arguments
    /// * `window` - The window to render to
    /// * `clear_color` - RGBA clear color for the background
    ///
    /// # Returns
    /// * `Ok(())` on successful initialization
    /// * `Err(RendererError)` if initialization fails
    pub fn init(&mut self, window: Arc<Window>, clear_color: [f32; 4]) -> Result<(), RendererError> {
        // Use pollster for async execution
        let mut renderer = pollster::block_on(renderer::init(window))?;

        renderer.set_clear_color(
            clear_color[0] as f64,
            clear_color[1] as f64,
            clear_color[2] as f64,
            clear_color[3] as f64,
        );

        // Create sprite pipeline with max 1000 sprites per batch
        let sprite_pipeline = SpritePipeline::new(renderer.device_ref(), 1000);

        self.renderer = Some(renderer);
        self.sprite_pipeline = Some(sprite_pipeline);

        log::info!("RenderManager initialized");
        Ok(())
    }

    /// Check if the renderer is initialized.
    pub fn is_initialized(&self) -> bool {
        self.renderer.is_some()
    }

    /// Resize the renderer surface.
    ///
    /// Updates both the renderer surface and camera viewport.
    pub fn resize(&mut self, width: u32, height: u32) {
        if let Some(renderer) = &mut self.renderer {
            renderer.resize(width, height);
        }
        self.camera.viewport_size = Vec2::new(width as f32, height as f32);
    }

    /// Update the camera viewport size based on current window dimensions.
    pub fn update_viewport_from_renderer(&mut self) {
        if let Some(renderer) = &self.renderer {
            let width = renderer.surface_width() as f32;
            let height = renderer.surface_height() as f32;
            if height > 0.0 {
                self.camera.viewport_size = Vec2::new(width, height);
            }
        }
    }

    /// Render sprites using the provided batcher and textures.
    ///
    /// # Arguments
    /// * `batches` - Sprite batches to render
    /// * `textures` - Texture resources for rendering
    ///
    /// # Returns
    /// * `Ok(())` on successful render
    /// * `Err(RendererError)` if rendering fails
    pub fn render(
        &mut self,
        batches: &[&SpriteBatch],
        textures: &HashMap<TextureHandle, TextureResource>,
    ) -> Result<(), RendererError> {
        let renderer = self.renderer.as_mut().ok_or_else(|| {
            RendererError::WindowCreationError("Renderer not initialized".to_string())
        })?;
        let pipeline = self.sprite_pipeline.as_mut().ok_or_else(|| {
            RendererError::WindowCreationError("Sprite pipeline not initialized".to_string())
        })?;

        match renderer.render_with_sprites(pipeline, &self.camera, textures, batches) {
            Ok(_) => Ok(()),
            Err(RendererError::SurfaceError(_)) => {
                // Try to recreate the surface
                if let Err(e) = renderer.recreate_surface() {
                    log::error!("Failed to recreate surface: {}", e);
                    return Err(e);
                }
                log::debug!("Surface recreated after loss");
                Ok(())
            }
            Err(e) => {
                log::error!("Render error: {}", e);
                Err(e)
            }
        }
    }

    /// Render a frame using a SpriteBatcher.
    ///
    /// This is a convenience method that extracts batches from the batcher.
    pub fn render_batcher(
        &mut self,
        batcher: &SpriteBatcher,
        textures: &HashMap<TextureHandle, TextureResource>,
    ) -> Result<(), RendererError> {
        let batches: Vec<SpriteBatch> = batcher.batches().values().cloned().collect();
        let batch_refs: Vec<&SpriteBatch> = batches.iter().collect();
        self.render(&batch_refs, textures)
    }

    /// Perform a basic render without sprites (just clears the screen).
    pub fn render_basic(&mut self) -> Result<(), RendererError> {
        let renderer = self.renderer.as_mut().ok_or_else(|| {
            RendererError::WindowCreationError("Renderer not initialized".to_string())
        })?;

        match renderer.render() {
            Ok(_) => Ok(()),
            Err(RendererError::SurfaceError(_)) => {
                if let Err(e) = renderer.recreate_surface() {
                    log::error!("Failed to recreate surface: {}", e);
                    return Err(e);
                }
                log::debug!("Surface recreated after loss");
                Ok(())
            }
            Err(e) => {
                log::error!("Render error: {}", e);
                Err(e)
            }
        }
    }

    /// Get the GPU device if the renderer is initialized.
    pub fn device(&self) -> Option<Arc<Device>> {
        self.renderer.as_ref().map(|r| r.device())
    }

    /// Get the GPU queue if the renderer is initialized.
    pub fn queue(&self) -> Option<Arc<Queue>> {
        self.renderer.as_ref().map(|r| r.queue())
    }

    /// Get a reference to the device (for pipeline creation).
    pub fn device_ref(&self) -> Option<&Device> {
        self.renderer.as_ref().map(|r| r.device_ref())
    }

    /// Get a reference to the camera.
    pub fn camera(&self) -> &Camera {
        &self.camera
    }

    /// Get a mutable reference to the camera.
    pub fn camera_mut(&mut self) -> &mut Camera {
        &mut self.camera
    }

    /// Set the camera viewport size.
    pub fn set_viewport_size(&mut self, width: f32, height: f32) {
        self.camera.viewport_size = Vec2::new(width, height);
    }

    /// Get the current surface width.
    pub fn surface_width(&self) -> Option<u32> {
        self.renderer.as_ref().map(|r| r.surface_width())
    }

    /// Get the current surface height.
    pub fn surface_height(&self) -> Option<u32> {
        self.renderer.as_ref().map(|r| r.surface_height())
    }

    /// Get adapter info string for debugging.
    pub fn adapter_info(&self) -> Option<String> {
        self.renderer.as_ref().map(|r| r.adapter_info())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_manager_new() {
        let manager = RenderManager::new();
        assert!(!manager.is_initialized());
        assert!(manager.device().is_none());
        assert!(manager.queue().is_none());
    }

    #[test]
    fn test_render_manager_default() {
        let manager = RenderManager::default();
        assert!(!manager.is_initialized());
    }

    #[test]
    fn test_camera_access() {
        let mut manager = RenderManager::new();

        // Test initial camera state - Camera2D::default() sets viewport to 800x600
        assert_eq!(manager.camera().viewport_size, Vec2::new(800.0, 600.0));

        // Test camera mutation
        manager.set_viewport_size(1024.0, 768.0);
        assert_eq!(manager.camera().viewport_size, Vec2::new(1024.0, 768.0));

        // Test mutable access
        manager.camera_mut().position = Vec2::new(100.0, 50.0);
        assert_eq!(manager.camera().position, Vec2::new(100.0, 50.0));
    }

    #[test]
    fn test_resize_without_renderer() {
        let mut manager = RenderManager::new();
        // Should not panic, just updates camera
        manager.resize(1024, 768);
        assert_eq!(manager.camera().viewport_size, Vec2::new(1024.0, 768.0));
    }
}
