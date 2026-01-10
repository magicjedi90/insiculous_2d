//! Texture loading and management system
//!
//! Provides comprehensive texture loading, caching, and management for the renderer.
//! Supports loading from files (PNG, JPEG, BMP, GIF) and raw RGBA data.

use std::sync::Arc;
use std::path::Path;
use std::collections::HashMap;
use wgpu::{Device, Queue, Sampler, TextureFormat, Extent3d};
use thiserror::Error;
use image::GenericImageView;

use crate::sprite_data::TextureResource;

/// Texture loading errors
#[derive(Debug, Error)]
pub enum TextureError {
    #[error("Failed to load image: {0}")]
    ImageLoadError(String),
    #[error("Failed to create texture: {0}")]
    TextureCreationError(String),
    #[error("Texture not found: {0}")]
    TextureNotFound(String),
    #[error("Invalid texture format")]
    InvalidFormat,
    #[error("Texture too large: {width}x{height}, max: {max_dimension}")]
    TextureTooLarge { width: u32, height: u32, max_dimension: u32 },
}

/// Handle to a loaded texture
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureHandle {
    pub id: u32,
}

impl TextureHandle {
    /// Create a new texture handle
    pub fn new(id: u32) -> Self {
        Self { id }
    }
}

impl Default for TextureHandle {
    fn default() -> Self {
        Self { id: 0 }
    }
}

/// Texture loading configuration
#[derive(Debug, Clone)]
pub struct TextureLoadConfig {
    /// Whether to generate mipmaps
    pub generate_mipmaps: bool,
    /// Texture format (None to auto-detect)
    pub format: Option<TextureFormat>,
    /// Sampler configuration
    pub sampler_config: SamplerConfig,
}

impl Default for TextureLoadConfig {
    fn default() -> Self {
        Self {
            generate_mipmaps: false,
            format: None,
            sampler_config: SamplerConfig::default(),
        }
    }
}

/// Sampler configuration
#[derive(Debug, Clone)]
pub struct SamplerConfig {
    pub address_mode_u: wgpu::AddressMode,
    pub address_mode_v: wgpu::AddressMode,
    pub address_mode_w: wgpu::AddressMode,
    pub mag_filter: wgpu::FilterMode,
    pub min_filter: wgpu::FilterMode,
    pub mipmap_filter: wgpu::MipmapFilterMode,
    pub lod_min_clamp: f32,
    pub lod_max_clamp: f32,
    pub compare: Option<wgpu::CompareFunction>,
    pub anisotropy_clamp: u16,
}

impl Default for SamplerConfig {
    fn default() -> Self {
        Self {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            lod_min_clamp: 0.0,
            lod_max_clamp: f32::MAX,
            compare: None,
            anisotropy_clamp: 1,
        }
    }
}

/// Texture manager for loading and caching textures
pub struct TextureManager {
    device: Arc<Device>,
    queue: Arc<Queue>,
    textures: HashMap<TextureHandle, TextureResource>,
    next_handle: u32,
    max_texture_dimension: u32,
}

impl TextureManager {
    /// Create a new texture manager
    pub fn new(device: Arc<Device>, queue: Arc<Queue>) -> Self {
        let max_texture_dimension = device.limits().max_texture_dimension_2d;
        
        Self {
            device,
            queue,
            textures: HashMap::new(),
            next_handle: 1, // 0 is reserved for default texture
            max_texture_dimension,
        }
    }

    /// Load a texture from a file path
    ///
    /// Supports PNG, JPEG, BMP, and GIF formats. The image is automatically
    /// converted to RGBA format for GPU upload.
    ///
    /// # Example
    /// ```ignore
    /// let handle = texture_manager.load_texture("assets/player.png", TextureLoadConfig::default())?;
    /// ```
    pub fn load_texture<P: AsRef<Path>>(
        &mut self,
        path: P,
        config: TextureLoadConfig,
    ) -> Result<TextureHandle, TextureError> {
        let path = path.as_ref();

        log::info!("Loading texture from path: {:?}", path);

        // Load the image from file
        let img = image::open(path).map_err(|e| {
            TextureError::ImageLoadError(format!("Failed to load {:?}: {}", path, e))
        })?;

        // Get image dimensions
        let (width, height) = img.dimensions();

        // Validate dimensions
        if width > self.max_texture_dimension || height > self.max_texture_dimension {
            return Err(TextureError::TextureTooLarge {
                width,
                height,
                max_dimension: self.max_texture_dimension,
            });
        }

        // Convert to RGBA8
        let rgba = img.to_rgba8();
        let data = rgba.as_raw();

        // Create handle and texture
        let handle = TextureHandle::new(self.next_handle);
        self.next_handle += 1;

        let texture = self.create_texture_from_rgba(width, height, data, config)?;
        self.textures.insert(handle, texture);

        log::info!("Loaded texture {:?}: {}x{} (handle {})", path, width, height, handle.id);

        Ok(handle)
    }

    /// Load a texture from raw bytes (file contents)
    ///
    /// Useful for loading textures from embedded assets or network resources.
    pub fn load_texture_from_bytes(
        &mut self,
        bytes: &[u8],
        config: TextureLoadConfig,
    ) -> Result<TextureHandle, TextureError> {
        let img = image::load_from_memory(bytes).map_err(|e| {
            TextureError::ImageLoadError(format!("Failed to decode image: {}", e))
        })?;

        let (width, height) = img.dimensions();

        if width > self.max_texture_dimension || height > self.max_texture_dimension {
            return Err(TextureError::TextureTooLarge {
                width,
                height,
                max_dimension: self.max_texture_dimension,
            });
        }

        let rgba = img.to_rgba8();
        let data = rgba.as_raw();

        let handle = TextureHandle::new(self.next_handle);
        self.next_handle += 1;

        let texture = self.create_texture_from_rgba(width, height, data, config)?;
        self.textures.insert(handle, texture);

        log::info!("Loaded texture from bytes: {}x{} (handle {})", width, height, handle.id);

        Ok(handle)
    }

    /// Load a texture from raw RGBA data
    pub fn load_texture_from_rgba(
        &mut self,
        width: u32,
        height: u32,
        data: &[u8],
        config: TextureLoadConfig,
    ) -> Result<TextureHandle, TextureError> {
        if width == 0 || height == 0 {
            return Err(TextureError::InvalidFormat);
        }

        if width > self.max_texture_dimension || height > self.max_texture_dimension {
            return Err(TextureError::TextureTooLarge {
                width,
                height,
                max_dimension: self.max_texture_dimension,
            });
        }

        if data.len() != (width * height * 4) as usize {
            return Err(TextureError::InvalidFormat);
        }

        let handle = TextureHandle::new(self.next_handle);
        self.next_handle += 1;

        let texture = self.create_texture_from_rgba(width, height, data, config)?;
        self.textures.insert(handle, texture);

        Ok(handle)
    }

    /// Create a solid color texture
    pub fn create_solid_color(
        &mut self,
        width: u32,
        height: u32,
        color: [u8; 4],
    ) -> Result<TextureHandle, TextureError> {
        let data = vec![color; (width * height) as usize];
        self.load_texture_from_rgba(width, height, bytemuck::cast_slice(&data), TextureLoadConfig::default())
    }

    /// Create a checkerboard pattern texture
    pub fn create_checkerboard(
        &mut self,
        width: u32,
        height: u32,
        color1: [u8; 4],
        color2: [u8; 4],
        check_size: u32,
    ) -> Result<TextureHandle, TextureError> {
        let mut data = Vec::with_capacity((width * height * 4) as usize);
        
        for y in 0..height {
            for x in 0..width {
                let check_x = (x / check_size) % 2;
                let check_y = (y / check_size) % 2;
                let color = if (check_x + check_y) % 2 == 0 { color1 } else { color2 };
                data.extend_from_slice(&color);
            }
        }

        self.load_texture_from_rgba(width, height, &data, TextureLoadConfig::default())
    }

    /// Get a texture by handle
    pub fn get_texture(&self, handle: TextureHandle) -> Option<&TextureResource> {
        self.textures.get(&handle)
    }

    /// Remove a texture
    pub fn remove_texture(&mut self, handle: TextureHandle) -> Option<TextureResource> {
        self.textures.remove(&handle)
    }

    /// Check if a texture exists
    pub fn has_texture(&self, handle: TextureHandle) -> bool {
        self.textures.contains_key(&handle)
    }

    /// Get all texture handles
    pub fn texture_handles(&self) -> Vec<TextureHandle> {
        self.textures.keys().copied().collect()
    }

    /// Get texture count
    pub fn texture_count(&self) -> usize {
        self.textures.len()
    }

    /// Get all textures as a reference to the internal HashMap
    ///
    /// Useful for passing texture resources to the renderer for sprite rendering.
    pub fn textures(&self) -> &HashMap<TextureHandle, TextureResource> {
        &self.textures
    }

    /// Get a cloned HashMap of all textures
    ///
    /// Returns a new HashMap with cloned TextureResource values.
    pub fn textures_cloned(&self) -> HashMap<TextureHandle, TextureResource> {
        self.textures.clone()
    }

    /// Create texture from RGBA data using write_texture directly with simplified layout
    fn create_texture_from_rgba(
        &self,
        width: u32,
        height: u32,
        data: &[u8],
        config: TextureLoadConfig,
    ) -> Result<TextureResource, TextureError> {
        let format = config.format.unwrap_or(TextureFormat::Rgba8UnormSrgb);
        
        let texture = Arc::new(self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Dynamic Texture"),
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: if config.generate_mipmaps {
                ((width.max(height) as f32).log2().floor() as u32) + 1
            } else {
                1
            },
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        }));

        // Upload texture data using write_texture directly with simplified layout
        self.queue.write_texture(
            texture.as_image_copy(),
            data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(width * 4),
                rows_per_image: None,
            },
            texture.size(),
        );

        // Create view
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Create sampler
        let sampler = self.create_sampler(&config.sampler_config);

        Ok(TextureResource {
            texture: Arc::clone(&texture),
            view,
            sampler,
            width,
            height,
        })
    }

    /// Create a placeholder texture (reserved for future use)
    #[allow(dead_code)]
    fn create_placeholder_texture(
        &self,
        width: u32,
        height: u32,
        color: &[u8; 4],
    ) -> Result<TextureResource, TextureError> {
        let data = vec![*color; (width * height) as usize];
        self.create_texture_from_rgba(width, height, bytemuck::cast_slice(&data), TextureLoadConfig::default())
    }

    /// Create sampler from config
    fn create_sampler(&self, config: &SamplerConfig) -> Sampler {
        self.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Texture Sampler"),
            address_mode_u: config.address_mode_u,
            address_mode_v: config.address_mode_v,
            address_mode_w: config.address_mode_w,
            mag_filter: config.mag_filter,
            min_filter: config.min_filter,
            mipmap_filter: config.mipmap_filter,
            lod_min_clamp: config.lod_min_clamp,
            lod_max_clamp: config.lod_max_clamp,
            compare: config.compare,
            anisotropy_clamp: config.anisotropy_clamp,
            ..Default::default()
        })
    }
}

/// Builder for creating texture atlases
pub struct TextureAtlasBuilder {
    regions: Vec<AtlasRegion>,
    max_width: u32,
    max_height: u32,
    padding: u32,
}

#[derive(Debug, Clone)]
pub struct AtlasRegion {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub data: Option<Vec<u8>>, // RGBA data
}

impl TextureAtlasBuilder {
    /// Create a new texture atlas builder
    pub fn new(max_width: u32, max_height: u32) -> Self {
        Self {
            regions: Vec::new(),
            max_width,
            max_height,
            padding: 2,
        }
    }

    /// Set padding between atlas regions
    pub fn with_padding(mut self, padding: u32) -> Self {
        self.padding = padding;
        self
    }

    /// Add a region to the atlas
    pub fn add_region(mut self, name: String, width: u32, height: u32, data: Option<Vec<u8>>) -> Self {
        self.regions.push(AtlasRegion {
            name,
            width,
            height,
            data,
        });
        self
    }

    /// Build the texture atlas
    pub fn build(self, device: &Device, _queue: &Queue) -> Result<crate::sprite::TextureAtlas, TextureError> {
        // Simple packing algorithm - for production, use a proper bin packing algorithm
        let mut x = 0;
        let mut y = 0;
        let mut max_row_height = 0;
        
        let mut atlas_regions = Vec::new();

        for region in &self.regions {
            if x + region.width + self.padding > self.max_width {
                // Move to next row
                x = 0;
                y += max_row_height + self.padding;
                max_row_height = 0;
            }

            if y + region.height + self.padding > self.max_height {
                return Err(TextureError::TextureCreationError("Atlas too small for all regions".to_string()));
            }

            atlas_regions.push((region.name.clone(), x, y, region.width, region.height));
            
            x += region.width + self.padding;
            max_row_height = max_row_height.max(region.height);
        }

        let atlas_width = self.max_width;
        let atlas_height = y + max_row_height + self.padding;

        // Create the atlas texture
        let mut atlas = crate::sprite::TextureAtlas::new(device, atlas_width, atlas_height);

        // Add regions to atlas
        for (name, x, y, width, height) in atlas_regions {
            atlas.add_region(name, x, y, width, height, atlas_width, atlas_height);
        }

        Ok(atlas)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== TextureHandle Tests ====================

    #[test]
    fn test_texture_handle_new() {
        let handle = TextureHandle::new(42);
        assert_eq!(handle.id, 42);
    }

    #[test]
    fn test_texture_handle_default() {
        let handle = TextureHandle::default();
        assert_eq!(handle.id, 0);
    }

    #[test]
    fn test_texture_handle_equality() {
        let handle1 = TextureHandle::new(5);
        let handle2 = TextureHandle::new(5);
        let handle3 = TextureHandle::new(10);

        assert_eq!(handle1, handle2);
        assert_ne!(handle1, handle3);
    }

    #[test]
    fn test_texture_handle_hash() {
        use std::collections::HashMap;

        let mut map: HashMap<TextureHandle, &str> = HashMap::new();
        map.insert(TextureHandle::new(1), "texture1");
        map.insert(TextureHandle::new(2), "texture2");

        assert_eq!(map.get(&TextureHandle::new(1)), Some(&"texture1"));
        assert_eq!(map.get(&TextureHandle::new(2)), Some(&"texture2"));
        assert_eq!(map.get(&TextureHandle::new(3)), None);
    }

    #[test]
    fn test_texture_handle_copy() {
        let handle1 = TextureHandle::new(7);
        let handle2 = handle1; // Copy
        assert_eq!(handle1.id, handle2.id);
    }

    // ==================== TextureLoadConfig Tests ====================

    #[test]
    fn test_texture_load_config_default() {
        let config = TextureLoadConfig::default();
        assert!(!config.generate_mipmaps);
        assert!(config.format.is_none());
    }

    #[test]
    fn test_texture_load_config_with_mipmaps() {
        let config = TextureLoadConfig {
            generate_mipmaps: true,
            ..Default::default()
        };
        assert!(config.generate_mipmaps);
    }

    #[test]
    fn test_texture_load_config_with_format() {
        let config = TextureLoadConfig {
            format: Some(wgpu::TextureFormat::Rgba8Unorm),
            ..Default::default()
        };
        assert_eq!(config.format, Some(wgpu::TextureFormat::Rgba8Unorm));
    }

    // ==================== SamplerConfig Tests ====================

    #[test]
    fn test_sampler_config_default() {
        let config = SamplerConfig::default();
        assert_eq!(config.address_mode_u, wgpu::AddressMode::ClampToEdge);
        assert_eq!(config.address_mode_v, wgpu::AddressMode::ClampToEdge);
        assert_eq!(config.address_mode_w, wgpu::AddressMode::ClampToEdge);
        assert_eq!(config.mag_filter, wgpu::FilterMode::Linear);
        assert_eq!(config.min_filter, wgpu::FilterMode::Linear);
        assert_eq!(config.mipmap_filter, wgpu::MipmapFilterMode::Linear);
        assert_eq!(config.lod_min_clamp, 0.0);
        assert_eq!(config.lod_max_clamp, f32::MAX);
        assert!(config.compare.is_none());
        assert_eq!(config.anisotropy_clamp, 1);
    }

    #[test]
    fn test_sampler_config_custom() {
        let config = SamplerConfig {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::MirrorRepeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            anisotropy_clamp: 4,
            ..Default::default()
        };

        assert_eq!(config.address_mode_u, wgpu::AddressMode::Repeat);
        assert_eq!(config.address_mode_v, wgpu::AddressMode::MirrorRepeat);
        assert_eq!(config.mag_filter, wgpu::FilterMode::Nearest);
        assert_eq!(config.min_filter, wgpu::FilterMode::Nearest);
        assert_eq!(config.anisotropy_clamp, 4);
    }

    // ==================== TextureError Tests ====================

    #[test]
    fn test_texture_error_display() {
        let error = TextureError::ImageLoadError("file not found".to_string());
        assert!(error.to_string().contains("file not found"));

        let error = TextureError::TextureNotFound("missing.png".to_string());
        assert!(error.to_string().contains("missing.png"));

        let error = TextureError::InvalidFormat;
        assert!(error.to_string().contains("Invalid"));

        let error = TextureError::TextureTooLarge {
            width: 10000,
            height: 10000,
            max_dimension: 8192,
        };
        let msg = error.to_string();
        assert!(msg.contains("10000"));
        assert!(msg.contains("8192"));
    }

    // ==================== AtlasRegion Tests ====================

    #[test]
    fn test_atlas_region_creation() {
        let region = AtlasRegion {
            name: "sprite1".to_string(),
            width: 64,
            height: 64,
            data: None,
        };

        assert_eq!(region.name, "sprite1");
        assert_eq!(region.width, 64);
        assert_eq!(region.height, 64);
        assert!(region.data.is_none());
    }

    #[test]
    fn test_atlas_region_with_data() {
        let data = vec![255u8; 64 * 64 * 4]; // RGBA data for 64x64 texture
        let region = AtlasRegion {
            name: "sprite2".to_string(),
            width: 64,
            height: 64,
            data: Some(data.clone()),
        };

        assert!(region.data.is_some());
        assert_eq!(region.data.unwrap().len(), 64 * 64 * 4);
    }

    // ==================== TextureAtlasBuilder Tests ====================

    #[test]
    fn test_texture_atlas_builder_new() {
        let builder = TextureAtlasBuilder::new(1024, 1024);
        assert_eq!(builder.max_width, 1024);
        assert_eq!(builder.max_height, 1024);
        assert_eq!(builder.padding, 2); // default padding
        assert!(builder.regions.is_empty());
    }

    #[test]
    fn test_texture_atlas_builder_with_padding() {
        let builder = TextureAtlasBuilder::new(1024, 1024).with_padding(4);
        assert_eq!(builder.padding, 4);
    }

    #[test]
    fn test_texture_atlas_builder_add_region() {
        let builder = TextureAtlasBuilder::new(1024, 1024)
            .add_region("sprite1".to_string(), 64, 64, None)
            .add_region("sprite2".to_string(), 128, 128, None);

        assert_eq!(builder.regions.len(), 2);
        assert_eq!(builder.regions[0].name, "sprite1");
        assert_eq!(builder.regions[0].width, 64);
        assert_eq!(builder.regions[1].name, "sprite2");
        assert_eq!(builder.regions[1].width, 128);
    }

    #[test]
    fn test_texture_atlas_builder_chaining() {
        let builder = TextureAtlasBuilder::new(512, 512)
            .with_padding(1)
            .add_region("a".to_string(), 32, 32, None)
            .add_region("b".to_string(), 32, 32, None)
            .add_region("c".to_string(), 32, 32, None);

        assert_eq!(builder.padding, 1);
        assert_eq!(builder.regions.len(), 3);
    }

    // Note: TextureManager and TextureAtlasBuilder.build() require GPU device,
    // so those are tested in integration tests or with mocked devices
}