//! Coverage for the editor's engine-time freeze: outside Play mode nothing
//! the engine steps on its own — particles, sprite animations — may advance.

use editor::PlayControlAction;
use engine_core::contexts::GameContext;
use engine_core::Game;
use glam::Vec2;

use super::EditorGame;

struct DummyGame;
impl Game for DummyGame {
    fn update(&mut self, _ctx: &mut GameContext) {}
}

#[test]
fn test_time_scale_is_frozen_while_not_playing() {
    let mut editor = EditorGame::new(DummyGame);
    let mut world = ecs::World::new();

    // Editing: engine-side time stops dead.
    assert_eq!(editor.editor_time_scale(1.0), 0.0);
    assert_eq!(editor.editor_time_scale(1.0), 0.0);

    // Play hands the game back the value it was running at.
    editor.handle_play_action(PlayControlAction::Play, &mut world);
    assert_eq!(editor.editor_time_scale(0.0), 1.0);
    // From then on the game owns it — a game that pauses itself stays paused.
    assert_eq!(editor.editor_time_scale(0.0), 0.0);
    assert_eq!(editor.editor_time_scale(0.5), 0.5);

    // Paused counts as not playing: frozen again, and the game's 0.5 is held.
    editor.handle_play_action(PlayControlAction::Pause, &mut world);
    assert_eq!(editor.editor_time_scale(0.5), 0.0);
    editor.handle_play_action(PlayControlAction::Play, &mut world);
    assert_eq!(editor.editor_time_scale(0.0), 0.5);
}

#[test]
fn test_particles_and_animations_do_not_advance_while_editing() {
    use ecs::sprite_components::{AnimationClip, Sprite, SpriteAnimation};
    use engine_core::particles::{ParticleConfig, ParticleEmitter, ParticleManager, ParticleSystem};

    let mut editor = EditorGame::new(DummyGame);
    let mut world = ecs::World::new();
    let mut particles = ParticleManager::with_capacity(64);

    let emitter_entity = world.create_entity();
    world.add_component(&emitter_entity, common::Transform2D::new(Vec2::ZERO)).ok();
    world
        .add_component(
            &emitter_entity,
            ParticleEmitter::new(30.0, ParticleConfig::default()),
        )
        .ok();

    let animated = world.create_entity();
    world.add_component(&animated, Sprite::new(0)).ok();
    let mut animation = SpriteAnimation::new(common::SheetGrid::new(4, 1))
        .with_clip("walk", AnimationClip::new(vec![0, 1, 2], 10.0));
    animation.play("walk");
    world.add_component(&animated, animation).ok();

    // One second of frames, exactly as the engine's frame tail runs them.
    let step_frames = |world: &mut ecs::World, particles: &mut ParticleManager, scale: f32| {
        for _ in 0..10 {
            let dt = 0.1 * scale;
            ParticleSystem::update(world, particles, dt);
            ecs::System::update(&mut ecs::SpriteAnimationSystem, world, dt);
        }
    };

    let editing_scale = editor.editor_time_scale(1.0);
    step_frames(&mut world, &mut particles, editing_scale);
    assert_eq!(particles.alive_count(), 0, "particles must not emit while editing");
    assert_eq!(
        world.get::<SpriteAnimation>(animated).unwrap().current_frame,
        0,
        "animations must not advance while editing"
    );
    assert_eq!(world.get::<Sprite>(animated).unwrap().tex_region, [0.0, 0.0, 0.25, 1.0]);

    // Play: the same frames now move both.
    editor.handle_play_action(PlayControlAction::Play, &mut world);
    let playing_scale = editor.editor_time_scale(1.0);
    step_frames(&mut world, &mut particles, playing_scale);
    assert!(particles.alive_count() > 0, "particles emit once Playing");
    assert_eq!(
        world.get::<SpriteAnimation>(animated).unwrap().current_frame,
        1,
        "10 frame steps over a looping 3-frame clip land on frame 1"
    );
}

