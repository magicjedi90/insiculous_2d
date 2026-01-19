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
mod gizmo;
mod menu;
mod selection;
mod toolbar;

// Re-export main types
pub use context::EditorContext;
pub use dock::{DockArea, DockPanel, DockPosition, PanelId};
pub use gizmo::{Gizmo, GizmoMode};
pub use menu::{Menu, MenuBar, MenuItem};
pub use selection::Selection;
pub use toolbar::{EditorTool, Toolbar};

/// Prelude module for convenient imports.
pub mod prelude {
    pub use crate::{
        DockArea, DockPanel, DockPosition, EditorContext, EditorTool, Gizmo,
        GizmoMode, Menu, MenuBar, MenuItem, PanelId, Selection, Toolbar,
    };
}
