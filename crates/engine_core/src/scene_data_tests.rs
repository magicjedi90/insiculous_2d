//! Tests for `scene_data` (hoisted to a sibling file for size).

use std::collections::HashMap;

use crate::scene_data::*;

#[test]
fn test_editor_settings_serialization() {
    let settings = EditorSettings {
        camera_position: (150.0, -200.0),
        camera_zoom: 1.5,
    };

    let ron_str = ron::ser::to_string_pretty(&settings, ron::ser::PrettyConfig::default())
        .expect("Failed to serialize");

    let parsed: EditorSettings = ron::from_str(&ron_str).expect("Failed to parse");
    assert_eq!(parsed.camera_position, (150.0, -200.0));
    assert_eq!(parsed.camera_zoom, 1.5);
}

#[test]
fn test_scene_data_with_editor_settings() {
    let scene = SceneData {
        name: "Test".to_string(),
        editor: Some(EditorSettings {
            camera_position: (100.0, 50.0),
            camera_zoom: 2.0,
        }),
        ..Default::default()
    };

    let config = ron::ser::PrettyConfig::default().struct_names(true);
    let ron_str = ron::ser::to_string_pretty(&scene, config)
        .expect("Failed to serialize");

    // RON serializes with struct names when struct_names(true) is set
    assert!(ron_str.contains("camera_position"));

    let parsed: SceneData = ron::from_str(&ron_str).expect("Failed to parse");
    assert!(parsed.editor.is_some());
    assert_eq!(parsed.editor.unwrap().camera_zoom, 2.0);
}

#[test]
fn test_scene_data_without_editor_settings_backward_compat() {
    // Old scene format without editor field
    let scene_ron = r#"
        SceneData(
            name: "Old Scene",
            entities: [],
        )
    "#;

    let parsed: SceneData = ron::from_str(scene_ron).expect("Failed to parse");
    assert!(parsed.editor.is_none());
}

#[test]
fn test_scene_data_serialization() {
    let scene = SceneData {
        name: "Test Scene".to_string(),
        physics: Some(PhysicsSettings::default()),
        editor: None,
        prefabs: HashMap::new(),
        entities: vec![EntityData {
            name: Some("player".to_string()),
            prefab: None,
            parent: None,
            overrides: Vec::new(),
            components: vec![
                ComponentData::Transform2D {
                    position: (100.0, 200.0),
                    rotation: 0.0,
                    scale: (1.0, 1.0),
                },
                ComponentData::Sprite {
                    texture: "#white".to_string(),
                    offset: (0.0, 0.0),
                    rotation: 0.0,
                    scale: (1.0, 1.0),
                    color: (1.0, 0.0, 0.0, 1.0),
                    depth: 0.0,
                    emissive: 0.0,
                },
            ],
            children: Vec::new(),
        }],
    };

    let ron_str = ron::ser::to_string_pretty(&scene, ron::ser::PrettyConfig::default())
        .expect("Failed to serialize");

    let parsed: SceneData = ron::from_str(&ron_str).expect("Failed to parse");
    assert_eq!(parsed.name, "Test Scene");
    assert_eq!(parsed.entities.len(), 1);
}

#[test]
fn test_prefab_with_overrides() {
    let scene = SceneData {
        name: "Prefab Test".to_string(),
        physics: None,
        editor: None,
        prefabs: {
            let mut map = HashMap::new();
            map.insert(
                "Enemy".to_string(),
                PrefabData {
                    components: vec![
                        ComponentData::Transform2D {
                            position: (0.0, 0.0),
                            rotation: 0.0,
                            scale: (1.0, 1.0),
                        },
                        ComponentData::Sprite {
                            texture: "#white".to_string(),
                            offset: (0.0, 0.0),
                            rotation: 0.0,
                            scale: (1.0, 1.0),
                            color: (1.0, 0.0, 0.0, 1.0),
                            depth: 0.0,
                            emissive: 0.0,
                        },
                    ],
                },
            );
            map
        },
        entities: vec![EntityData {
            name: Some("enemy1".to_string()),
            prefab: Some("Enemy".to_string()),
            parent: None,
            overrides: vec![ComponentData::Transform2D {
                position: (500.0, 100.0),
                rotation: 0.0,
                scale: (1.0, 1.0),
            }],
            components: Vec::new(),
            children: Vec::new(),
        }],
    };

    let ron_str = ron::ser::to_string_pretty(&scene, ron::ser::PrettyConfig::default())
        .expect("Failed to serialize");

    assert!(ron_str.contains("Enemy"));
    assert!(ron_str.contains("enemy1"));
}

#[test]
fn test_physics_components() {
    let entity = EntityData {
        name: Some("physics_entity".to_string()),
        prefab: None,
        parent: None,
        overrides: Vec::new(),
        components: vec![
            ComponentData::RigidBody {
                body_type: RigidBodyTypeData::Dynamic,
                velocity: (0.0, 0.0),
                angular_velocity: 0.0,
                gravity_scale: 1.0,
                linear_damping: 5.0,
                angular_damping: 0.0,
                can_rotate: false,
                ccd_enabled: false,
            },
            ComponentData::Collider {
                shape: ColliderShapeData::Box {
                    half_extents: (40.0, 40.0),
                },
                offset: (0.0, 0.0),
                is_sensor: false,
                friction: 0.8,
                restitution: 0.0,
            },
        ],
        children: Vec::new(),
    };

    let ron_str = ron::ser::to_string_pretty(&entity, ron::ser::PrettyConfig::default())
        .expect("Failed to serialize");

    assert!(ron_str.contains("RigidBody"));
    assert!(ron_str.contains("Collider"));
}
