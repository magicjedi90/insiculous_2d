//! Scene data structures for RON serialization
//!
//! This module defines the data structures used to serialize and deserialize
//! scene files in RON (Rusty Object Notation) format.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Editor-specific settings persisted with the scene
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct EditorSettings {
    /// Camera position when scene was last saved
    #[serde(default)]
    pub camera_position: (f32, f32),
    /// Camera zoom level when scene was last saved
    #[serde(default = "default_zoom")]
    pub camera_zoom: f32,
}

/// Root structure for a scene file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneData {
    /// Scene name
    pub name: String,
    /// Physics settings for this scene
    #[serde(default)]
    pub physics: Option<PhysicsSettings>,
    /// Editor settings (camera position, zoom) - optional for backward compatibility
    #[serde(default)]
    pub editor: Option<EditorSettings>,
    /// Prefab definitions (reusable entity templates)
    #[serde(default)]
    pub prefabs: HashMap<String, PrefabData>,
    /// Entity instances
    #[serde(default)]
    pub entities: Vec<EntityData>,
}

impl Default for SceneData {
    fn default() -> Self {
        Self {
            name: "Untitled".to_string(),
            physics: None,
            editor: None,
            prefabs: HashMap::new(),
            entities: Vec::new(),
        }
    }
}

/// Physics world settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsSettings {
    /// Gravity vector (default: (0.0, -980.0) for platformer)
    #[serde(default = "default_gravity")]
    pub gravity: (f32, f32),
    /// Pixels per meter scale (default: 100.0)
    #[serde(default = "default_pixels_per_meter")]
    pub pixels_per_meter: f32,
    /// Physics timestep in seconds (default: 1/60)
    #[serde(default = "default_timestep")]
    pub timestep: f32,
}

fn default_gravity() -> (f32, f32) {
    (0.0, -980.0)
}

fn default_pixels_per_meter() -> f32 {
    100.0
}

fn default_timestep() -> f32 {
    1.0 / 60.0
}

impl Default for PhysicsSettings {
    fn default() -> Self {
        Self {
            gravity: default_gravity(),
            pixels_per_meter: default_pixels_per_meter(),
            timestep: default_timestep(),
        }
    }
}

/// Prefab definition - a reusable entity template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrefabData {
    /// Components that make up this prefab
    pub components: Vec<ComponentData>,
}

/// Entity instance in a scene
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EntityData {
    /// Optional name for this entity (for lookup)
    #[serde(default)]
    pub name: Option<String>,
    /// Optional prefab to instantiate
    #[serde(default)]
    pub prefab: Option<String>,
    /// Optional parent entity name (for hierarchy)
    ///
    /// The parent must be defined earlier in the entities list so it exists
    /// when this entity is created. Use the parent entity's `name` field.
    #[serde(default)]
    pub parent: Option<String>,
    /// Component overrides (applied on top of prefab)
    #[serde(default)]
    pub overrides: Vec<ComponentData>,
    /// Inline components (if no prefab is used)
    #[serde(default)]
    pub components: Vec<ComponentData>,
    /// Child entities (alternative to using parent field - creates hierarchy inline)
    #[serde(default)]
    pub children: Vec<EntityData>,
}

/// Component data variants for serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComponentData {
    /// Transform component
    Transform2D {
        #[serde(default)]
        position: (f32, f32),
        #[serde(default)]
        rotation: f32,
        #[serde(default = "default_scale")]
        scale: (f32, f32),
    },
    /// Sprite component
    Sprite {
        /// Texture reference: "#white", "#solid:RRGGBB", or file path
        #[serde(default = "default_texture")]
        texture: String,
        #[serde(default)]
        offset: (f32, f32),
        #[serde(default)]
        rotation: f32,
        #[serde(default = "default_scale")]
        scale: (f32, f32),
        #[serde(default = "default_color")]
        color: (f32, f32, f32, f32),
        #[serde(default)]
        depth: f32,
    },
    /// Camera component
    Camera2D {
        #[serde(default)]
        position: (f32, f32),
        #[serde(default)]
        rotation: f32,
        #[serde(default = "default_zoom")]
        zoom: f32,
        #[serde(default = "default_viewport")]
        viewport_size: (f32, f32),
        #[serde(default)]
        is_main_camera: bool,
    },
    /// Sprite animation component
    SpriteAnimation {
        #[serde(default = "default_fps")]
        fps: f32,
        #[serde(default)]
        frames: Vec<(f32, f32, f32, f32)>,
        #[serde(default = "default_true")]
        playing: bool,
        #[serde(default = "default_true")]
        loop_animation: bool,
    },
    /// Rigid body component
    RigidBody {
        #[serde(default)]
        body_type: RigidBodyTypeData,
        #[serde(default)]
        velocity: (f32, f32),
        #[serde(default)]
        angular_velocity: f32,
        #[serde(default = "default_gravity_scale")]
        gravity_scale: f32,
        #[serde(default)]
        linear_damping: f32,
        #[serde(default)]
        angular_damping: f32,
        #[serde(default = "default_true")]
        can_rotate: bool,
        #[serde(default)]
        ccd_enabled: bool,
    },
    /// Collider component
    Collider {
        #[serde(default)]
        shape: ColliderShapeData,
        #[serde(default)]
        offset: (f32, f32),
        #[serde(default)]
        is_sensor: bool,
        #[serde(default = "default_friction")]
        friction: f32,
        #[serde(default)]
        restitution: f32,
    },
    /// Behavior component - defines how an entity responds to input/events
    Behavior(BehaviorData),
    /// Dynamic component loaded via component registry
    ///
    /// This variant allows loading components by type name without hardcoded
    /// handling. The component must be registered in the global ComponentRegistry.
    ///
    /// Note: Full support requires type-erased component storage in World.
    /// Currently logs a warning when encountered.
    Dynamic {
        /// Component type name (must match registry)
        #[serde(rename = "type")]
        component_type: String,
        /// Component data as JSON
        #[serde(flatten)]
        data: serde_json::Value,
    },
}

/// Behavior data for serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BehaviorData {
    /// Player-controlled platformer movement (WASD + Space for jump)
    PlayerPlatformer {
        /// Horizontal movement speed in pixels per second
        #[serde(default = "default_move_speed")]
        move_speed: f32,
        /// Upward impulse applied when jumping
        #[serde(default = "default_jump_impulse")]
        jump_impulse: f32,
        /// Cooldown between jumps in seconds
        #[serde(default = "default_jump_cooldown")]
        jump_cooldown: f32,
        /// Tag to identify this entity for targeting
        #[serde(default = "default_player_tag")]
        tag: String,
    },
    /// Player-controlled top-down movement (WASD in all directions)
    PlayerTopDown {
        /// Movement speed in pixels per second
        #[serde(default = "default_move_speed")]
        move_speed: f32,
        /// Tag to identify this entity for targeting
        #[serde(default = "default_player_tag")]
        tag: String,
    },
    /// Follow another entity by name
    FollowEntity {
        /// Name of the target entity to follow
        target_name: String,
        /// Minimum distance to maintain from target
        #[serde(default = "default_follow_distance")]
        follow_distance: f32,
        /// Movement speed when following
        #[serde(default = "default_follow_speed")]
        follow_speed: f32,
    },
    /// Follow the nearest entity with a specific tag
    FollowTagged {
        /// Tag of entities to follow
        #[serde(default = "default_player_tag")]
        target_tag: String,
        /// Minimum distance to maintain from target
        #[serde(default = "default_follow_distance")]
        follow_distance: f32,
        /// Movement speed when following
        #[serde(default = "default_follow_speed")]
        follow_speed: f32,
    },
    /// Patrol between two world positions
    Patrol {
        /// First patrol point (x, y)
        point_a: (f32, f32),
        /// Second patrol point (x, y)
        point_b: (f32, f32),
        /// Movement speed
        #[serde(default = "default_patrol_speed")]
        speed: f32,
        /// Time to wait at each point before moving
        #[serde(default = "default_wait_time")]
        wait_time: f32,
    },
    /// Collectible item that can be picked up by entities with a specific tag
    Collectible {
        /// Score value when collected
        #[serde(default = "default_score")]
        score_value: u32,
        /// Whether to despawn when collected
        #[serde(default = "default_true")]
        despawn_on_collect: bool,
        /// Tag of entities that can collect this item
        #[serde(default = "default_player_tag")]
        collector_tag: String,
    },
    /// AI that chases entities with a specific tag when in range
    ChaseTagged {
        /// Tag of entities to chase
        #[serde(default = "default_player_tag")]
        target_tag: String,
        /// Distance at which the entity starts chasing
        #[serde(default = "default_detection_range")]
        detection_range: f32,
        /// Movement speed when chasing
        #[serde(default = "default_chase_speed")]
        chase_speed: f32,
        /// Distance at which the entity stops chasing
        #[serde(default = "default_lose_range")]
        lose_interest_range: f32,
    },
}

// Default value functions
fn default_scale() -> (f32, f32) {
    (1.0, 1.0)
}

fn default_texture() -> String {
    "#white".to_string()
}

fn default_color() -> (f32, f32, f32, f32) {
    (1.0, 1.0, 1.0, 1.0)
}

fn default_zoom() -> f32 {
    1.0
}

fn default_viewport() -> (f32, f32) {
    (800.0, 600.0)
}

fn default_fps() -> f32 {
    10.0
}

fn default_true() -> bool {
    true
}

fn default_gravity_scale() -> f32 {
    1.0
}

fn default_friction() -> f32 {
    0.5
}

// Behavior default value functions
fn default_move_speed() -> f32 {
    120.0
}
fn default_jump_impulse() -> f32 {
    420.0
}
fn default_jump_cooldown() -> f32 {
    0.3
}
fn default_follow_distance() -> f32 {
    50.0
}
fn default_follow_speed() -> f32 {
    100.0
}
fn default_patrol_speed() -> f32 {
    80.0
}
fn default_wait_time() -> f32 {
    1.0
}
fn default_score() -> u32 {
    10
}
fn default_detection_range() -> f32 {
    200.0
}
fn default_chase_speed() -> f32 {
    80.0
}
fn default_lose_range() -> f32 {
    300.0
}
fn default_player_tag() -> String {
    "player".to_string()
}

/// Rigid body type for serialization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum RigidBodyTypeData {
    #[default]
    Dynamic,
    Static,
    Kinematic,
}

/// Collider shape for serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ColliderShapeData {
    Box { half_extents: (f32, f32) },
    Circle { radius: f32 },
    CapsuleY { half_height: f32, radius: f32 },
    CapsuleX { half_height: f32, radius: f32 },
}

impl Default for ColliderShapeData {
    fn default() -> Self {
        Self::Box {
            half_extents: (16.0, 16.0),
        }
    }
}

/// Error type for scene loading
#[derive(Debug, thiserror::Error)]
pub enum SceneLoadError {
    #[error("Failed to read scene file: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Failed to parse RON: {0}")]
    RonError(#[from] ron::error::SpannedError),

    #[error("Prefab not found: {0}")]
    PrefabNotFound(String),

    #[error("Invalid texture reference: {0}")]
    InvalidTextureRef(String),

    #[error("Failed to load texture: {0}")]
    TextureLoadError(String),

    #[error("Component error: {0}")]
    ComponentError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
