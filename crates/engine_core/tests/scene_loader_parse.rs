//! Parse-level tests for `SceneLoader` (public API only — tests that need
//! private loader methods stay inline in `scene_loader.rs`).

use engine_core::scene_data::ComponentData;
use engine_core::scene_loader::SceneLoader;

#[test]
fn test_parse_scene_basic() {
    let scene_ron = r#"
        SceneData(
            name: "Test Scene",
            entities: [
                EntityData(
                    name: Some("player"),
                    components: [
                        Transform2D(position: (100.0, 200.0)),
                    ],
                ),
            ],
        )
    "#;

    let scene = SceneLoader::parse(scene_ron).unwrap();
    assert_eq!(scene.name, "Test Scene");
    assert_eq!(scene.entities.len(), 1);
    assert_eq!(scene.entities[0].name, Some("player".to_string()));
}

#[test]
fn test_parse_scene_with_prefabs() {
    let scene_ron = r##"
        SceneData(
            name: "Prefab Test",
            prefabs: {
                "Enemy": PrefabData(
                    components: [
                        Transform2D(position: (0.0, 0.0)),
                        Sprite(texture: "#white", color: (1.0, 0.0, 0.0, 1.0)),
                    ],
                ),
            },
            entities: [
                EntityData(
                    name: Some("enemy1"),
                    prefab: Some("Enemy"),
                    overrides: [
                        Transform2D(position: (500.0, 100.0)),
                    ],
                ),
            ],
        )
    "##;

    let scene = SceneLoader::parse(scene_ron).unwrap();
    assert_eq!(scene.prefabs.len(), 1);
    assert!(scene.prefabs.contains_key("Enemy"));
}

#[test]
fn test_parse_sprite_emissive_defaults_to_zero() {
    let scene_ron = r##"
        SceneData(
            name: "Emissive Default",
            entities: [
                EntityData(
                    components: [
                        Sprite(texture: "#white"),
                    ],
                ),
            ],
        )
    "##;

    let scene = SceneLoader::parse(scene_ron).unwrap();
    match &scene.entities[0].components[0] {
        ComponentData::Sprite { emissive, .. } => assert_eq!(*emissive, 0.0),
        other => panic!("Expected Sprite, got {:?}", other),
    }
}

#[test]
fn test_parse_sprite_emissive_explicit() {
    let scene_ron = r##"
        SceneData(
            name: "Emissive",
            entities: [
                EntityData(
                    components: [
                        Sprite(texture: "#white", emissive: 0.9),
                    ],
                ),
            ],
        )
    "##;

    let scene = SceneLoader::parse(scene_ron).unwrap();
    match &scene.entities[0].components[0] {
        ComponentData::Sprite { emissive, .. } => assert_eq!(*emissive, 0.9),
        other => panic!("Expected Sprite, got {:?}", other),
    }
}

#[test]
fn test_parse_sprite_tex_region_and_visible_default() {
    // Scenes written before these fields existed must load unchanged:
    // full-texture region, visible.
    let scene_ron = r##"
        SceneData(
            name: "Region Default",
            entities: [
                EntityData(
                    components: [
                        Sprite(texture: "#white"),
                    ],
                ),
            ],
        )
    "##;

    let scene = SceneLoader::parse(scene_ron).unwrap();
    match &scene.entities[0].components[0] {
        ComponentData::Sprite { tex_region, visible, .. } => {
            assert_eq!(*tex_region, (0.0, 0.0, 1.0, 1.0));
            assert!(*visible);
        }
        other => panic!("Expected Sprite, got {:?}", other),
    }
}

#[test]
fn test_parse_sprite_tex_region_and_visible_explicit() {
    let scene_ron = r##"
        SceneData(
            name: "Region",
            entities: [
                EntityData(
                    components: [
                        Sprite(
                            texture: "#white",
                            tex_region: (0.25, 0.5, 0.25, 0.5),
                            visible: false,
                        ),
                    ],
                ),
            ],
        )
    "##;

    let scene = SceneLoader::parse(scene_ron).unwrap();
    match &scene.entities[0].components[0] {
        ComponentData::Sprite { tex_region, visible, .. } => {
            assert_eq!(*tex_region, (0.25, 0.5, 0.25, 0.5));
            assert!(!*visible);
        }
        other => panic!("Expected Sprite, got {:?}", other),
    }
}

#[test]
fn test_parse_entity_tag_component() {
    let scene_ron = r#"
        SceneData(
            name: "Tag Test",
            entities: [
                EntityData(
                    name: Some("goblin"),
                    components: [
                        EntityTag(tag: "enemy"),
                    ],
                ),
            ],
        )
    "#;

    let scene = SceneLoader::parse(scene_ron).unwrap();
    match &scene.entities[0].components[0] {
        ComponentData::EntityTag { tag } => assert_eq!(tag, "enemy"),
        other => panic!("Expected EntityTag, got {:?}", other),
    }
}

#[test]
fn test_tilemap_parses_and_instantiates_with_resolved_tileset() {
    use ecs::{Tilemap, World};
    use engine_core::scene_data::SceneLoadError;
    use engine_core::TextureResolver;
    use renderer::texture::TextureHandle;

    struct StubResolver;
    impl TextureResolver for StubResolver {
        fn resolve_texture(&mut self, _texture_ref: &str) -> Result<TextureHandle, SceneLoadError> {
            Ok(TextureHandle::WHITE)
        }
    }

    let scene_ron = r##"
        SceneData(
            name: "Tilemap Test",
            entities: [
                EntityData(
                    name: Some("level"),
                    components: [
                        Transform2D(position: (-160.0, 120.0)),
                        Tilemap(
                            tileset: "#white",
                            width: 3,
                            height: 2,
                            tile_size: 40.0,
                            tiles: [1, 0, 2, 0, 3, 0],
                            tile_uv_size: (0.25, 0.25),
                        ),
                    ],
                ),
            ],
        )
    "##;

    let scene = SceneLoader::parse(scene_ron).unwrap();
    let mut world = World::new();
    SceneLoader::instantiate(&scene, &mut world, &mut StubResolver).unwrap();

    let entity = world.entities()[0];
    let tilemap = world.get::<Tilemap>(entity).expect("Tilemap missing after load");
    assert_eq!(tilemap.width, 3);
    assert_eq!(tilemap.height, 2);
    assert_eq!(tilemap.tileset, TextureHandle::WHITE.id);
    assert_eq!(tilemap.tiles, vec![1, 0, 2, 0, 3, 0]);
    assert_eq!(tilemap.depth, -1.0); // serde default
    assert_eq!(tilemap.sprite_instances().count(), 3);
}

#[test]
fn test_bundled_example_scenes_parse() {
    // The example scene files checked into the repo must always parse —
    // hello_world.scene.ron doubles as the editor demo's level.
    for name in ["hello_world.scene.ron", "behavior_demo.scene.ron"] {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/assets/scenes/");
        let text = std::fs::read_to_string(format!("{path}{name}"))
            .unwrap_or_else(|e| panic!("read {name}: {e}"));
        let scene = SceneLoader::parse(&text).unwrap_or_else(|e| panic!("parse {name}: {e}"));
        assert!(!scene.entities.is_empty(), "{name} has entities");
    }
}

#[test]
fn test_hello_world_scene_has_camera_follow_setup() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/assets/scenes/hello_world.scene.ron"
    );
    let text = std::fs::read_to_string(path).unwrap();
    let scene = SceneLoader::parse(&text).unwrap();

    // A main-camera entity with the CameraFollow behavior
    let camera = scene
        .entities
        .iter()
        .find(|e| e.name.as_deref() == Some("camera"))
        .expect("camera entity present");
    assert!(camera.components.iter().any(|c| matches!(
        c,
        ComponentData::Camera2D { is_main_camera: true, .. }
    )));
    assert!(camera.components.iter().any(|c| matches!(
        c,
        ComponentData::Behavior(engine_core::scene_data::BehaviorData::CameraFollow {
            target_tag, look_ahead, ..
        })
            if target_tag == "player" && *look_ahead == (220.0, 140.0)
    )));

    // The player prefab carries the tag the camera follows
    let player_prefab = &scene.prefabs["Player"];
    assert!(player_prefab.components.iter().any(|c| matches!(
        c,
        ComponentData::EntityTag { tag } if tag == "player"
    )));
}

#[test]
fn test_legacy_camera_follow_scene_without_look_ahead_still_parses() {
    // Scenes authored before look-ahead existed must keep loading, with the
    // feature defaulted off (serde defaults on both new fields).
    let text = r#"SceneData(
        name: "Legacy",
        entities: [
            EntityData(
                name: Some("camera"),
                components: [
                    Transform2D(position: (0.0, 0.0)),
                    Behavior(CameraFollow(
                        target_tag: "player",
                        lerp_speed: 0.12,
                        offset: (0.0, 60.0),
                        dead_zone: Some((160.0, 100.0)),
                    )),
                ],
            ),
        ],
    )"#;

    let scene = SceneLoader::parse(text).expect("legacy scene must parse");
    let behavior = scene.entities[0]
        .components
        .iter()
        .find_map(|c| match c {
            ComponentData::Behavior(b) => Some(b),
            _ => None,
        })
        .expect("camera behavior present");

    match behavior {
        engine_core::scene_data::BehaviorData::CameraFollow { look_ahead, look_ahead_lerp, .. } => {
            assert_eq!(*look_ahead, (0.0, 0.0), "look-ahead defaults to disabled");
            assert_eq!(*look_ahead_lerp, 0.08);
        }
        _ => panic!("expected CameraFollow"),
    }
}
