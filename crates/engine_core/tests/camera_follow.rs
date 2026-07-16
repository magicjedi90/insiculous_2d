//! Acceptance tests for `Behavior::CameraFollow` (Phase B, Gap 1).
//!
//! All headless: a `World`, a `BehaviorRunner`, and fixed 60 FPS steps —
//! no physics, so position commands write `Transform2D` directly.

use ecs::behavior::{Behavior, EntityTag};
use ecs::sprite_components::Transform2D;
use ecs::{EntityId, World};
use engine_core::behavior_runner::BehaviorRunner;
use glam::Vec2;
use input::InputHandler;

const DT: f32 = 1.0 / 60.0;

/// Spawn a "player"-tagged target at `pos` and a camera-follow entity at the
/// origin with the given behavior fields.
fn setup(
    world: &mut World,
    target_pos: Vec2,
    lerp_speed: f32,
    offset: (f32, f32),
    dead_zone: Option<(f32, f32)>,
) -> EntityId {
    let target = world.create_entity();
    world.add_component(&target, Transform2D::new(target_pos)).unwrap();
    world.add_component(&target, EntityTag::new("player")).unwrap();

    let camera = world.create_entity();
    world.add_component(&camera, Transform2D::new(Vec2::ZERO)).unwrap();
    world
        .add_component(
            &camera,
            Behavior::CameraFollow {
                target_tag: "player".to_string(),
                lerp_speed,
                offset,
                dead_zone,
            },
        )
        .unwrap();
    camera
}

fn position_of(world: &World, entity: EntityId) -> Vec2 {
    world.get::<Transform2D>(entity).unwrap().position
}

fn step_frames(world: &mut World, runner: &mut BehaviorRunner, frames: usize) {
    let input = InputHandler::new();
    for _ in 0..frames {
        runner.update(world, &input, DT, None);
    }
}

#[test]
fn test_camera_converges_within_10_frames_at_lerp_half() {
    let mut world = World::new();
    let mut runner = BehaviorRunner::new();
    let target_pos = Vec2::new(400.0, 300.0);
    let camera = setup(&mut world, target_pos, 0.5, (0.0, 0.0), None);

    let initial_distance = target_pos.length();
    step_frames(&mut world, &mut runner, 10);

    // 0.5 per frame over 10 frames leaves 0.5^10 ≈ 0.1% of the distance.
    let remaining = (target_pos - position_of(&world, camera)).length();
    assert!(
        remaining < initial_distance * 0.01,
        "camera should be within 1% of target after 10 frames, {remaining} px left"
    );
}

#[test]
fn test_lerp_speed_one_snaps_in_a_single_frame() {
    let mut world = World::new();
    let mut runner = BehaviorRunner::new();
    let target_pos = Vec2::new(-250.0, 80.0);
    let camera = setup(&mut world, target_pos, 1.0, (0.0, 0.0), None);

    step_frames(&mut world, &mut runner, 1);
    assert_eq!(position_of(&world, camera), target_pos);
}

#[test]
fn test_offset_shifts_the_convergence_point() {
    let mut world = World::new();
    let mut runner = BehaviorRunner::new();
    let target_pos = Vec2::new(100.0, 100.0);
    let camera = setup(&mut world, target_pos, 1.0, (0.0, 50.0), None);

    step_frames(&mut world, &mut runner, 1);
    assert_eq!(position_of(&world, camera), Vec2::new(100.0, 150.0));
}

#[test]
fn test_dead_zone_ignores_targets_inside_the_box() {
    let mut world = World::new();
    let mut runner = BehaviorRunner::new();
    // Target 40 px away, dead zone half-extent (100, 60) — inside the box.
    let camera = setup(&mut world, Vec2::new(40.0, 30.0), 0.5, (0.0, 0.0), Some((200.0, 120.0)));

    step_frames(&mut world, &mut runner, 30);
    assert_eq!(position_of(&world, camera), Vec2::ZERO);
}

#[test]
fn test_dead_zone_converges_with_target_on_the_box_edge() {
    let mut world = World::new();
    let mut runner = BehaviorRunner::new();
    // Target 400 px right of camera, dead zone 200 px wide (100 half-extent):
    // camera moves right until the target sits on the box's right edge.
    let camera = setup(&mut world, Vec2::new(400.0, 0.0), 0.5, (0.0, 0.0), Some((200.0, 200.0)));

    step_frames(&mut world, &mut runner, 40);
    let pos = position_of(&world, camera);
    assert!(
        (pos - Vec2::new(300.0, 0.0)).length() < 1.0,
        "camera should stop with target on box edge (300, 0), got {pos}"
    );
}

#[test]
fn test_camera_without_target_stays_put() {
    let mut world = World::new();
    let mut runner = BehaviorRunner::new();

    let camera = world.create_entity();
    world
        .add_component(&camera, Transform2D::new(Vec2::new(5.0, 5.0)))
        .unwrap();
    world
        .add_component(
            &camera,
            Behavior::CameraFollow {
                target_tag: "player".to_string(),
                lerp_speed: 0.5,
                offset: (0.0, 0.0),
                dead_zone: None,
            },
        )
        .unwrap();

    step_frames(&mut world, &mut runner, 5);
    assert_eq!(position_of(&world, camera), Vec2::new(5.0, 5.0));
}
