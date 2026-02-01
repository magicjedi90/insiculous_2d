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

mod component_editors;
mod context;
mod dock;
mod editable_inspector;
mod editor_input;
pub mod file_operations;
mod gizmo;
mod grid;
mod hierarchy;
mod inspector;
mod menu;
mod picking;
mod selection;
mod toolbar;
mod viewport;
mod viewport_input;
pub mod layout;

// Re-export main types
pub use component_editors::{
    edit_audio_source, edit_collider, edit_rigid_body, edit_sprite, edit_transform2d,
    AudioSourceEditResult, ColliderEditResult, RigidBodyEditResult, SpriteEditResult,
    TransformEditResult,
};
pub use context::EditorContext;
pub use dock::{DockArea, DockPanel, DockPosition, PanelId};
pub use editable_inspector::{
    component_header, display_u32, edit_bool, edit_color, edit_f32, edit_normalized_f32, edit_vec2,
    EditableFieldStyle, EditableInspector, EditResult, FieldId,
};
pub use editor_input::{EditorAction, EditorInputMapping, EditorInputState};
pub use file_operations::{
    load_scene, new_scene, save_scene, save_scene_as, FileOperationError,
};
pub use gizmo::{Gizmo, GizmoMode};
pub use hierarchy::HierarchyPanel;
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
        component_header, display_u32, edit_audio_source, edit_bool, edit_collider, edit_color,
        edit_f32, edit_normalized_f32, edit_rigid_body, edit_sprite, edit_transform2d, edit_vec2,
        inspect_component, AudioSourceEditResult, ColliderEditResult, DockArea, DockPanel,
        DockPosition, EditorAction, EditorContext, EditorInputMapping, EditorInputState, EditorTool,
        EditableFieldStyle, EditableInspector, EditResult, EntityPicker, FieldId, Gizmo, GizmoMode,
        GridRenderer, HierarchyPanel, InspectorStyle, Menu, MenuBar, MenuItem, PanelId, PickResult,
        PickableEntity, RigidBodyEditResult, SceneViewport, Selection, SelectionRect,
        SpriteEditResult, Toolbar, TransformEditResult, ViewportInputConfig, ViewportInputHandler,
        ViewportInputResult, AABB,
    };
}
