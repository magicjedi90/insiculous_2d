//! Visual scene editor for the insiculous_2d game engine.
//!
//! This crate provides a visual editor for building game worlds, editing
//! entity properties, and managing scene hierarchies. The editor is built
//! on top of the existing immediate-mode UI system and integrates with
//! the engine's ECS, rendering, and asset systems.
//!
//! # Features
//! - Dockable panel system (Scene view, Inspector, Hierarchy, Asset browser)
//! - Entity selection and manipulation (Select, Move, Rotate, Scale)
//! - Visual transform gizmos with grid snapping
//! - Scene saving/loading with editor state preservation
//! - Component property editing with automatic UI generation
//!
//! # Example
//! ```ignore
//! use editor::prelude::*;
//!
//! fn main() {
//!     // Run the editor instead of the game
//!     run_editor(EditorConfig::default()).unwrap();
//! }
//! ```

mod context;
mod dock;
mod editor_input;
mod gizmo;
mod grid;
mod inspector;
mod menu;
mod picking;
mod selection;
mod toolbar;
mod viewport;
mod viewport_input;
pub mod layout;

// Re-export main types
pub use context::EditorContext;
pub use dock::{DockArea, DockPanel, DockPosition, PanelId};
pub use editor_input::{EditorAction, EditorInputMapping, EditorInputState};
pub use gizmo::{Gizmo, GizmoMode};
pub use grid::{GridColors, GridConfig, GridRenderer};
pub use inspector::{inspect_component, InspectorStyle};
pub use menu::{Menu, MenuBar, MenuItem};
pub use picking::{EntityPicker, PickResult, PickableEntity, SelectionRect, AABB};
pub use selection::Selection;
pub use toolbar::{EditorTool, Toolbar};
pub use viewport::SceneViewport;
pub use viewport_input::{ViewportInputConfig, ViewportInputHandler, ViewportInputResult};

/// Prelude module for convenient imports.
pub mod prelude {
    pub use crate::{
        inspect_component, DockArea, DockPanel, DockPosition, EditorAction, EditorContext,
        EditorInputMapping, EditorInputState, EditorTool, EntityPicker, Gizmo, GizmoMode,
        GridRenderer, InspectorStyle, Menu, MenuBar, MenuItem, PanelId, PickResult,
        PickableEntity, SceneViewport, Selection, SelectionRect, Toolbar, ViewportInputConfig,
        ViewportInputHandler, ViewportInputResult, AABB,
    };
}
