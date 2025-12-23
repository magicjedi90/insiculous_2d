//! Tests for sprite batching system

use glam::{Vec2, Vec4};
use renderer::sprite::*;

#[test]
fn test_sprite_creation() {
    let sprite = Sprite::new(TextureHandle::new(1))
        .with_position(Vec2::new(10.0, 20.0))
        .with_rotation(std::f32::consts::PI)
        .with_scale(Vec2::new(2.0, 3.0))
        .with_tex_region(0.1, 0.2, 0.3, 0.4)
        .with_color(Vec4::new(1.0, 0.5, 0.0, 0.8))
        .with_depth(5.0);

    assert_eq!(sprite.position, Vec2::new(10.0, 20.0));
    assert_eq!(sprite.rotation, std::f32::consts::PI);
    assert_eq!(sprite.scale, Vec2::new(2.0, 3.0));
    assert_eq!(sprite.tex_region, [0.1, 0.2, 0.3, 0.4]);
    assert_eq!(sprite.color, Vec4::new(1.0, 0.5, 0.0, 0.8));
    assert_eq!(sprite.depth, 5.0);
    assert_eq!(sprite.texture_handle, TextureHandle::new(1));
}

#[test]
fn test_sprite_default() {
    let sprite = Sprite::default();
    
    assert_eq!(sprite.position, Vec2::ZERO);
    assert_eq!(sprite.rotation, 0.0);
    assert_eq!(sprite.scale, Vec2::ONE);
    assert_eq!(sprite.tex_region, [0.0, 0.0, 1.0, 1.0]);
    assert_eq!(sprite.color, Vec4::ONE);
    assert_eq!(sprite.depth, 0.0);
    assert_eq!(sprite.texture_handle, TextureHandle::default());
}

#[test]
fn test_sprite_to_instance() {
    let sprite = Sprite {
        position: Vec2::new(10.0, 20.0),
        rotation: std::f32::consts::PI,
        scale: Vec2::new(2.0, 3.0),
        tex_region: [0.1, 0.2, 0.3, 0.4],
        color: Vec4::new(1.0, 0.5, 0.0, 0.8),
        depth: 5.0,
        texture_handle: TextureHandle::new(1),
    };

    let instance = sprite.to_instance();

    assert_eq!(instance.position, [10.0, 20.0]);
    assert_eq!(instance.rotation, std::f32::consts::PI);
    assert_eq!(instance.scale, [2.0, 3.0]);
    assert_eq!(instance.tex_region, [0.1, 0.2, 0.3, 0.4]);
    assert_eq!(instance.color, [1.0, 0.5, 0.0, 0.8]);
    assert_eq!(instance.depth, 5.0);
}

#[test]
fn test_sprite_batch_creation() {
    let mut batch = SpriteBatch::new(TextureHandle::new(1));
    
    assert_eq!(batch.texture_handle, TextureHandle::new(1));
    assert!(batch.is_empty());
    assert_eq!(batch.len(), 0);
    assert!(!batch.sorted);
}

#[test]
fn test_sprite_batch_add_instance() {
    let mut batch = SpriteBatch::new(TextureHandle::new(1));
    
    let instance = renderer::sprite_data::SpriteInstance::new(
        Vec2::new(10.0, 20.0),
        0.0,
        Vec2::ONE,
        [0.0, 0.0, 1.0, 1.0],
        Vec4::ONE,
        0.0,
    );
    
    batch.add_instance(instance);
    
    assert_eq!(batch.len(), 1);
    assert!(!batch.is_empty());
    assert!(!batch.sorted);
}

#[test]
fn test_sprite_batch_sort_by_depth() {
    let mut batch = SpriteBatch::new(TextureHandle::new(1));
    
    // Add instances with different depths
    batch.add_instance(renderer::sprite_data::SpriteInstance::new(
        Vec2::ZERO,
        0.0,
        Vec2::ONE,
        [0.0, 0.0, 1.0, 1.0],
        Vec4::ONE,
        3.0,
    ));
    
    batch.add_instance(renderer::sprite_data::SpriteInstance::new(
        Vec2::ZERO,
        0.0,
        Vec2::ONE,
        [0.0, 0.0, 1.0, 1.0],
        Vec4::ONE,
        1.0,
    ));
    
    batch.add_instance(renderer::sprite_data::SpriteInstance::new(
        Vec2::ZERO,
        0.0,
        Vec2::ONE,
        [0.0, 0.0, 1.0, 1.0],
        Vec4::ONE,
        2.0,
    ));
    
    batch.sort_by_depth();
    
    assert!(batch.sorted);
    assert_eq!(batch.instances[0].depth, 1.0);
    assert_eq!(batch.instances[1].depth, 2.0);
    assert_eq!(batch.instances[2].depth, 3.0);
}

#[test]
fn test_sprite_batcher_creation() {
    let batcher = SpriteBatcher::new(1000);
    
    assert_eq!(batcher.sprite_count(), 0);
    assert_eq!(batcher.max_sprites_per_batch, 1000);
}

#[test]
fn test_sprite_batcher_add_sprite() {
    let mut batcher = SpriteBatcher::new(1000);
    
    let sprite1 = Sprite::new(TextureHandle::new(1))
        .with_position(Vec2::new(10.0, 20.0));
    
    let sprite2 = Sprite::new(TextureHandle::new(1))
        .with_position(Vec2::new(30.0, 40.0));
    
    let sprite3 = Sprite::new(TextureHandle::new(2))
        .with_position(Vec2::new(50.0, 60.0));
    
    batcher.add_sprite(&sprite1);
    batcher.add_sprite(&sprite2);
    batcher.add_sprite(&sprite3);
    
    assert_eq!(batcher.sprite_count(), 3);
    
    // Should have 2 batches (one for texture 1, one for texture 2)
    assert_eq!(batcher.batches().len(), 2);
    
    // Batch for texture 1 should have 2 sprites
    let batch1 = &batcher.batches()[&TextureHandle::new(1)];
    assert_eq!(batch1.len(), 2);
    
    // Batch for texture 2 should have 1 sprite
    let batch2 = &batcher.batches()[&TextureHandle::new(2)];
    assert_eq!(batch2.len(), 1);
}

#[test]
fn test_sprite_batcher_clear() {
    let mut batcher = SpriteBatcher::new(1000);
    
    let sprite = Sprite::new(TextureHandle::new(1));
    batcher.add_sprite(&sprite);
    batcher.add_sprite(&sprite);
    
    assert_eq!(batcher.sprite_count(), 2);
    
    batcher.clear();
    
    assert_eq!(batcher.sprite_count(), 0);
    assert_eq!(batcher.batches().len(), 2); // Batches still exist but are empty
    
    // Check that batches are cleared
    for batch in batcher.batches().values() {
        assert!(batch.is_empty());
    }
}

#[test]
fn test_sprite_batcher_sort_all_batches() {
    let mut batcher = SpriteBatcher::new(1000);
    
    // Add sprites with different depths
    let sprite1 = Sprite {
        depth: 3.0,
        texture_handle: TextureHandle::new(1),
        ..Default::default()
    };
    
    let sprite2 = Sprite {
        depth: 1.0,
        texture_handle: TextureHandle::new(1),
        ..Default::default()
    };
    
    let sprite3 = Sprite {
        depth: 2.0,
        texture_handle: TextureHandle::new(1),
        ..Default::default()
    };
    
    batcher.add_sprite(&sprite1);
    batcher.add_sprite(&sprite2);
    batcher.add_sprite(&sprite3);
    
    batcher.sort_all_batches();
    
    let batch = &batcher.batches()[&TextureHandle::new(1)];
    assert!(batch.sorted);
    assert_eq!(batch.instances[0].depth, 1.0);
    assert_eq!(batch.instances[1].depth, 2.0);
    assert_eq!(batch.instances[2].depth, 3.0);
}

#[test]
fn test_texture_atlas_creation() {
    use std::sync::Arc;
    use wgpu::{Device, Instance};
    
    // This test requires a WGPU device, so we'll skip it if we can't create one
    let instance = Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::default(),
        force_fallback_adapter: true,
        compatible_surface: None,
    }));
    
    if let Some(adapter) = adapter {
        let (device, _queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor::default(),
            None,
        )).unwrap();
        
        let atlas = TextureAtlas::new(&device, 512, 512);
        
        assert_eq!(atlas.regions.len(), 0);
        assert_eq!(atlas.width, 512);
        assert_eq!(atlas.height, 512);
    }
}

#[test]
fn test_texture_atlas_regions() {
    use std::sync::Arc;
    use wgpu::{Device, Instance};
    
    let instance = Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::default(),
        force_fallback_adapter: true,
        compatible_surface: None,
    }));
    
    if let Some(adapter) = adapter {
        let (device, _queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor::default(),
            None,
        )).unwrap();
        
        let mut atlas = TextureAtlas::new(&device, 512, 512);
        
        // Add some regions
        atlas.add_region("player".to_string(), 0, 0, 64, 64, 512, 512);
        atlas.add_region("enemy".to_string(), 64, 0, 32, 32, 512, 512);
        atlas.add_region("item".to_string(), 0, 64, 16, 16, 512, 512);
        
        // Check regions
        let player_region = atlas.get_region("player").unwrap();
        assert_eq!(player_region, [0.0, 0.0, 64.0 / 512.0, 64.0 / 512.0]);
        
        let enemy_region = atlas.get_region("enemy").unwrap();
        assert_eq!(enemy_region, [64.0 / 512.0, 0.0, 32.0 / 512.0, 32.0 / 512.0]);
        
        let item_region = atlas.get_region("item").unwrap();
        assert_eq!(item_region, [0.0, 64.0 / 512.0, 16.0 / 512.0, 16.0 / 512.0]);
        
        // Check non-existent region
        assert!(atlas.get_region("nonexistent").is_none());
    }
}