//! Tests for ECS sprite components

use ecs::sprite_components::*;
use ecs::Component;

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
