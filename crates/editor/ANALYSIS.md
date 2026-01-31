# Editor Crate Analysis

## Overview (January 2026)

The editor crate provides a visual scene editor for building game worlds, editing entity properties, and managing scene hierarchies. It is built on top of the existing immediate-mode UI system and integrates with the engine's ECS, rendering, and asset systems.

### Summary
- Visual scene editor with dockable panels
- Entity selection and manipulation tools
- Transform gizmos for position, rotation, and scale
- Menu bar with standard editor operations
- Grid snapping and camera controls
- Scene viewport with camera pan/zoom and grid overlay
- Read-only component inspector (serde-based)

### Status
- **Tests**: 136 passing (100% success rate)
- **Code Quality**: Clean, minor dead code warnings (reserved methods)
- **Dependencies**: ui, ecs, input, renderer, engine_core, common, physics, audio

## Architecture

### Module Structure

```
crates/editor/src/
├── lib.rs              # Crate entry point and re-exports
├── context.rs          # EditorContext - main editor state
├── dock.rs             # Dockable panel system
├── editor_input.rs     # Editor-specific input mapping
├── gizmo.rs            # Transform manipulation gizmos
├── grid.rs             # Grid overlay rendering
├── inspector.rs        # Component property inspector (read-only)
├── layout.rs           # Editor layout management
├── menu.rs             # Menu bar and dropdown menus
├── picking.rs          # Entity picking and selection rectangle
├── selection.rs        # Entity selection management
├── toolbar.rs          # Editor tool selection
├── viewport.rs         # Scene viewport (camera, coordinate conversion)
└── viewport_input.rs   # Viewport navigation input handling
```

### Core Components

#### EditorContext (`context.rs`)
Central editor state that extends game context with editor-specific features:
- Selection management
- Transform gizmo state
- Editor camera (offset, zoom) via SceneViewport
- Grid settings (visibility, size, snap)
- Play mode toggle
- Layout management
- Viewport input handling

#### SceneViewport (`viewport.rs`)
Manages rendering the game world within the scene view panel:
- Camera position and zoom with smooth interpolation
- World-to-screen and screen-to-world coordinate conversion
- Visible world bounds calculation
- Entity sprite generation for viewport rendering
- Focus on entity/selection

#### ViewportInputHandler (`viewport_input.rs`)
Handles viewport navigation input:
- Pan (middle mouse or Space + drag)
- Zoom (scroll wheel, centered on cursor)
- Selection rectangle (primary drag)
- Focus selection (F key)
- Reset camera (Home key)

#### GridRenderer (`grid.rs`)
Renders configurable grid overlay:
- Primary and subdivision lines with LOD
- X/Y axis lines through origin
- Adjustable grid size and colors

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

#### Inspector (`inspector.rs`)
Read-only component property display:
- Uses serde serialization for field extraction
- Supports Vec2/Vec3/Vec4 formatting
- Displays nested objects

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
let world_pos = editor.screen_to_world(mouse_pos);
let screen_pos = editor.world_to_screen(entity_pos);
```

## Strengths

1. **Modular Design**: Each component (selection, gizmo, dock, etc.) is independent
2. **Test Coverage**: 121 tests covering all major functionality
3. **UI Reuse**: Built on existing immediate-mode UI system
4. **Extensible**: Easy to add new tools, panels, or menu items
5. **Scene Viewport Complete**: Full camera controls, grid rendering, coordinate conversion

## Current Limitations

1. **No actual editor binary** - Framework only, needs integration
2. **Gizmo rendering basic** - Uses simple shapes, not proper handles
3. **No undo/redo system** - Command pattern needed
4. **No asset browser** - Planned for Phase 3
5. **No scene serialization** - Editor state not persisted
6. ~~**Inspector is read-only**~~ - **RESOLVED** - Editable inspector with sliders, checkboxes, color picker

## Phase 1 Status

### Completed ✅
- [x] **Editor UI Framework** - Dockable panels, toolbar, menus (64 tests)
- [x] **Scene Viewport** - Camera pan/zoom, grid overlay, coordinate conversion (45 tests)
- [x] **Entity Inspector** - Editable component properties (27 tests)
  - f32 sliders with configurable ranges
  - Vec2 dual X/Y inputs
  - Bool checkboxes
  - Vec4 color picker with RGBA sliders
  - Component-specific editors for Transform2D, Sprite, RigidBody, Collider, AudioSource

### Remaining
- [ ] **Hierarchy Panel** - Tree view with drag-and-drop reparenting
- [ ] **Scene Saving/Loading** - Editor state preservation
- [ ] **Hierarchy Panel** - Tree view with drag-and-drop
- [ ] **Scene Saving/Loading** - Editor state preservation

## Component Inspector Requirements

### Field Types Needed

| Field Type | UI Widget | Components Using |
|------------|-----------|------------------|
| `f32` | Slider or text input | Transform2D.rotation, Sprite.depth, RigidBody.*_damping |
| `Vec2` | X/Y dual input | Transform2D.position/scale, Sprite.offset, RigidBody.velocity |
| `Vec4` | Color picker | Sprite.color |
| `bool` | Checkbox | RigidBody.can_rotate/ccd_enabled, Collider.is_sensor |
| `enum` | Dropdown | RigidBody.body_type, Collider.shape |
| `u32` | Text input | Sprite.texture_handle, collision groups |
| `[f32; 4]` | 4-value input | Sprite.tex_region |

### Editable Components

1. **Transform2D**
   - position: Vec2
   - rotation: f32 (radians, with degree display)
   - scale: Vec2

2. **Sprite**
   - offset: Vec2
   - rotation: f32
   - scale: Vec2
   - tex_region: [f32; 4] (UV coordinates)
   - color: Vec4 (color picker)
   - depth: f32
   - texture_handle: u32 (readonly, asset reference)

3. **RigidBody**
   - body_type: RigidBodyType enum (Dynamic/Static/Kinematic)
   - velocity: Vec2
   - angular_velocity: f32
   - gravity_scale: f32
   - linear_damping: f32
   - angular_damping: f32
   - can_rotate: bool
   - ccd_enabled: bool

4. **Collider**
   - shape: ColliderShape enum (Box/Circle/Capsule)
   - offset: Vec2
   - is_sensor: bool
   - friction: f32 (0.0-1.0 slider)
   - restitution: f32 (0.0-1.0 slider)
   - collision_groups: u32
   - collision_filter: u32

5. **AudioSource**
   - sound_id: u32 (asset reference)
   - volume: f32 (0.0-1.0 slider)
   - pitch: f32
   - looping: bool
   - play_on_spawn: bool
   - spatial: bool
   - max_distance: f32
   - reference_distance: f32
   - rolloff_factor: f32

## Integration Points

### Gizmo to Entity Transform
The gizmo needs to update entity transforms when manipulated:
```rust
// In update loop
if gizmo.is_active() {
    let delta = gizmo.delta();
    let world_delta = ctx.gizmo_delta_to_world(delta);
    
    // Apply to selected entity's Transform2D
    if let Some(transform) = world.get_mut::<Transform2D>(selected_entity) {
        transform.position += world_delta;
    }
}
```

### Inspector to ECS
The inspector needs mutable access to components:
```rust
// Editable inspector pattern
pub fn edit_component<T: Component + Serialize + DeserializeOwned>(
    ui: &mut UIContext,
    world: &mut World,
    entity: EntityId,
    type_name: &str,
) -> bool { // returns true if modified
    // Get mutable reference, render editable fields
    // Apply changes back to component
}
```

## Test Summary

| Module | Tests | Description |
|--------|-------|-------------|
| context | 18 | Editor state, camera, grid, tools, gizmo integration |
| dock | 13 | Panel layout and positioning |
| gizmo | 10 | Gizmo modes and interaction |
| grid | 14 | Grid rendering and LOD |
| menu | 15 | Menu bar and items |
| picking | 6 | Entity picking and selection rectangle |
| selection | 12 | Entity selection management |
| toolbar | 7 | Tool selection |
| viewport | 20 | Camera, coordinates, visible bounds |
| viewport_input | 11 | Pan, zoom, selection input |
| component_editors | 10 | Component-specific editable inspectors |
| editable_inspector | 5 | Editable field widgets (slider, checkbox, color) |
| inspector | 5 | Component display (read-only) |

**Total**: 136 tests, 100% passing
