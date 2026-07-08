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
