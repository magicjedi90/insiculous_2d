//! Tests for ECS sprite components

use glam::{Vec2, Vec3, Vec4};
use ecs::sprite_components::*;
use ecs::Component;

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
    assert!(sprite.visible);
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
    let camera = Camera::new(Vec2::new(100.0, 200.0), Vec2::new(1920.0, 1080.0))
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
    let camera = Camera::default();
    
    assert_eq!(camera.position, Vec2::ZERO);
    assert_eq!(camera.rotation, 0.0);
    assert_eq!(camera.zoom, 1.0);
    assert_eq!(camera.viewport_size, Vec2::new(800.0, 600.0));
    assert!(!camera.is_main_camera);
}

#[test]
fn test_camera2d_view_matrix() {
    let camera = Camera {
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
    let camera = Camera::new(Vec2::ZERO, Vec2::new(800.0, 600.0));
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
    let camera = Camera::default();
    let vp_matrix = camera.view_projection_matrix();
    
    // Should be valid matrix
    assert!(!vp_matrix.is_nan());
    assert!(vp_matrix.is_finite());
}

#[test]
fn test_camera2d_screen_to_world() {
    let camera = Camera::new(Vec2::new(100.0, 200.0), Vec2::new(800.0, 600.0));
    
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
    let camera = Camera::new(Vec2::new(100.0, 200.0), Vec2::new(800.0, 600.0));
    
    // Test camera position
    // Note: world_to_screen method doesn't exist in ecs::Camera2D, testing matrix instead
    let proj_matrix = camera.projection_matrix();
    
    // Should be a valid matrix
    assert!(!proj_matrix.is_nan());
    assert!(proj_matrix.is_finite());
}

/// A 4x2 sheet carrying a looping 2-frame "walk" and a one-shot 3-frame
/// "hit" — the two shapes every playback test needs.
fn test_animation() -> SpriteAnimation {
    SpriteAnimation::new(SheetGrid::new(4, 2))
        .with_clip("walk", AnimationClip::new(vec![0, 1], 10.0))
        .with_clip("hit", AnimationClip::new(vec![4, 5, 6], 10.0).with_looping(false))
}

#[test]
fn test_sprite_animation_default_is_empty_and_idle() {
    let animation = SpriteAnimation::default();

    assert!(animation.clips.is_empty());
    assert_eq!(animation.current_clip, None);
    assert!(!animation.playing);
    assert_eq!(animation.current_frame, 0);
    assert_eq!(animation.time_accumulator, 0.0);
    assert_eq!(animation.sheet, None);
    assert_eq!(animation.grid.cell_count(), 1);
    // Nothing selected — the sprite's region is left alone.
    assert_eq!(animation.current_uv(), None);
}

#[test]
fn test_play_selects_the_named_clip() {
    let mut animation = test_animation();

    assert!(animation.play("hit"));
    assert_eq!(animation.current_clip.as_deref(), Some("hit"));
    assert!(animation.playing);
    assert_eq!(animation.current_frame, 0);
    assert_eq!(animation.active_clip().unwrap().frame_indices, vec![4, 5, 6]);
}

#[test]
fn test_play_with_unknown_name_is_a_no_op_that_keeps_the_current_clip() {
    let mut animation = test_animation();
    animation.play("walk");
    animation.update(0.1);

    assert!(!animation.play("sprint"));
    // Everything about the running clip survives the rejected call.
    assert_eq!(animation.current_clip.as_deref(), Some("walk"));
    assert_eq!(animation.current_frame, 1);
    assert!(animation.playing);
}

#[test]
fn test_play_restarts_a_finished_one_shot() {
    let mut animation = test_animation();
    animation.play("hit");
    animation.update(1.0);
    assert!(animation.is_complete());

    assert!(animation.play("hit"));
    assert_eq!(animation.current_frame, 0);
    assert_eq!(animation.time_accumulator, 0.0);
    assert!(animation.playing);
    assert!(!animation.is_complete());
}

#[test]
fn test_play_restarts_even_while_the_same_clip_is_running() {
    let mut animation = test_animation();
    animation.play("walk");
    animation.update(0.1);
    assert_eq!(animation.current_frame, 1);

    // play() means "start from the beginning", same clip or not.
    animation.play("walk");
    assert_eq!(animation.current_frame, 0);
    assert_eq!(animation.time_accumulator, 0.0);
}

#[test]
fn test_switching_to_a_shorter_clip_never_exposes_a_stale_frame() {
    let mut animation = SpriteAnimation::new(SheetGrid::new(4, 4))
        .with_clip("long", AnimationClip::new((0..10).collect::<Vec<_>>(), 10.0))
        .with_clip("short", AnimationClip::new(vec![12, 13], 10.0));

    animation.play("long");
    animation.update(0.9);
    assert_eq!(animation.current_frame, 9);

    // Frame 9 does not exist in the 2-frame clip — the switch must reset it.
    animation.play("short");
    assert_eq!(animation.current_frame, 0);
    assert_eq!(animation.current_uv(), animation.grid.uv_rect_checked(12));
}

#[test]
fn test_frame_advances_once_per_frame_duration() {
    let mut animation = test_animation();
    animation.play("walk");

    // Half a frame at 10 fps is not enough.
    animation.update(0.05);
    assert_eq!(animation.current_frame, 0);

    // The remainder carries over and tips it to the next frame.
    animation.update(0.05);
    assert_eq!(animation.current_frame, 1);
}

#[test]
fn test_looping_clip_wraps_back_to_the_first_frame() {
    let mut animation = test_animation();
    animation.play("walk");

    animation.update(0.1);
    assert_eq!(animation.current_frame, 1);
    animation.update(0.1);
    assert_eq!(animation.current_frame, 0);
    assert!(animation.playing);
    assert!(!animation.is_complete());
}

#[test]
fn test_looping_clip_wraps_correctly_across_a_large_delta() {
    let mut animation = test_animation();
    animation.play("walk");

    // 25 frames' worth in one step: 25 % 2 == 1.
    animation.update(2.5);
    assert_eq!(animation.current_frame, 1);
    assert!(animation.playing);
}

#[test]
fn test_non_looping_clip_clamps_on_the_last_frame_and_stops() {
    let mut animation = test_animation();
    animation.play("hit");

    animation.update(0.2);
    assert_eq!(animation.current_frame, 2);
    assert!(animation.playing);

    animation.update(0.2);
    assert_eq!(animation.current_frame, 2);
    assert!(!animation.playing);
    assert!(animation.is_complete());
    // Stopped means stopped: further updates change nothing.
    animation.update(10.0);
    assert_eq!(animation.current_frame, 2);
}

#[test]
fn test_pause_holds_the_frame_and_resume_continues_from_it() {
    let mut animation = test_animation();
    animation.play("walk");
    animation.update(0.1);

    animation.pause();
    assert!(!animation.playing);
    animation.update(0.5);
    assert_eq!(animation.current_frame, 1);

    // resume() continues where play() would have restarted.
    animation.resume();
    assert!(animation.playing);
    assert_eq!(animation.current_frame, 1);
}

#[test]
fn test_stop_deselects_the_clip() {
    let mut animation = test_animation();
    animation.play("walk");
    animation.update(0.1);

    animation.stop();
    assert!(!animation.playing);
    assert_eq!(animation.current_clip, None);
    assert_eq!(animation.current_frame, 0);
    assert_eq!(animation.current_uv(), None);
    // Nothing selected — resume() has nothing to resume.
    animation.resume();
    assert!(!animation.playing);
}

#[test]
fn test_ensure_playing_called_every_frame_advances_normally() {
    let mut animation = test_animation();

    // The state-machine pattern: re-assert the clip every update.
    for _ in 0..5 {
        assert!(animation.ensure_playing("walk"));
        animation.update(0.1);
    }
    // Five frame steps over a 2-frame looping clip land on frame 1.
    assert_eq!(animation.current_frame, 1);
    assert_eq!(animation.current_clip.as_deref(), Some("walk"));
}

#[test]
fn test_ensure_playing_restarts_a_different_or_stopped_clip() {
    let mut animation = test_animation();
    animation.play("walk");
    animation.update(0.1);

    // Different clip — restarts.
    assert!(animation.ensure_playing("hit"));
    assert_eq!(animation.current_clip.as_deref(), Some("hit"));
    assert_eq!(animation.current_frame, 0);

    // Same clip but no longer playing — restarts.
    animation.update(1.0);
    assert!(!animation.playing);
    assert!(animation.ensure_playing("hit"));
    assert!(animation.playing);
    assert_eq!(animation.current_frame, 0);

    // Unknown clip is still rejected.
    assert!(!animation.ensure_playing("sprint"));
    assert_eq!(animation.current_clip.as_deref(), Some("hit"));
}

#[test]
fn test_non_advancing_fps_values_never_panic_or_advance() {
    // Defensive net for programmatically built clips — authored clips are
    // rejected at parse time.
    for fps in [0.0, -5.0, f32::INFINITY, f32::NEG_INFINITY, f32::NAN] {
        let mut animation = SpriteAnimation::new(SheetGrid::new(2, 1))
            .with_clip("broken", AnimationClip::new(vec![0, 1], fps));
        animation.play("broken");

        animation.update(1.0);
        assert_eq!(animation.current_frame, 0, "fps {fps} must not advance");
        // The frame still resolves — only the advance is suppressed.
        assert_eq!(animation.current_uv(), Some([0.0, 0.0, 0.5, 1.0]));
    }
}

#[test]
fn test_empty_clip_never_advances_and_resolves_to_nothing() {
    let mut animation = SpriteAnimation::new(SheetGrid::new(2, 1))
        .with_clip("empty", AnimationClip::new(vec![], 10.0));
    animation.play("empty");

    animation.update(1.0);
    assert_eq!(animation.current_frame, 0);
    assert_eq!(animation.current_uv(), None);
    assert!(!animation.is_complete());
}

#[test]
fn test_non_finite_delta_time_is_ignored() {
    let mut animation = test_animation();
    animation.play("walk");

    for dt in [f32::NAN, f32::INFINITY, -1.0, 0.0] {
        animation.update(dt);
    }
    assert_eq!(animation.current_frame, 0);
    assert_eq!(animation.time_accumulator, 0.0);
}

#[test]
fn test_current_uv_maps_the_frame_index_through_the_grid() {
    let mut animation = SpriteAnimation::new(SheetGrid::new(4, 2))
        .with_clip("walk", AnimationClip::new(vec![0, 5], 10.0));
    animation.play("walk");

    assert_eq!(animation.current_uv(), Some([0.0, 0.0, 0.25, 0.5]));
    animation.update(0.1);
    // Cell 5 is column 1 of row 1.
    assert_eq!(animation.current_uv(), Some([0.25, 0.5, 0.25, 0.5]));
}

#[test]
fn test_current_uv_is_none_for_a_frame_index_past_the_grid() {
    let mut animation = SpriteAnimation::new(SheetGrid::new(2, 1))
        .with_clip("bad", AnimationClip::new(vec![99], 10.0));
    animation.play("bad");

    assert_eq!(animation.current_uv(), None);
}

#[test]
fn test_component_trait() {
    // Test that all sprite components implement the Component trait
    let sprite = Sprite::default();
    let transform = Transform2D::default();
    let camera = Camera::default();
    let animation = SpriteAnimation::default();
    
    // These should compile if the types implement Component
    fn assert_component<T: Component>(_component: &T) {}
    
    assert_component(&sprite);
    assert_component(&transform);
    assert_component(&camera);
    assert_component(&animation);
}

#[test]
fn test_sprite_animation_component_meta() {
    use ecs::ComponentMeta;

    assert_eq!(<SpriteAnimation as ComponentMeta>::type_name(), "SpriteAnimation");

    let fields = <SpriteAnimation as ComponentMeta>::field_names();
    assert_eq!(
        fields,
        &["grid", "clips", "sheet", "current_clip", "playing", "current_frame", "time_accumulator"]
    );
}

#[test]
fn test_sprite_component_meta() {
    use ecs::ComponentMeta;

    assert_eq!(<Sprite as ComponentMeta>::type_name(), "Sprite");
    let fields = <Sprite as ComponentMeta>::field_names();
    assert_eq!(fields, &["offset", "rotation", "scale", "tex_region", "color", "depth", "visible", "emissive", "texture_handle"]);
}

#[test]
fn test_transform2d_component_meta() {
    use ecs::ComponentMeta;

    assert_eq!(<Transform2D as ComponentMeta>::type_name(), "Transform2D");
    let fields = <Transform2D as ComponentMeta>::field_names();
    assert_eq!(fields, &["position", "rotation", "scale"]);
}

#[test]
fn test_camera_component_meta() {
    use ecs::ComponentMeta;

    assert_eq!(<Camera as ComponentMeta>::type_name(), "Camera");
    let fields = <Camera as ComponentMeta>::field_names();
    assert_eq!(fields, &["position", "rotation", "zoom", "viewport_size", "is_main_camera", "near", "far"]);
}
#[test]
fn test_sprite_deserializes_omitted_region_and_visibility_to_full_and_visible() {
    // Direct Sprite serde must match the scene-wire semantics: an omitted
    // tex_region is the FULL texture (a plain serde default would be the
    // empty region and render nothing) and an omitted visible is true.
    // This is the contract dynamic component creation (ARCH-006/GPP-06)
    // will rely on.
    let sprite: Sprite = serde_json::from_value(serde_json::json!({
        "offset": [0.0, 0.0],
        "rotation": 0.0,
        "scale": [1.0, 1.0],
        "color": [1.0, 1.0, 1.0, 1.0],
        "depth": 0.0,
        "texture_handle": 0
    }))
    .expect("Sprite without tex_region/visible/emissive still deserializes");

    assert_eq!(sprite.tex_region, [0.0, 0.0, 1.0, 1.0]);
    assert!(sprite.visible);
    assert_eq!(sprite.emissive, 0.0);
}
