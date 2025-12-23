//! Tests for sprite data structures

use glam::{Vec2, Vec3, Vec4};
use renderer::sprite_data::*;
use renderer::TextureHandle;

#[test]
fn test_sprite_vertex_creation() {
    let vertex = SpriteVertex::new(
        Vec3::new(1.0, 2.0, 3.0),
        Vec2::new(0.5, 0.6),
        Vec4::new(1.0, 0.5, 0.0, 1.0)
    );

    assert_eq!(vertex.position, [1.0, 2.0, 3.0]);
    assert_eq!(vertex.tex_coords, [0.5, 0.6]);
    assert_eq!(vertex.color, [1.0, 0.5, 0.0, 1.0]);
}

#[test]
fn test_sprite_instance_creation() {
    let instance = SpriteInstance::new(
        Vec2::new(10.0, 20.0),
        std::f32::consts::PI,
        Vec2::new(2.0, 3.0),
        [0.1, 0.2, 0.3, 0.4],
        Vec4::new(1.0, 1.0, 1.0, 1.0),
        5.0
    );

    assert_eq!(instance.position, [10.0, 20.0]);
    assert_eq!(instance.rotation, std::f32::consts::PI);
    assert_eq!(instance.scale, [2.0, 3.0]);
    assert_eq!(instance.tex_region, [0.1, 0.2, 0.3, 0.4]);
    assert_eq!(instance.color, [1.0, 1.0, 1.0, 1.0]);
    assert_eq!(instance.depth, 5.0);
}

#[test]
fn test_camera2d_default() {
    let camera = Camera2D::default();
    
    assert_eq!(camera.position, Vec2::ZERO);
    assert_eq!(camera.rotation, 0.0);
    assert_eq!(camera.zoom, 1.0);
    assert_eq!(camera.viewport_size, Vec2::new(800.0, 600.0));
    assert_eq!(camera.near, -1000.0);
    assert_eq!(camera.far, 1000.0);
}

#[test]
fn test_camera2d_creation() {
    let camera = Camera2D::new(Vec2::new(100.0, 200.0), Vec2::new(1920.0, 1080.0));
    
    assert_eq!(camera.position, Vec2::new(100.0, 200.0));
    assert_eq!(camera.viewport_size, Vec2::new(1920.0, 1080.0));
    assert_eq!(camera.rotation, 0.0);
    assert_eq!(camera.zoom, 1.0);
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
    let world_point = Vec3::new(100.0, 200.0, 0.0);
    let view_point = view_matrix * Vec4::new(world_point.x, world_point.y, world_point.z, 1.0);
    
    // The camera position should be at the origin in view space
    assert!((view_point.x).abs() < 0.001);
    assert!((view_point.y).abs() < 0.001);
}

#[test]
fn test_camera2d_projection_matrix() {
    let camera = Camera2D::new(Vec2::ZERO, Vec2::new(800.0, 600.0));
    let proj_matrix = camera.projection_matrix();
    
    // Test that projection matrix is orthographic
    let near_point = Vec4::new(0.0, 0.0, -1000.0, 1.0);
    let far_point = Vec4::new(0.0, 0.0, 1000.0, 1.0);
    
    let near_clip = proj_matrix * near_point;
    let far_clip = proj_matrix * far_point;
    
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
    // Check that matrix is finite (no infinite values)
    assert!(vp_matrix.x_axis.x.is_finite());
    assert!(vp_matrix.y_axis.y.is_finite());
    assert!(vp_matrix.z_axis.z.is_finite());
    assert!(vp_matrix.w_axis.w.is_finite());
}

#[test]
fn test_camera2d_screen_to_world() {
    let camera = Camera2D::new(Vec2::new(100.0, 200.0), Vec2::new(800.0, 600.0));
    
    // Test center of screen
    let screen_center = Vec2::new(400.0, 300.0);
    let world_pos = camera.screen_to_world(screen_center);
    
    // Should be close to camera position
    assert!((world_pos.x - camera.position.x).abs() < 1.0);
    assert!((world_pos.y - camera.position.y).abs() < 1.0);
}

#[test]
fn test_camera2d_world_to_screen() {
    let camera = Camera2D::new(Vec2::new(100.0, 200.0), Vec2::new(800.0, 600.0));
    
    // Test camera position
    let screen_pos = camera.world_to_screen(camera.position);
    
    // Should be close to center of screen
    assert!((screen_pos.x - 400.0).abs() < 1.0);
    assert!((screen_pos.y - 300.0).abs() < 1.0);
}

#[test]
fn test_camera_uniform_creation() {
    let camera = Camera2D {
        position: Vec2::new(50.0, 75.0),
        rotation: std::f32::consts::PI,
        zoom: 1.5,
        ..Default::default()
    };

    let uniform = CameraUniform::from_camera(&camera);
    
    // Check that view-projection matrix is valid
    assert!(!uniform.view_projection.iter().any(|row| row.iter().any(|&val| val.is_nan() || val.is_infinite())));
    
    // Check camera position
    assert_eq!(uniform.position, [50.0, 75.0]);
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
fn test_texture_handle_default() {
    let handle = TextureHandle::default();
    assert_eq!(handle.id, 0);
}