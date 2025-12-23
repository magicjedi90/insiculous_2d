//! Simple test to verify sprite rendering system works

use renderer::prelude::*;

#[test]
fn test_sprite_creation() {
    let sprite = Sprite::new(TextureHandle::new(1))
        .with_position(glam::Vec2::new(100.0, 200.0))
        .with_rotation(std::f32::consts::PI / 4.0)
        .with_scale(glam::Vec2::new(2.0, 2.0))
        .with_color(glam::Vec4::new(1.0, 0.5, 0.0, 1.0))
        .with_depth(1.0);

    assert_eq!(sprite.position, glam::Vec2::new(100.0, 200.0));
    assert_eq!(sprite.rotation, std::f32::consts::PI / 4.0);
    assert_eq!(sprite.scale, glam::Vec2::new(2.0, 2.0));
    assert_eq!(sprite.color, glam::Vec4::new(1.0, 0.5, 0.0, 1.0));
    assert_eq!(sprite.depth, 1.0);
    assert_eq!(sprite.texture_handle.id, 1);
}

#[test]
fn test_sprite_instance_creation() {
    let instance = SpriteInstance::new(
        glam::Vec2::new(50.0, 75.0),
        0.5,
        glam::Vec2::new(1.5, 1.5),
        [0.1, 0.2, 0.3, 0.4],
        glam::Vec4::new(0.2, 0.3, 0.4, 0.5),
        2.0,
    );

    assert_eq!(instance.position, [50.0, 75.0]);
    assert_eq!(instance.rotation, 0.5);
    assert_eq!(instance.scale, [1.5, 1.5]);
    assert_eq!(instance.tex_region, [0.1, 0.2, 0.3, 0.4]);
    assert_eq!(instance.color, [0.2, 0.3, 0.4, 0.5]);
    assert_eq!(instance.depth, 2.0);
}

#[test]
fn test_camera_2d_creation() {
    let camera = Camera2D::new(glam::Vec2::new(0.0, 0.0), glam::Vec2::new(800.0, 600.0));
    
    assert_eq!(camera.position, glam::Vec2::new(0.0, 0.0));
    assert_eq!(camera.viewport_size, glam::Vec2::new(800.0, 600.0));
    assert_eq!(camera.zoom, 1.0);
    assert_eq!(camera.rotation, 0.0);
}

#[test]
fn test_camera_matrices() {
    let camera = Camera2D::new(glam::Vec2::new(100.0, 200.0), glam::Vec2::new(800.0, 600.0));
    
    let view_matrix = camera.view_matrix();
    let proj_matrix = camera.projection_matrix();
    let vp_matrix = camera.view_projection_matrix();
    
    // Verify matrices are created without panicking
    assert!(view_matrix.is_finite());
    assert!(proj_matrix.is_finite());
    assert!(vp_matrix.is_finite());
}

#[test]
fn test_sprite_batch_creation() {
    let mut batch = SpriteBatch::new(TextureHandle::new(1));
    
    assert_eq!(batch.texture_handle.id, 1);
    assert!(batch.is_empty());
    assert_eq!(batch.len(), 0);
    
    let instance = SpriteInstance::new(
        glam::Vec2::new(10.0, 20.0),
        0.0,
        glam::Vec2::new(1.0, 1.0),
        [0.0, 0.0, 1.0, 1.0],
        glam::Vec4::ONE,
        0.0,
    );
    
    batch.add_instance(instance);
    assert!(!batch.is_empty());
    assert_eq!(batch.len(), 1);
}

#[test]
fn test_sprite_batcher() {
    let mut batcher = SpriteBatcher::new(1000);
    
    let sprite1 = Sprite::new(TextureHandle::new(1))
        .with_position(glam::Vec2::new(10.0, 20.0));
    
    let sprite2 = Sprite::new(TextureHandle::new(2))
        .with_position(glam::Vec2::new(30.0, 40.0));
    
    batcher.add_sprite(&sprite1);
    batcher.add_sprite(&sprite2);
    
    assert_eq!(batcher.sprite_count(), 2);
    assert_eq!(batcher.batches().len(), 2); // Should have 2 batches for 2 different textures
}

#[test]
fn test_texture_handle() {
    let handle1 = TextureHandle::new(42);
    let handle2 = TextureHandle::new(42);
    let handle3 = TextureHandle::new(43);
    
    assert_eq!(handle1, handle2);
    assert_ne!(handle1, handle3);
    assert_eq!(handle1.id, 42);
    assert_eq!(handle3.id, 43);
}

#[test]
fn test_camera_uniform() {
    let camera = Camera2D::new(glam::Vec2::new(50.0, 75.0), glam::Vec2::new(1024.0, 768.0));
    let uniform = CameraUniform::from_camera(&camera);
    
    // Verify uniform is created without panicking
    assert!(uniform.view_projection.iter().all(|row| row.iter().all(|&val| val.is_finite())));
    assert_eq!(uniform.position, [50.0, 75.0]);
}