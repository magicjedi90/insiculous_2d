//! Tests for texture loading and management system

use std::sync::Arc;
use renderer::texture::*;

async fn create_test_device() -> Option<(Arc<wgpu::Device>, Arc<wgpu::Queue>)> {
    let instance = wgpu::Instance::default();
    let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::default(),
        force_fallback_adapter: true,
        compatible_surface: None,
    }).await;
    
    if let Some(adapter) = adapter {
        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor::default(),
            None,
        ).await.ok()?;
        
        Some((Arc::new(device), Arc::new(queue)))
    } else {
        None
    }
}

#[test]
fn test_texture_handle() {
    let handle1 = TextureHandle::new(42);
    let handle2 = TextureHandle::new(42);
    let handle3 = TextureHandle::new(43);
    
    assert_eq!(handle1.id, 42);
    assert_eq!(handle1, handle2);
    assert_ne!(handle1, handle3);
}

#[test]
fn test_texture_load_config_default() {
    let config = TextureLoadConfig::default();
    
    assert!(!config.generate_mipmaps);
    assert!(config.format.is_none());
    assert_eq!(config.sampler_config.address_mode_u, wgpu::AddressMode::ClampToEdge);
    assert_eq!(config.sampler_config.mag_filter, wgpu::FilterMode::Linear);
}

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

#[tokio::test]
async fn test_texture_manager_creation() {
    if let Some((device, queue)) = create_test_device().await {
        let manager = TextureManager::new(device.clone(), queue.clone());
        
        assert_eq!(manager.texture_count(), 0);
        assert!(manager.texture_handles().is_empty());
    }
}

#[tokio::test]
async fn test_create_solid_color_texture() {
    if let Some((device, queue)) = create_test_device().await {
        let mut manager = TextureManager::new(device.clone(), queue.clone());
        
        let handle = manager.create_solid_color(64, 64, [255, 0, 0, 255]).unwrap();
        
        assert_eq!(manager.texture_count(), 1);
        assert!(manager.has_texture(handle));
        
        let texture = manager.get_texture(handle).unwrap();
        assert_eq!(texture.width, 64);
        assert_eq!(texture.height, 64);
    }
}

#[tokio::test]
async fn test_create_checkerboard_texture() {
    if let Some((device, queue)) = create_test_device().await {
        let mut manager = TextureManager::new(device.clone(), queue.clone());
        
        let handle = manager.create_checkerboard(
            64, 64,
            [255, 255, 255, 255], // White
            [0, 0, 0, 255],       // Black
            8
        ).unwrap();
        
        assert_eq!(manager.texture_count(), 1);
        assert!(manager.has_texture(handle));
        
        let texture = manager.get_texture(handle).unwrap();
        assert_eq!(texture.width, 64);
        assert_eq!(texture.height, 64);
    }
}

#[tokio::test]
async fn test_load_texture_from_rgba() {
    if let Some((device, queue)) = create_test_device().await {
        let mut manager = TextureManager::new(device.clone(), queue.clone());
        
        // Create a simple 4x4 texture with red color
        let mut data = Vec::new();
        for _ in 0..16 {
            data.extend_from_slice(&[255, 0, 0, 255]); // Red
        }
        
        let handle = manager.load_texture_from_rgba(
            4, 4,
            &data,
            TextureLoadConfig::default()
        ).unwrap();
        
        assert_eq!(manager.texture_count(), 1);
        assert!(manager.has_texture(handle));
        
        let texture = manager.get_texture(handle).unwrap();
        assert_eq!(texture.width, 4);
        assert_eq!(texture.height, 4);
    }
}

#[tokio::test]
async fn test_texture_error_invalid_size() {
    if let Some((device, queue)) = create_test_device().await {
        let mut manager = TextureManager::new(device.clone(), queue.clone());
        
        let result = manager.load_texture_from_rgba(
            0, 4,
            &[0; 16],
            TextureLoadConfig::default()
        );
        
        assert!(matches!(result, Err(TextureError::InvalidFormat)));
    }
}

#[tokio::test]
async fn test_texture_error_wrong_data_size() {
    if let Some((device, queue)) = create_test_device().await {
        let mut manager = TextureManager::new(device.clone(), queue.clone());
        
        let result = manager.load_texture_from_rgba(
            4, 4,
            &[0; 10], // Wrong size - should be 4*4*4 = 64 bytes
            TextureLoadConfig::default()
        );
        
        assert!(matches!(result, Err(TextureError::InvalidFormat)));
    }
}

#[tokio::test]
async fn test_remove_texture() {
    if let Some((device, queue)) = create_test_device().await {
        let mut manager = TextureManager::new(device.clone(), queue.clone());
        
        let handle = manager.create_solid_color(64, 64, [255, 0, 0, 255]).unwrap();
        assert_eq!(manager.texture_count(), 1);
        
        let removed = manager.remove_texture(handle).unwrap();
        assert_eq!(manager.texture_count(), 0);
        assert_eq!(removed.width, 64);
        assert_eq!(removed.height, 64);
        
        // Should not be able to get texture after removal
        assert!(!manager.has_texture(handle));
        assert!(manager.get_texture(handle).is_none());
    }
}

#[tokio::test]
async fn test_texture_handles() {
    if let Some((device, queue)) = create_test_device().await {
        let mut manager = TextureManager::new(device.clone(), queue.clone());
        
        let handle1 = manager.create_solid_color(32, 32, [255, 0, 0, 255]).unwrap();
        let handle2 = manager.create_solid_color(64, 64, [0, 255, 0, 255]).unwrap();
        let handle3 = manager.create_solid_color(128, 128, [0, 0, 255, 255]).unwrap();
        
        let handles = manager.texture_handles();
        assert_eq!(handles.len(), 3);
        assert!(handles.contains(&handle1));
        assert!(handles.contains(&handle2));
        assert!(handles.contains(&handle3));
    }
}

#[tokio::test]
async fn test_texture_atlas_builder() {
    if let Some((device, queue)) = create_test_device().await {
        let builder = TextureAtlasBuilder::new(512, 512)
            .with_padding(4);
        
        // Add some regions
        let builder = builder
            .add_region("player".to_string(), 64, 64, None)
            .add_region("enemy".to_string(), 32, 32, None)
            .add_region("item".to_string(), 16, 16, None);
        
        let atlas = builder.build(&device, &queue).unwrap();
        
        assert!(atlas.get_region("player").is_some());
        assert!(atlas.get_region("enemy").is_some());
        assert!(atlas.get_region("item").is_some());
        assert!(atlas.get_region("nonexistent").is_none());
    }
}

#[tokio::test]
async fn test_texture_atlas_builder_too_small() {
    if let Some((device, queue)) = create_test_device().await {
        let builder = TextureAtlasBuilder::new(10, 10)
            .add_region("large_texture".to_string(), 100, 100, None);
        
        let result = builder.build(&device, &queue);
        assert!(result.is_err());
    }
}