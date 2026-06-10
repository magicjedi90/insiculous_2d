//! Texture atlas types: runtime atlas regions and the builder that packs them.

use std::collections::HashMap;
use std::sync::Arc;
use wgpu::{Device, Queue, Sampler, TextureView};

use crate::texture::{SamplerConfig, TextureError};

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
        let sampler = SamplerConfig::default().create_sampler(device, Some("Texture Atlas Sampler"));

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

/// A named region staged for packing into a [`TextureAtlas`]
#[derive(Debug, Clone)]
pub struct AtlasRegion {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub data: Option<Vec<u8>>, // RGBA data
}

/// Builder for creating texture atlases
pub struct TextureAtlasBuilder {
    regions: Vec<AtlasRegion>,
    max_width: u32,
    max_height: u32,
    padding: u32,
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
    pub fn build(self, device: &Device, _queue: &Queue) -> Result<TextureAtlas, TextureError> {
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
        let mut atlas = TextureAtlas::new(device, atlas_width, atlas_height);

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

    // Note: TextureAtlas and TextureAtlasBuilder.build() require a GPU device,
    // so those paths are exercised by ignored GPU tests / examples.
}
