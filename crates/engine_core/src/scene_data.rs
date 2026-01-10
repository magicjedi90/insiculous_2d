//! Scene data structures for RON serialization
//!
//! This module defines the data structures used to serialize and deserialize
//! scene files in RON (Rusty Object Notation) format.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Root structure for a scene file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneData {
    /// Scene name
    pub name: String,
    /// Physics settings for this scene
    #[serde(default)]
    pub physics: Option<PhysicsSettings>,
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityData {
    /// Optional name for this entity (for lookup)
    #[serde(default)]
    pub name: Option<String>,
    /// Optional prefab to instantiate
    #[serde(default)]
    pub prefab: Option<String>,
    /// Component overrides (applied on top of prefab)
    #[serde(default)]
    pub overrides: Vec<ComponentData>,
    /// Inline components (if no prefab is used)
    #[serde(default)]
    pub components: Vec<ComponentData>,
}

impl Default for EntityData {
    fn default() -> Self {
        Self {
            name: None,
            prefab: None,
            overrides: Vec::new(),
            components: Vec::new(),
        }
    }
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scene_data_serialization() {
        let scene = SceneData {
            name: "Test Scene".to_string(),
            physics: Some(PhysicsSettings::default()),
            prefabs: HashMap::new(),
            entities: vec![EntityData {
                name: Some("player".to_string()),
                prefab: None,
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
                overrides: vec![ComponentData::Transform2D {
                    position: (500.0, 100.0),
                    rotation: 0.0,
                    scale: (1.0, 1.0),
                }],
                components: Vec::new(),
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
        };

        let ron_str = ron::ser::to_string_pretty(&entity, ron::ser::PrettyConfig::default())
            .expect("Failed to serialize");

        assert!(ron_str.contains("RigidBody"));
        assert!(ron_str.contains("Collider"));
    }
}
