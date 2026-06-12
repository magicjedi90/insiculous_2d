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
//! ```
//! use editor::EditorContext;
//! use ecs::World;
//!
//! // EditorContext holds all editor state: selection, tools, play state,
//! // theme, and panels. The `editor_integration` crate wires it to a
//! // running game via `run_game_with_editor()`.
//! let mut editor = EditorContext::new();
//! let mut world = World::new();
//! let entity = world.create_entity();
//!
//! editor.selection.select(entity);
//! assert_eq!(editor.selection.primary(), Some(entity));
//! assert!(editor.is_editing()); // starts in Editing play state
//! ```

mod collider_overlay;
pub mod commands;
mod component_editors;
mod context;
mod dock;
mod editable_inspector;
mod field_style;
mod editor_input;
mod gizmo;
mod grid;
mod hierarchy;
mod inspector;
mod menu;
mod picking;
mod play_controls;
mod play_state;
mod selection;
pub mod status_bar;
pub mod stored_component;
pub mod theme;
mod toolbar;
mod viewport;
mod viewport_input;
pub mod editor_preferences;
pub mod layout;
pub mod world_snapshot;

// Re-export main types
pub use collider_overlay::{
    collider_outline_segments, render_collider_overlay, ColliderOverlayColors,
};
pub use commands::{CommandHistory, EditorCommand};
pub use component_editors::{
    edit_audio_source, edit_collider, edit_rigid_body, edit_sprite, edit_transform2d,
    ComponentEdit,
};
pub use context::EditorContext;
pub use editor_preferences::EditorPreferences;
pub use dock::{DockArea, DockPanel, DockPosition, PanelId};
pub use editable_inspector::{
    component_header, display_u32, edit_bool, edit_color, edit_f32, edit_normalized_f32, edit_vec2,
    EditableFieldStyle, EditableInspector, EditResult, FieldId,
};
pub use editor_input::{EditorAction, EditorInputMapping, EditorInputState};
pub use gizmo::{Gizmo, GizmoMode, GizmoPalette};
pub use hierarchy::HierarchyPanel;
pub use grid::{GridColors, GridConfig, GridRenderer};
pub use inspector::{inspect_component, InspectorStyle};
pub use menu::{Menu, MenuBar, MenuItem};
pub use picking::{EntityPicker, PickResult, PickableEntity, SelectionRect, AABB};
pub use play_controls::{PlayControlAction, PlayControls};
pub use play_state::EditorPlayState;
pub use selection::Selection;
pub use status_bar::{StatusBar, StatusBarStats, STATUS_BAR_HEIGHT};
pub use stored_component::{
    available_components, capture_all_components, categorized_components,
    inspect_all_components, restore_components, ComponentCategory, ComponentKind,
    StoredComponent,
};
pub use theme::EditorTheme;
pub use toolbar::{EditorTool, Toolbar};
pub use viewport::SceneViewport;
pub use viewport_input::{ViewportInputConfig, ViewportInputHandler, ViewportInputResult};

/// Prelude module for convenient imports.
pub mod prelude {
    pub use crate::{
        collider_outline_segments, render_collider_overlay, ColliderOverlayColors,
        available_components, capture_all_components, categorized_components,
        inspect_all_components, restore_components, CommandHistory, ComponentCategory,
        ComponentEdit, ComponentKind, EditorCommand, StoredComponent,
        component_header, display_u32, edit_audio_source, edit_bool, edit_collider, edit_color,
        edit_f32, edit_normalized_f32, edit_rigid_body, edit_sprite, edit_transform2d, edit_vec2,
        inspect_component, DockArea, DockPanel,
        DockPosition, EditorAction, EditorContext, EditorInputMapping, EditorInputState,
        EditorPlayState, EditorPreferences, EditorTool, EditableFieldStyle, EditableInspector,
        EditorTheme, EditResult, EntityPicker, FieldId, Gizmo, GizmoMode, GridRenderer,
        HierarchyPanel, InspectorStyle, Menu, MenuBar, MenuItem, PanelId, PickResult,
        PickableEntity, StatusBar, StatusBarStats, STATUS_BAR_HEIGHT,
        PlayControlAction, PlayControls, SceneViewport, Selection,
        SelectionRect, Toolbar, ViewportInputConfig,
        ViewportInputHandler, ViewportInputResult, AABB,
    };
}
