//! Shared editor-integration constants.
//!
//! Centralizes values that would otherwise be scattered as magic numbers
//! across the editor wrapper, entity operations, and panel rendering.

use glam::Vec2;

/// Default scene file path used until a file picker exists (Phase 2+).
pub(crate) const DEFAULT_SCENE_PATH: &str = "scenes/scene.ron";

/// Minimum window width for the editor to be usable.
pub(crate) const MIN_EDITOR_WINDOW_WIDTH: u32 = 1024;

/// Minimum window height for the editor to be usable.
pub(crate) const MIN_EDITOR_WINDOW_HEIGHT: u32 = 720;

/// Smallest allowed entity scale when dragging the scale gizmo
/// (prevents zero/negative scale).
pub(crate) const MIN_ENTITY_SCALE: f32 = 0.01;

/// World-space offset applied to duplicated entities so the copy is visible
/// next to the original.
pub(crate) const DUPLICATE_OFFSET: Vec2 = Vec2::new(20.0, -20.0);
