//! Runtime prefab spawning (PATTERNS_AUDIT.md GPP-07, the Prototype
//! pattern's actual purpose): a loaded scene retains its prefab table and
//! can stamp out new entities from it mid-game — with override semantics —
//! all headless via a stub `TextureResolver`.

use std::collections::HashMap;

use ecs::behavior::EntityTag;
use ecs::sprite_components::{Sprite, Transform2D};
use ecs::World;
use engine_core::prelude::*;
use engine_core::TextureResolver;
use glam::Vec2;
use renderer::TextureHandle;

/// GPU-free resolver: every reference resolves to the built-in white texture.
struct StubResolver;

impl TextureResolver for StubResolver {
    fn resolve_texture(&mut self, _texture_ref: &str) -> Result<TextureHandle, SceneLoadError> {
        Ok(TextureHandle::WHITE)
    }
}

/// A scene with one "Ball" prefab (transform + sprite + tag) and one
/// pre-placed entity using it.
fn ball_scene() -> SceneData {
    let ball_prefab = PrefabData {
        components: vec![
            ComponentData::Transform2D { position: (0.0, 0.0), rotation: 0.0, scale: (1.0, 1.0) },
            ComponentData::Sprite {
                texture: "#white".to_string(),
                offset: (0.0, 0.0),
                rotation: 0.0,
                scale: (16.0, 16.0),
                color: (1.0, 1.0, 1.0, 1.0),
                depth: 0.0,
                emissive: 0.0,
            },
            ComponentData::EntityTag { tag: "ball".to_string() },
        ],
    };

    let mut prefabs = HashMap::new();
    prefabs.insert("Ball".to_string(), ball_prefab);

    SceneData {
        name: "prefab test".to_string(),
        physics: None,
        editor: None,
        prefabs,
        entities: vec![EntityData {
            name: Some("first_ball".to_string()),
            prefab: Some("Ball".to_string()),
            overrides: vec![],
            components: vec![],
            parent: None,
            children: vec![],
        }],
    }
}

#[test]
fn test_instance_retains_prefab_table() {
    let mut world = World::new();
    let instance = SceneLoader::instantiate(&ball_scene(), &mut world, &mut StubResolver).unwrap();
    assert!(instance.has_prefab("Ball"));
    assert!(!instance.has_prefab("Paddle"));
}

#[test]
fn test_spawn_prefab_stamps_out_new_entity() {
    let mut world = World::new();
    let instance = SceneLoader::instantiate(&ball_scene(), &mut world, &mut StubResolver).unwrap();
    let scene_ball = instance.get_entity("first_ball").unwrap();

    let spawned = instance
        .spawn_prefab(&mut world, &mut StubResolver, "Ball", &[])
        .unwrap();

    assert_ne!(spawned, scene_ball, "runtime spawn is a NEW entity");
    assert!(world.get::<Transform2D>(spawned).is_some());
    assert!(world.get::<Sprite>(spawned).is_some());
    assert!(world.get::<EntityTag>(spawned).unwrap().matches("ball"));
    // The scene's own bookkeeping is untouched — the caller owns the spawn.
    assert_eq!(instance.entity_count, 1);
}

#[test]
fn test_spawn_prefab_applies_overrides() {
    let mut world = World::new();
    let instance = SceneLoader::instantiate(&ball_scene(), &mut world, &mut StubResolver).unwrap();

    let overrides = [ComponentData::Transform2D {
        position: (200.0, 300.0),
        rotation: 0.0,
        scale: (1.0, 1.0),
    }];
    let spawned = instance
        .spawn_prefab(&mut world, &mut StubResolver, "Ball", &overrides)
        .unwrap();

    let t = world.get::<Transform2D>(spawned).unwrap();
    assert_eq!(t.position, Vec2::new(200.0, 300.0), "override replaces the prefab transform");
    assert!(
        world.get::<Sprite>(spawned).is_some(),
        "non-overridden prefab components still apply"
    );
}

#[test]
fn test_spawn_unknown_prefab_errors_and_leaves_world_unchanged() {
    let mut world = World::new();
    let instance = SceneLoader::instantiate(&ball_scene(), &mut world, &mut StubResolver).unwrap();
    let before = world.entities().len();

    let result = instance.spawn_prefab(&mut world, &mut StubResolver, "Nope", &[]);
    assert!(matches!(result, Err(SceneLoadError::PrefabNotFound(_))));
    assert_eq!(world.entities().len(), before, "no entity debris on failure");
}

#[test]
fn test_spawn_prefab_failure_removes_half_built_entity() {
    /// A resolver that always fails — simulates a missing texture file.
    struct FailingResolver;
    impl TextureResolver for FailingResolver {
        fn resolve_texture(&mut self, r: &str) -> Result<TextureHandle, SceneLoadError> {
            Err(SceneLoadError::TextureLoadError(r.to_string()))
        }
    }

    let mut world = World::new();
    let instance = SceneLoader::instantiate(&ball_scene(), &mut world, &mut StubResolver).unwrap();
    let before = world.entities().len();

    let result = instance.spawn_prefab(&mut world, &mut FailingResolver, "Ball", &[]);
    assert!(result.is_err());
    assert_eq!(
        world.entities().len(),
        before,
        "a spawn that fails mid-build must not leave a half-built entity"
    );
}
