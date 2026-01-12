//! Asset management system for the Insiculous 2D engine.
//!
//! Provides a unified interface for loading and managing game assets including
//! textures, audio (future), and other resources.
//!
//! # Example
//!
//! ```ignore
//! use engine_core::prelude::*;
//!
//! struct MyGame {
//!     player_texture: TextureHandle,
//! }
//!
//! impl Game for MyGame {
//!     fn init(&mut self, ctx: &mut GameContext) {
//!         // Load a texture from file
//!         self.player_texture = ctx.assets.load_texture("assets/player.png").unwrap();
//!     }
//!
//!     fn update(&mut self, ctx: &mut GameContext) {
//!         // Use the texture handle for sprite rendering
//!     }
//! }
//! ```

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use renderer::{
    TextureManager, TextureHandle, TextureResource, TextureLoadConfig, TextureError,
};

// Re-export wgpu types from renderer
use renderer::wgpu::{Device, Queue};

/// Asset loading errors
#[derive(Debug, thiserror::Error)]
pub enum AssetError {
    #[error("Texture error: {0}")]
    Texture(#[from] TextureError),

    #[error("Asset not found: {0}")]
    NotFound(String),

    #[error("Asset manager not initialized")]
    NotInitialized,
}

/// Configuration for the asset manager
#[derive(Debug, Clone)]
pub struct AssetConfig {
    /// Base path for asset loading (relative to the executable)
    pub base_path: String,
    /// Whether to log asset loading operations
    pub log_loading: bool,
}

impl Default for AssetConfig {
    fn default() -> Self {
        Self {
            base_path: "assets".to_string(),
            log_loading: true,
        }
    }
}

/// Unified asset manager for loading and managing game resources.
///
/// The AssetManager provides a convenient interface for loading textures
/// and other game assets. It handles caching and resource lifecycle management.
pub struct AssetManager {
    texture_manager: TextureManager,
    config: AssetConfig,
}

impl AssetManager {
    /// Create a new asset manager with the given WGPU device and queue
    pub fn new(device: Arc<Device>, queue: Arc<Queue>) -> Self {
        Self {
            texture_manager: TextureManager::new(device, queue),
            config: AssetConfig::default(),
        }
    }

    /// Create a new asset manager with custom configuration
    pub fn with_config(device: Arc<Device>, queue: Arc<Queue>, config: AssetConfig) -> Self {
        Self {
            texture_manager: TextureManager::new(device, queue),
            config,
        }
    }

    /// Load a texture from a file path
    ///
    /// The path can be absolute or relative. If relative, it will be resolved
    /// against the asset manager's base path.
    ///
    /// # Example
    /// ```ignore
    /// let handle = assets.load_texture("player.png")?;
    /// // Or with full path:
    /// let handle = assets.load_texture("sprites/enemies/boss.png")?;
    /// ```
    pub fn load_texture<P: AsRef<Path>>(&mut self, path: P) -> Result<TextureHandle, AssetError> {
        let path = path.as_ref();

        // Resolve path against base path if relative
        let full_path = if path.is_relative() {
            Path::new(&self.config.base_path).join(path)
        } else {
            path.to_path_buf()
        };

        if self.config.log_loading {
            log::info!("Loading texture: {:?}", full_path);
        }

        let handle = self.texture_manager.load_texture(&full_path, TextureLoadConfig::default())?;

        Ok(handle)
    }

    /// Load a texture with custom configuration
    pub fn load_texture_with_config<P: AsRef<Path>>(
        &mut self,
        path: P,
        config: TextureLoadConfig,
    ) -> Result<TextureHandle, AssetError> {
        let path = path.as_ref();
        let full_path = if path.is_relative() {
            Path::new(&self.config.base_path).join(path)
        } else {
            path.to_path_buf()
        };

        if self.config.log_loading {
            log::info!("Loading texture with config: {:?}", full_path);
        }

        let handle = self.texture_manager.load_texture(&full_path, config)?;

        Ok(handle)
    }

    /// Load a texture from raw bytes (file contents)
    ///
    /// Useful for loading textures from embedded assets or network resources.
    pub fn load_texture_from_bytes(&mut self, bytes: &[u8]) -> Result<TextureHandle, AssetError> {
        let handle = self.texture_manager.load_texture_from_bytes(bytes, TextureLoadConfig::default())?;
        Ok(handle)
    }

    /// Create a solid color texture
    ///
    /// Useful for placeholder textures or colored rectangles.
    pub fn create_solid_color(
        &mut self,
        width: u32,
        height: u32,
        color: [u8; 4],
    ) -> Result<TextureHandle, AssetError> {
        let handle = self.texture_manager.create_solid_color(width, height, color)?;
        Ok(handle)
    }

    /// Create a checkerboard pattern texture
    ///
    /// Useful for debugging or placeholder textures.
    pub fn create_checkerboard(
        &mut self,
        width: u32,
        height: u32,
        color1: [u8; 4],
        color2: [u8; 4],
        check_size: u32,
    ) -> Result<TextureHandle, AssetError> {
        let handle = self.texture_manager.create_checkerboard(width, height, color1, color2, check_size)?;
        Ok(handle)
    }

    /// Create a glyph texture from grayscale bitmap data
    ///
    /// Converts a grayscale alpha bitmap (one byte per pixel) to an RGBA texture
    /// where RGB are set to the provided color and A is the grayscale value.
    /// This is used for rendering text glyphs.
    pub fn create_glyph_texture(
        &mut self,
        width: u32,
        height: u32,
        grayscale: &[u8],
        color: [u8; 3],
    ) -> Result<TextureHandle, AssetError> {
        if width == 0 || height == 0 {
            // Return the white texture for empty glyphs (like spaces)
            return Ok(TextureHandle { id: 0 });
        }

        // Convert grayscale to RGBA where RGB = color and A = grayscale alpha
        let mut rgba = Vec::with_capacity((width * height * 4) as usize);
        for &alpha in grayscale {
            rgba.push(color[0]); // R
            rgba.push(color[1]); // G
            rgba.push(color[2]); // B
            rgba.push(alpha);    // A from grayscale
        }

        let handle = self.texture_manager.load_texture_from_rgba(
            width,
            height,
            &rgba,
            TextureLoadConfig::default(),
        )?;
        Ok(handle)
    }

    /// Get a texture resource by handle
    pub fn get_texture(&self, handle: TextureHandle) -> Option<&TextureResource> {
        self.texture_manager.get_texture(handle)
    }

    /// Check if a texture exists
    pub fn has_texture(&self, handle: TextureHandle) -> bool {
        self.texture_manager.has_texture(handle)
    }

    /// Unload a texture, freeing GPU resources
    pub fn unload_texture(&mut self, handle: TextureHandle) -> bool {
        self.texture_manager.remove_texture(handle).is_some()
    }

    /// Get the number of loaded textures
    pub fn texture_count(&self) -> usize {
        self.texture_manager.texture_count()
    }

    /// Get all texture handles
    pub fn texture_handles(&self) -> Vec<TextureHandle> {
        self.texture_manager.texture_handles()
    }

    /// Get all textures as a HashMap for rendering
    ///
    /// This is used internally to pass textures to the sprite renderer.
    pub fn textures(&self) -> &HashMap<TextureHandle, TextureResource> {
        self.texture_manager.textures()
    }

    /// Get a cloned HashMap of all textures
    pub fn textures_cloned(&self) -> HashMap<TextureHandle, TextureResource> {
        self.texture_manager.textures_cloned()
    }

    /// Set the base path for asset loading
    pub fn set_base_path(&mut self, path: impl Into<String>) {
        self.config.base_path = path.into();
    }

    /// Get the current base path
    pub fn base_path(&self) -> &str {
        &self.config.base_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_config_default() {
        let config = AssetConfig::default();
        assert_eq!(config.base_path, "assets");
        assert!(config.log_loading);
    }

    #[test]
    fn test_asset_error_display() {
        let err = AssetError::NotFound("player.png".to_string());
        assert!(format!("{}", err).contains("player.png"));
    }
}
