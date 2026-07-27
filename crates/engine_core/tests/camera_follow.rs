//! Acceptance tests for `Behavior::CameraFollow` (Phase B, Gap 1) and its
//! input-driven look-ahead.
//!
//! All headless: a `World`, a `BehaviorRunner`, and fixed 60 FPS steps —
//! no physics, so position commands write `Transform2D` directly.
//!
//! Input lifecycle: ONE `InputHandler` lives for the whole simulation (a
//! fresh handler per frame would lose held-key state). Each frame ends the
//! previous frame, queues that frame's events, and processes them before the
//! runner update — holding a key is a single `KeyPressed` whose state
//! persists until the matching `KeyReleased`.
//!
//! Exponential lerps never exactly reach their asymptote, so convergence
//! assertions run to settle and compare within `EPS`.

use ecs::behavior::{Behavior, EntityTag};
use ecs::sprite_components::Transform2D;
use ecs::{EntityId, World};
use engine_core::behavior_runner::BehaviorRunner;
use glam::Vec2;
use input::{InputEvent, InputHandler};
use winit::keyboard::KeyCode;

const DT: f32 = 1.0 / 60.0;
/// Frames to run before asserting a settled position.
const SETTLE_FRAMES: usize = 600;
/// Tolerance for settled-position assertions, in pixels.
const EPS: f32 = 0.5;

/// Behavior fields for a camera-follow test entity.
struct Follow {
    lerp_speed: f32,
    offset: (f32, f32),
    dead_zone: Option<(f32, f32)>,
    look_ahead: (f32, f32),
    look_ahead_lerp: f32,
}

impl Follow {
    /// Plain follow at the given lerp speed: no offset, dead zone, or lead.
    fn plain(lerp_speed: f32) -> Self {
        Self {
            lerp_speed,
            offset: (0.0, 0.0),
            dead_zone: None,
            look_ahead: (0.0, 0.0),
            look_ahead_lerp: 0.08,
        }
    }

    fn with_offset(mut self, offset: (f32, f32)) -> Self {
        self.offset = offset;
        self
    }

    fn with_dead_zone(mut self, dead_zone: (f32, f32)) -> Self {
        self.dead_zone = Some(dead_zone);
        self
    }

    fn with_look_ahead(mut self, look_ahead: (f32, f32)) -> Self {
        self.look_ahead = look_ahead;
        self
    }
}

/// Spawn a "player"-tagged target at `pos` and a camera-follow entity at the
/// origin with the given behavior fields.
fn setup(world: &mut World, target_pos: Vec2, follow: Follow) -> EntityId {
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
                lerp_speed: follow.lerp_speed,
                offset: follow.offset,
                dead_zone: follow.dead_zone,
                look_ahead: follow.look_ahead,
                look_ahead_lerp: follow.look_ahead_lerp,
            },
        )
        .unwrap();
    camera
}

fn position_of(world: &World, entity: EntityId) -> Vec2 {
    world.get::<Transform2D>(entity).unwrap().position
}

/// Advance `frames` frames with no input at all.
fn step_frames(world: &mut World, runner: &mut BehaviorRunner, frames: usize) {
    let input = InputHandler::new();
    for _ in 0..frames {
        runner.update(world, &input, DT, None);
    }
}

/// Advance `frames` frames while `keys` stay held down on the shared handler.
fn step_frames_holding(
    world: &mut World,
    runner: &mut BehaviorRunner,
    input: &mut InputHandler,
    keys: &[KeyCode],
    frames: usize,
) {
    for frame in 0..frames {
        input.end_frame();
        // Press once on the first frame; the handler keeps the key held.
        if frame == 0 {
            for key in keys {
                input.queue_event(InputEvent::KeyPressed(*key));
            }
        }
        input.process_queued_events();
        runner.update(world, input, DT, None);
    }
}

/// Release `keys` and advance `frames` further frames.
fn step_frames_releasing(
    world: &mut World,
    runner: &mut BehaviorRunner,
    input: &mut InputHandler,
    keys: &[KeyCode],
    frames: usize,
) {
    for frame in 0..frames {
        input.end_frame();
        if frame == 0 {
            for key in keys {
                input.queue_event(InputEvent::KeyReleased(*key));
            }
        }
        input.process_queued_events();
        runner.update(world, input, DT, None);
    }
}

fn assert_near(actual: Vec2, expected: Vec2, what: &str) {
    assert!(
        (actual - expected).length() < EPS,
        "{what}: expected ~{expected}, got {actual}"
    );
}

#[test]
fn test_camera_converges_within_10_frames_at_lerp_half() {
    let mut world = World::new();
    let mut runner = BehaviorRunner::new();
    let target_pos = Vec2::new(400.0, 300.0);
    let camera = setup(&mut world, target_pos, Follow::plain(0.5));

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
    let camera = setup(&mut world, target_pos, Follow::plain(1.0));

    step_frames(&mut world, &mut runner, 1);
    assert_eq!(position_of(&world, camera), target_pos);
}

#[test]
fn test_offset_shifts_the_convergence_point() {
    let mut world = World::new();
    let mut runner = BehaviorRunner::new();
    let target_pos = Vec2::new(100.0, 100.0);
    let camera = setup(&mut world, target_pos, Follow::plain(1.0).with_offset((0.0, 50.0)));

    step_frames(&mut world, &mut runner, 1);
    assert_eq!(position_of(&world, camera), Vec2::new(100.0, 150.0));
}

#[test]
fn test_dead_zone_ignores_targets_inside_the_box() {
    let mut world = World::new();
    let mut runner = BehaviorRunner::new();
    // Target 40 px away, dead zone half-extent (100, 60) — inside the box.
    let camera = setup(
        &mut world,
        Vec2::new(40.0, 30.0),
        Follow::plain(0.5).with_dead_zone((200.0, 120.0)),
    );

    step_frames(&mut world, &mut runner, 30);
    assert_eq!(position_of(&world, camera), Vec2::ZERO);
}

#[test]
fn test_dead_zone_converges_with_target_on_the_box_edge() {
    let mut world = World::new();
    let mut runner = BehaviorRunner::new();
    // Target 400 px right of camera, dead zone 200 px wide (100 half-extent):
    // camera moves right until the target sits on the box's right edge.
    let camera = setup(
        &mut world,
        Vec2::new(400.0, 0.0),
        Follow::plain(0.5).with_dead_zone((200.0, 200.0)),
    );

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
                look_ahead: (220.0, 140.0),
                look_ahead_lerp: 0.08,
            },
        )
        .unwrap();

    step_frames(&mut world, &mut runner, 5);
    assert_eq!(position_of(&world, camera), Vec2::new(5.0, 5.0));
}

#[test]
fn test_zero_look_ahead_leaves_held_input_without_effect() {
    let mut world = World::new();
    let mut runner = BehaviorRunner::new();
    let mut input = InputHandler::new();
    let camera = setup(&mut world, Vec2::new(100.0, 0.0), Follow::plain(0.5));

    step_frames_holding(&mut world, &mut runner, &mut input, &[KeyCode::KeyD], SETTLE_FRAMES);

    assert_near(
        position_of(&world, camera),
        Vec2::new(100.0, 0.0),
        "look_ahead (0,0) must behave exactly like plain follow",
    );
}

#[test]
fn test_holding_right_leads_the_camera_by_look_ahead_x() {
    let mut world = World::new();
    let mut runner = BehaviorRunner::new();
    let mut input = InputHandler::new();
    let camera = setup(
        &mut world,
        Vec2::ZERO,
        Follow::plain(0.5).with_look_ahead((220.0, 140.0)),
    );

    step_frames_holding(&mut world, &mut runner, &mut input, &[KeyCode::KeyD], SETTLE_FRAMES);

    assert_near(
        position_of(&world, camera),
        Vec2::new(220.0, 0.0),
        "holding right should lead by look_ahead.x",
    );
}

#[test]
fn test_holding_up_and_down_lead_vertically() {
    for (key, expected_y) in [(KeyCode::KeyW, 140.0), (KeyCode::KeyS, -140.0)] {
        let mut world = World::new();
        let mut runner = BehaviorRunner::new();
        let mut input = InputHandler::new();
        let camera = setup(
            &mut world,
            Vec2::ZERO,
            Follow::plain(0.5).with_look_ahead((220.0, 140.0)),
        );

        step_frames_holding(&mut world, &mut runner, &mut input, &[key], SETTLE_FRAMES);

        assert_near(
            position_of(&world, camera),
            Vec2::new(0.0, expected_y),
            "vertical lead should follow the pressed direction (+y = up)",
        );
    }
}

#[test]
fn test_releasing_the_direction_decays_the_lead() {
    let mut world = World::new();
    let mut runner = BehaviorRunner::new();
    let mut input = InputHandler::new();
    let camera = setup(
        &mut world,
        Vec2::ZERO,
        Follow::plain(0.5).with_look_ahead((220.0, 140.0)),
    );

    step_frames_holding(&mut world, &mut runner, &mut input, &[KeyCode::KeyD], SETTLE_FRAMES);
    step_frames_releasing(&mut world, &mut runner, &mut input, &[KeyCode::KeyD], SETTLE_FRAMES);

    assert_near(
        position_of(&world, camera),
        Vec2::ZERO,
        "releasing should glide back to the plain follow position",
    );
}

#[test]
fn test_opposite_directions_cancel_the_lead() {
    let mut world = World::new();
    let mut runner = BehaviorRunner::new();
    let mut input = InputHandler::new();
    let camera = setup(
        &mut world,
        Vec2::ZERO,
        Follow::plain(0.5).with_look_ahead((220.0, 140.0)),
    );

    step_frames_holding(
        &mut world,
        &mut runner,
        &mut input,
        &[KeyCode::KeyA, KeyCode::KeyD],
        SETTLE_FRAMES,
    );

    assert_near(
        position_of(&world, camera),
        Vec2::ZERO,
        "left + right held should produce no lead",
    );
}

#[test]
fn test_dead_zone_absorbs_part_of_the_lead() {
    let mut world = World::new();
    let mut runner = BehaviorRunner::new();
    let mut input = InputHandler::new();
    // Lead is applied to the focus point BEFORE the dead-zone clamp, so a
    // stationary player leads by look_ahead.x − half-width = 220 − 80.
    let camera = setup(
        &mut world,
        Vec2::ZERO,
        Follow::plain(0.5)
            .with_dead_zone((160.0, 100.0))
            .with_look_ahead((220.0, 0.0)),
    );

    step_frames_holding(&mut world, &mut runner, &mut input, &[KeyCode::KeyD], SETTLE_FRAMES);

    assert_near(
        position_of(&world, camera),
        Vec2::new(140.0, 0.0),
        "dead zone should absorb its half-width of the lead",
    );
}

#[test]
fn test_lead_ramps_instead_of_snapping() {
    let mut world = World::new();
    let mut runner = BehaviorRunner::new();
    let mut input = InputHandler::new();
    let camera = setup(
        &mut world,
        Vec2::ZERO,
        Follow::plain(0.5).with_look_ahead((220.0, 0.0)),
    );

    step_frames_holding(&mut world, &mut runner, &mut input, &[KeyCode::KeyD], 1);

    let x = position_of(&world, camera).x;
    assert!(
        x > 0.0 && x < 220.0,
        "one frame of holding right should ramp partway, got {x}"
    );
}

#[test]
fn test_negative_and_nan_look_ahead_degrade_to_plain_follow() {
    let mut world = World::new();
    let mut runner = BehaviorRunner::new();
    let mut input = InputHandler::new();
    let camera = setup(
        &mut world,
        Vec2::new(100.0, 50.0),
        Follow::plain(0.5).with_look_ahead((-220.0, f32::NAN)),
    );

    for frame in 0..SETTLE_FRAMES {
        input.end_frame();
        if frame == 0 {
            input.queue_event(InputEvent::KeyPressed(KeyCode::KeyD));
        }
        input.process_queued_events();
        runner.update(&mut world, &input, DT, None);
        assert!(
            position_of(&world, camera).is_finite(),
            "bad scene data must never produce a non-finite position"
        );
    }

    assert_near(
        position_of(&world, camera),
        Vec2::new(100.0, 50.0),
        "negative/NaN look-ahead should behave as plain follow",
    );
}
