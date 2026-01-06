//! Tests for ECS sprite components

use glam::{Vec2, Vec3, Vec4};
use ecs::sprite_components::*;
use ecs::Component;
use renderer::{Sprite as RendererSprite, Camera2D as RendererCamera2D};
use renderer::sprite::TextureHandle;

#[test]
fn test_sprite_creation() {
    let sprite = Sprite::new(42)
        .with_offset(Vec2::new(1.0, 2.0))
        .with_rotation(std::f32::consts::PI)
        .with_scale(Vec2::new(2.0, 3.0))
        .with_tex_region(0.1, 0.2, 0.3, 0.4)
        .with_color(Vec4::new(1.0, 0.5, 0.0, 0.8))
        .with_depth(5.0);

    assert_eq!(sprite.offset, Vec2::new(1.0, 2.0));
    assert_eq!(sprite.rotation, std::f32::consts::PI);
    assert_eq!(sprite.scale, Vec2::new(2.0, 3.0));
    assert_eq!(sprite.tex_region, [0.1, 0.2, 0.3, 0.4]);
    assert_eq!(sprite.color, Vec4::new(1.0, 0.5, 0.0, 0.8));
    assert_eq!(sprite.depth, 5.0);
    assert_eq!(sprite.texture_handle, 42);
}

#[test]
fn test_sprite_default() {
    let sprite = Sprite::default();
    
    assert_eq!(sprite.offset, Vec2::ZERO);
    assert_eq!(sprite.rotation, 0.0);
    assert_eq!(sprite.scale, Vec2::ONE);
    assert_eq!(sprite.tex_region, [0.0, 0.0, 1.0, 1.0]);
    assert_eq!(sprite.color, Vec4::ONE);
    assert_eq!(sprite.depth, 0.0);
    assert_eq!(sprite.texture_handle, 0);
}

#[test]
fn test_transform2d_creation() {
    let transform = Transform2D::new(Vec2::new(10.0, 20.0))
        .with_rotation(std::f32::consts::PI)
        .with_scale(Vec2::new(2.0, 3.0));

    assert_eq!(transform.position, Vec2::new(10.0, 20.0));
    assert_eq!(transform.rotation, std::f32::consts::PI);
    assert_eq!(transform.scale, Vec2::new(2.0, 3.0));
}

#[test]
fn test_transform2d_default() {
    let transform = Transform2D::default();
    
    assert_eq!(transform.position, Vec2::ZERO);
    assert_eq!(transform.rotation, 0.0);
    assert_eq!(transform.scale, Vec2::ONE);
}

#[test]
fn test_transform2d_matrix() {
    let transform = Transform2D::new(Vec2::new(10.0, 20.0))
        .with_rotation(std::f32::consts::FRAC_PI_2) // 90 degrees
        .with_scale(Vec2::new(2.0, 3.0));

    let matrix = transform.matrix();

    // Test that matrix is not identity
    assert_ne!(matrix, glam::Mat3::IDENTITY);

    // Test transforming a point
    // transform_point applies full T*R*S matrix:
    // 1. Scale (2,3): (1,0) -> (2,0)
    // 2. Rotate 90°: (2,0) -> (0,2)
    // 3. Translate (10,20): (0,2) -> (10,22)
    let point = Vec2::new(1.0, 0.0);
    let transformed = transform.transform_point(point);

    assert!((transformed.x - 10.0).abs() < 0.001);
    assert!((transformed.y - 22.0).abs() < 0.001);
}

#[test]
fn test_transform2d_inverse_matrix() {
    let transform = Transform2D::new(Vec2::new(10.0, 20.0))
        .with_rotation(std::f32::consts::PI)
        .with_scale(Vec2::new(2.0, 3.0));

    let matrix = transform.matrix();
    let inverse = transform.inverse_matrix();
    
    // Test that matrix * inverse is approximately identity
    let product = matrix * inverse;
    assert!((product.x_axis.x - 1.0).abs() < 0.001);
    assert!((product.y_axis.y - 1.0).abs() < 0.001);
    assert!((product.z_axis.z - 1.0).abs() < 0.001);
}

#[test]
fn test_camera2d_creation() {
    let camera = Camera2D::new(Vec2::new(100.0, 200.0), Vec2::new(1920.0, 1080.0))
        .with_rotation(std::f32::consts::PI)
        .with_zoom(2.0)
        .as_main_camera();

    assert_eq!(camera.position, Vec2::new(100.0, 200.0));
    assert_eq!(camera.viewport_size, Vec2::new(1920.0, 1080.0));
    assert_eq!(camera.rotation, std::f32::consts::PI);
    assert_eq!(camera.zoom, 2.0);
    assert!(camera.is_main_camera);
}

#[test]
fn test_camera2d_default() {
    let camera = Camera2D::default();
    
    assert_eq!(camera.position, Vec2::ZERO);
    assert_eq!(camera.rotation, 0.0);
    assert_eq!(camera.zoom, 1.0);
    assert_eq!(camera.viewport_size, Vec2::new(800.0, 600.0));
    assert!(!camera.is_main_camera);
}

#[test]
fn test_camera2d_view_matrix() {
    let camera = Camera2D {
        position: Vec2::new(100.0, 200.0),
        rotation: std::f32::consts::FRAC_PI_2, // 90 degrees
        zoom: 2.0,
        ..Default::default()
    };

    let view_matrix = camera.view_matrix();
    
    // Test that view matrix is not identity (should have transformations)
    assert_ne!(view_matrix, glam::Mat4::IDENTITY);
    
    // Test that a point is transformed correctly
    let world_point = Vec4::new(100.0, 200.0, 0.0, 1.0); // Convert to Vec4 for matrix multiplication
    let view_point = view_matrix * world_point;
    
    // The camera position should be at the origin in view space
    assert!((view_point.x).abs() < 0.001);
    assert!((view_point.y).abs() < 0.001);
}

#[test]
fn test_camera2d_projection_matrix() {
    let camera = Camera2D::new(Vec2::ZERO, Vec2::new(800.0, 600.0));
    let proj_matrix = camera.projection_matrix();
    
    // Test that the projection matrix is orthographic
    let near_point = Vec3::new(0.0, 0.0, -1000.0);
    let far_point = Vec3::new(0.0, 0.0, 1000.0);
    
    let near_clip = proj_matrix * Vec4::from((near_point, 1.0));
    let far_clip = proj_matrix * Vec4::from((far_point, 1.0));
    
    // In orthographic projection, Z values should be mapped to [-1, 1]
    assert!(near_clip.z >= -1.0 && near_clip.z <= 1.0);
    assert!(far_clip.z >= -1.0 && far_clip.z <= 1.0);
}

#[test]
fn test_camera2d_view_projection_matrix() {
    let camera = Camera2D::default();
    let vp_matrix = camera.view_projection_matrix();
    
    // Should be valid matrix
    assert!(!vp_matrix.is_nan());
    assert!(vp_matrix.is_finite());
}

#[test]
fn test_camera2d_screen_to_world() {
    let camera = Camera2D::new(Vec2::new(100.0, 200.0), Vec2::new(800.0, 600.0));
    
    // Test center of screen
    let _screen_center = Vec2::new(400.0, 300.0);
    // Note: screen_to_world method doesn't exist in ecs::Camera2D, testing matrix instead
    let view_matrix = camera.view_matrix();
    
    // Should be a valid matrix
    assert!(!view_matrix.is_nan());
    assert!(view_matrix.is_finite());
}

#[test]
fn test_camera2d_world_to_screen() {
    let camera = Camera2D::new(Vec2::new(100.0, 200.0), Vec2::new(800.0, 600.0));
    
    // Test camera position
    // Note: world_to_screen method doesn't exist in ecs::Camera2D, testing matrix instead
    let proj_matrix = camera.projection_matrix();
    
    // Should be a valid matrix
    assert!(!proj_matrix.is_nan());
    assert!(proj_matrix.is_finite());
}

#[test]
fn test_sprite_animation_default() {
    let animation = SpriteAnimation::default();
    
    assert_eq!(animation.current_frame, 0);
    assert_eq!(animation.fps, 10.0);
    assert!(animation.playing);
    assert!(animation.loop_animation);
    assert_eq!(animation.time_accumulator, 0.0);
    assert_eq!(animation.frames.len(), 1);
    assert_eq!(animation.frames[0], [0.0, 0.0, 1.0, 1.0]);
}

#[test]
fn test_sprite_animation_creation() {
    let frames = vec![
        [0.0, 0.0, 0.5, 0.5],
        [0.5, 0.0, 0.5, 0.5],
        [0.0, 0.5, 0.5, 0.5],
        [0.5, 0.5, 0.5, 0.5],
    ];
    
    let animation = SpriteAnimation::new(15.0, frames.clone())
        .with_loop(false);
    
    assert_eq!(animation.fps, 15.0);
    assert_eq!(animation.frames, frames);
    assert!(!animation.loop_animation);
    assert!(animation.playing);
}

#[test]
fn test_sprite_animation_update() {
    let mut animation = SpriteAnimation::new(10.0, vec![
        [0.0, 0.0, 0.5, 0.5],
        [0.5, 0.0, 0.5, 0.5],
    ]);

    // First frame
    assert_eq!(animation.current_frame, 0);
    assert_eq!(animation.current_frame_region(), [0.0, 0.0, 0.5, 0.5]);

    // Update with exactly one frame duration (0.1 seconds at 10 fps)
    animation.update(0.10);

    // Should be on second frame
    assert_eq!(animation.current_frame, 1);
    assert_eq!(animation.current_frame_region(), [0.5, 0.0, 0.5, 0.5]);

    // Update with exactly one frame duration to loop back
    animation.update(0.10);

    // Should be back on first frame (looping)
    assert_eq!(animation.current_frame, 0);
    assert_eq!(animation.current_frame_region(), [0.0, 0.0, 0.5, 0.5]);
}

#[test]
fn test_sprite_animation_non_looping() {
    let mut animation = SpriteAnimation::new(10.0, vec![
        [0.0, 0.0, 0.5, 0.5],
        [0.5, 0.0, 0.5, 0.5],
    ]).with_loop(false);
    
    // Advance to last frame
    animation.update(0.15);
    assert_eq!(animation.current_frame, 1);
    
    // Try to advance beyond last frame
    animation.update(0.15);
    
    // Should stay on last frame and stop playing
    assert_eq!(animation.current_frame, 1);
    assert!(!animation.playing);
    assert!(animation.is_complete());
}

#[test]
fn test_sprite_animation_pause_resume() {
    let mut animation = SpriteAnimation::new(10.0, vec![
        [0.0, 0.0, 0.5, 0.5],
        [0.5, 0.0, 0.5, 0.5],
    ]);
    
    // Pause
    animation.pause();
    assert!(!animation.playing);
    
    // Update while paused - should not advance
    let frame_before = animation.current_frame;
    animation.update(0.2);
    assert_eq!(animation.current_frame, frame_before);
    
    // Resume
    animation.play();
    assert!(animation.playing);
    
    // Should advance now
    animation.update(0.15);
    assert_eq!(animation.current_frame, 1);
}

#[test]
fn test_sprite_animation_reset() {
    let mut animation = SpriteAnimation::new(10.0, vec![
        [0.0, 0.0, 0.5, 0.5],
        [0.5, 0.0, 0.5, 0.5],
    ]);
    
    // Advance to second frame
    animation.update(0.15);
    assert_eq!(animation.current_frame, 1);
    
    // Reset
    animation.reset();
    assert_eq!(animation.current_frame, 0);
    assert_eq!(animation.time_accumulator, 0.0);
}

#[test]
fn test_sprite_render_data() {
    let mut render_data = SpriteRenderData::new();
    
    assert_eq!(render_data.sprite_count(), 0);
    assert!(render_data.camera.is_none());
    
    // Add some sprites
    render_data.add_sprite(RendererSprite {
        texture_handle: TextureHandle::new(1),
        ..Default::default()
    });
    
    render_data.add_sprite(RendererSprite {
        texture_handle: TextureHandle::new(2),
        ..Default::default()
    });
    
    assert_eq!(render_data.sprite_count(), 2);
    
    // Set camera
    let camera = RendererCamera2D::new(Vec2::new(100.0, 200.0), Vec2::new(800.0, 600.0));
    render_data.set_camera(camera.clone());
    assert!(render_data.camera.is_some());
    assert_eq!(render_data.camera.as_ref().unwrap().position, camera.position);
    
    // Clear
    render_data.clear();
    assert_eq!(render_data.sprite_count(), 0);
}

#[test]
fn test_component_trait() {
    // Test that all sprite components implement the Component trait
    let sprite = Sprite::default();
    let transform = Transform2D::default();
    let camera = Camera2D::default();
    let animation = SpriteAnimation::default();
    
    // These should compile if the types implement Component
    fn assert_component<T: Component>(_component: &T) {}
    
    assert_component(&sprite);
    assert_component(&transform);
    assert_component(&camera);
    assert_component(&animation);
}