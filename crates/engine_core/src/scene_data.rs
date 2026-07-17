//! Scene data structures for RON serialization
//!
//! This module defines the data structures used to serialize and deserialize
//! scene files in RON (Rusty Object Notation) format.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

// BehaviorData (+ its Behavior conversions) lives in `behavior_data.rs` for
// file-size reasons; re-exported here so the scene schema stays one import.
pub use crate::behavior_data::BehaviorData;

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
        /// Emissive strength for bloom (0.0 = no glow)
        #[serde(default)]
        emissive: f32,
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
    /// Tilemap component - grid of tile indices drawn from a tileset
    Tilemap {
        /// Tileset texture reference: "#white", "#solid:RRGGBB", or file path
        #[serde(default = "default_texture")]
        tileset: String,
        #[serde(default)]
        width: u32,
        #[serde(default)]
        height: u32,
        #[serde(default = "default_tile_size")]
        tile_size: f32,
        /// Row-major tile values, 0 = empty
        #[serde(default)]
        tiles: Vec<u32>,
        /// Fraction of the tileset per tile, e.g. (0.25, 0.25) for 4x4
        #[serde(default = "default_scale")]
        tile_uv_size: (f32, f32),
        #[serde(default = "default_tilemap_depth")]
        depth: f32,
    },
    /// Screen-space text label (data-driven UI; `@key` text localizes)
    UiLabel {
        #[serde(default)]
        text: String,
        #[serde(default)]
        anchor: ecs::UiAnchor,
        #[serde(default)]
        offset: (f32, f32),
        #[serde(default = "default_ui_font_size")]
        font_size: f32,
        #[serde(default = "default_color")]
        color: (f32, f32, f32, f32),
        #[serde(default = "default_true")]
        visible: bool,
    },
    /// Screen-space colored panel (data-driven UI)
    UiPanel {
        #[serde(default)]
        anchor: ecs::UiAnchor,
        #[serde(default)]
        offset: (f32, f32),
        #[serde(default = "default_ui_panel_size")]
        size: (f32, f32),
        #[serde(default = "default_ui_panel_background")]
        background: (f32, f32, f32, f32),
        #[serde(default = "default_color")]
        border: (f32, f32, f32, f32),
        #[serde(default = "default_ui_border_width")]
        border_width: f32,
        #[serde(default = "default_true")]
        visible: bool,
    },
    /// Screen-space clickable button (data-driven UI; presses surface as
    /// `UiButtonPressed` events carrying `id`)
    UiButton {
        #[serde(default)]
        text: String,
        #[serde(default)]
        id: String,
        #[serde(default)]
        anchor: ecs::UiAnchor,
        #[serde(default)]
        offset: (f32, f32),
        #[serde(default = "default_ui_button_size")]
        size: (f32, f32),
        #[serde(default = "default_true")]
        visible: bool,
    },
    /// Behavior component - defines how an entity responds to input/events
    Behavior(BehaviorData),
    /// Tag component for entity identification (targeted by behaviors)
    EntityTag {
        #[serde(default = "default_player_tag")]
        tag: String,
    },
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

fn default_ui_font_size() -> f32 {
    16.0
}

fn default_ui_panel_size() -> (f32, f32) {
    (200.0, 120.0)
}

fn default_ui_panel_background() -> (f32, f32, f32, f32) {
    (0.1, 0.1, 0.15, 0.85)
}

fn default_ui_border_width() -> f32 {
    1.0
}

fn default_ui_button_size() -> (f32, f32) {
    (120.0, 32.0)
}

fn default_tile_size() -> f32 {
    32.0
}

fn default_tilemap_depth() -> f32 {
    -1.0
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
