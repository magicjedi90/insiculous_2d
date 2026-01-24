# Snap to Grid Design

**Date:** 2026-01-24
**Status:** Approved
**Scope:** Editor scene viewport snap-to-grid functionality + crate restructure

---

## Overview

Add snap-to-grid functionality to the scene viewport editor, allowing precise entity placement by snapping entity edges to grid lines while holding a modifier key.

---

## Core Behavior

### Activation
- Hold `Ctrl` while dragging an entity to enable snapping
- Without `Ctrl`, entities move freely (current behavior)
- Works with translate gizmo and direct entity dragging

### What Snaps
- The **nearest edge** of the entity's bounding box snaps to the nearest grid line
- For a platform at position (100, 50) with size (64, 16):
  - Left edge at x=68, right edge at x=132
  - Bottom edge at y=42, top edge at y=58
- While dragging, the edge closest to a grid line wins

### Snap Threshold
- Edge snaps when within `grid_size / 4` of a grid line (8 pixels for 32px grid)
- Configurable via `SnapSystem.threshold`

### Scope
- Grid-only snapping (no entity-to-entity snapping)

---

## Visual Feedback

### Snap Lines
- When an edge snaps, draw a thin line from the entity edge to the grid line
- Line style: solid, semi-transparent cyan (60% opacity)
- Line extends the full width/height of the entity edge

### Example
```
     Grid line (x=128)
          │
          │◄─── snap line ───►┌──────────┐
          │                   │  Entity  │
          │                   └──────────┘
```

### Multiple Snaps
- Up to 2 snap lines simultaneously (one horizontal, one vertical)
- Prioritizes the edges actively snapping during the drag

---

## Implementation

### New Types (`viewport/snap.rs`)

```rust
pub struct SnapSystem {
    pub enabled: bool,                // Ctrl held?
    pub threshold: f32,               // Snap distance (grid_size / 4)
    pub active_snaps: Vec<SnapLine>,  // Currently showing
}

pub struct SnapLine {
    pub edge: Edge,          // Which entity edge
    pub grid_value: f32,     // Grid line position
    pub axis: Axis,          // Horizontal or Vertical
}

pub enum Edge { Left, Right, Top, Bottom }
pub enum Axis { X, Y }
```

### Core Function

```rust
impl SnapSystem {
    /// Returns position adjustment to snap entity edges to grid
    pub fn calculate_snap(
        &mut self,
        entity_bounds: AABB,
        grid_size: f32,
    ) -> Vec2  // offset to apply
}
```

### Integration Points

| Component | Change |
|-----------|--------|
| `Gizmo` | Call `snap.calculate()` when translating, apply offset |
| `ViewportInput` | Detect Ctrl key, set `snap.enabled` |
| `GridRenderer` | Add `render_snap_lines()` method |
| `EditorContext` | Hold `SnapSystem` instance |

---

## Editor Crate Restructure

Reorganize the editor crate from flat structure to feature-based folders.

### New Structure

```
crates/editor/src/
├── lib.rs                    # Re-exports, EditorContext
├── context.rs                # EditorContext struct
│
├── viewport/
│   ├── mod.rs               # Re-exports
│   ├── scene_viewport.rs    # Camera, coordinate conversion
│   ├── grid.rs              # Grid rendering, LOD
│   ├── snap.rs              # NEW: Snap system
│   └── picking.rs           # Entity picking, selection rect
│
├── tools/
│   ├── mod.rs               # Re-exports
│   ├── gizmo.rs             # Transform gizmos
│   └── selection.rs         # Selection state management
│
├── ui/
│   ├── mod.rs               # Re-exports
│   ├── dock.rs              # Dockable panels
│   ├── menu.rs              # Menu bar
│   └── toolbar.rs           # Toolbar buttons
│
└── input/
    ├── mod.rs               # Re-exports
    ├── editor_input.rs      # Global editor input
    └── viewport_input.rs    # Viewport-specific input
```

### File Moves

| From | To |
|------|-----|
| `viewport.rs` | `viewport/scene_viewport.rs` |
| `grid.rs` | `viewport/grid.rs` |
| `picking.rs` | `viewport/picking.rs` |
| `gizmo.rs` | `tools/gizmo.rs` |
| `selection.rs` | `tools/selection.rs` |
| `dock.rs` | `ui/dock.rs` |
| `menu.rs` | `ui/menu.rs` |
| `toolbar.rs` | `ui/toolbar.rs` |
| `editor_input.rs` | `input/editor_input.rs` |
| `viewport_input.rs` | `input/viewport_input.rs` |

### Re-exports

Keep public API stable via `lib.rs`:

```rust
pub use viewport::{SceneViewport, GridRenderer, SnapSystem, EntityPicker};
pub use tools::{Gizmo, GizmoMode, Selection};
pub use ui::{DockArea, DockPanel, MenuBar, Toolbar};
pub use input::{EditorInput, ViewportInput};
```

---

## Implementation Order

1. **Restructure editor crate** (move files, update imports)
2. **Create `SnapSystem`** with `calculate_snap()`
3. **Integrate with `Gizmo`** translate mode
4. **Add snap line rendering** to `GridRenderer`
5. **Wire up Ctrl detection** in `ViewportInput`
6. **Write tests** for snap calculations

---

## Success Criteria

- [ ] Holding Ctrl while dragging snaps entity edges to grid
- [ ] Snap lines visible when snapping occurs
- [ ] Editor crate organized into `viewport/`, `tools/`, `ui/`, `input/`
- [ ] All existing tests pass after restructure
- [ ] New tests cover snap threshold and edge detection
