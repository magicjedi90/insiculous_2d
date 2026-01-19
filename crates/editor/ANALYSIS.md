# Editor Crate Analysis

## Overview (January 2026)

The editor crate provides a visual scene editor for building game worlds, editing entity properties, and managing scene hierarchies. It is built on top of the existing immediate-mode UI system and integrates with the engine's ECS, rendering, and asset systems.

### Summary
- Visual scene editor with dockable panels
- Entity selection and manipulation tools
- Transform gizmos for position, rotation, and scale
- Menu bar with standard editor operations
- Grid snapping and camera controls

### Status
- **Tests**: 64 passing (100% success rate)
- **Code Quality**: 1 minor dead code warning (reserved method)
- **Dependencies**: ui, ecs, input, renderer, engine_core, common

## Architecture

### Module Structure

```
crates/editor/src/
├── lib.rs          # Crate entry point and re-exports
├── context.rs      # EditorContext - main editor state
├── dock.rs         # Dockable panel system
├── gizmo.rs        # Transform manipulation gizmos
├── menu.rs         # Menu bar and dropdown menus
├── selection.rs    # Entity selection management
└── toolbar.rs      # Editor tool selection
```

### Core Components

#### EditorContext (`context.rs`)
Central editor state that extends game context with editor-specific features:
- Selection management
- Transform gizmo state
- Editor camera (offset, zoom)
- Grid settings (visibility, size, snap)
- Play mode toggle
- Layout management

#### Selection (`selection.rs`)
Manages entity selection with support for:
- Single selection
- Multi-selection (add/toggle)
- Primary selection for property editing
- Selection queries and iteration

#### DockArea/DockPanel (`dock.rs`)
Flexible panel layout system:
- Dockable positions: Left, Right, Top, Bottom, Center, Floating
- Resizable panels with minimum size constraints
- Automatic layout calculation
- Panel visibility toggle

#### Toolbar (`toolbar.rs`)
Editor tool selection:
- Select (Q) - Click and select entities
- Move (W) - Translate entities
- Rotate (E) - Rotate entities
- Scale (R) - Scale entities
- Keyboard shortcuts

#### MenuBar/Menu (`menu.rs`)
Standard editor menus:
- File: New, Open, Save, Exit
- Edit: Undo, Redo, Cut, Copy, Paste, Delete, Duplicate
- View: Panel toggles, Reset Layout
- Entity: Create Empty, 2D Objects, Physics bodies

#### Gizmo (`gizmo.rs`)
Visual transform handles:
- Translate gizmo with X/Y axes and center handle
- Rotate gizmo with ring interaction
- Scale gizmo with corner handles
- Delta calculation for transform updates

## Design Patterns

### Immediate-Mode UI Integration
The editor uses the existing ui crate's immediate-mode paradigm:
```rust
// Menu bar renders and returns clicked item
if let Some(action) = editor.menu_bar.render(ui, window_width) {
    handle_menu_action(action);
}

// Toolbar renders and tracks tool changes
if let Some(tool) = editor.toolbar.render(ui) {
    handle_tool_change(tool);
}
```

### Tool-Mode Synchronization
Tool selection automatically updates gizmo mode:
```rust
ctx.set_tool(EditorTool::Move);
// gizmo.mode() is now GizmoMode::Translate
```

### Coordinate Conversion
Editor maintains separate camera from game camera:
```rust
let world_pos = editor.screen_to_world(mouse_pos, window_center);
let screen_pos = editor.world_to_screen(entity_pos, window_center);
```

## Strengths

1. **Modular Design**: Each component (selection, gizmo, dock, etc.) is independent
2. **Test Coverage**: 64 tests covering all major functionality
3. **UI Reuse**: Built on existing immediate-mode UI system
4. **Extensible**: Easy to add new tools, panels, or menu items

## Current Limitations

1. **No actual editor binary** - Framework only, needs integration
2. **Gizmo rendering basic** - Uses simple shapes, not proper handles
3. **No undo/redo system** - Command pattern needed
4. **No asset browser** - Planned for Phase 3
5. **No scene serialization** - Editor state not persisted

## Future Work (See PROJECT_ROADMAP.md)

### Phase 1 Remaining
- [ ] Scene Viewport - Render game world with editor overlay
- [ ] Entity Inspector - Edit component properties
- [ ] Hierarchy Panel - Tree view with drag-and-drop
- [ ] Scene Saving/Loading - Editor state preservation

### Integration Points
- Connect EditorContext with GameContext
- Wire up gizmo transforms to ECS entities
- Implement render_dropdown for inspector components
- Add scene serialization with editor state

## Test Summary

| Module | Tests | Description |
|--------|-------|-------------|
| context | 10 | Editor state, camera, grid, tools |
| dock | 13 | Panel layout and positioning |
| gizmo | 10 | Gizmo modes and interaction |
| menu | 15 | Menu bar and items |
| selection | 12 | Entity selection management |
| toolbar | 7 | Tool selection |

**Total**: 64 tests, 100% passing
