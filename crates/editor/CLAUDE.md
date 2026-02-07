# Editor Crate — Agent Context

You are working in the editor crate. UI panels, tools, inspector, hierarchy — the editor's data and widgets.
This crate has NO dependency on engine_core. It depends on: ecs, ui, input, renderer, physics, common.

## Architecture
```
EditorContext (selection, tool state, play state, camera)
├── Panels: SceneView, Hierarchy, Inspector, AssetBrowser, Console
├── Tools: Select, Move, Rotate, Scale (Q/W/E/R shortcuts)
├── Gizmos: Translate, Rotate, Scale handles
├── Picking: EntityPicker, SelectionRect, screen_to_world()
├── Inspector: Generic serde-based + per-component editors with writeback
└── Play state: EditorPlayState (Editing/Playing/Paused), WorldSnapshot
```

## File Map
- `context.rs` — EditorContext struct (selection, tools, state)
- `inspector.rs` — Generic `inspect_component()` (read-only, serde-based)
- `editable_inspector.rs` — Editable field widgets (sliders, Vec2, checkboxes, color)
- `component_editors.rs` — Per-component editors: `edit_transform2d()`, `edit_sprite()`, etc.
- `picking.rs` — EntityPicker, PickableEntity, SelectionRect, screen_to_world()
- `gizmo.rs` — Transform gizmos (translate, rotate, scale handles)
- `hierarchy.rs` — Hierarchy panel tree view
- `viewport.rs` — Scene viewport with camera pan/zoom
- `play_controls.rs` — Play/Pause/Stop widget
- `play_state.rs` — EditorPlayState enum
- `world_snapshot.rs` — WorldSnapshot save/restore

## Key Patterns
- Inspector uses `serde_json::to_value()` to extract component fields generically
- Component editors return result types (e.g., `TransformEditResult`) that the integration crate applies
- `EditorPlayState::Editing` → editable, `Playing` → read-only inspector, `Paused` → editable
- Selection: `editor.selection.primary()` returns the main selected EntityId

## Testing
- 162 tests, run with `cargo test -p editor`

## Godot Oracle — When Stuck
Use `WebFetch` to read from `https://github.com/godotengine/godot/blob/master/`

| Our Concept | Godot Equivalent | File |
|-------------|-----------------|------|
| EditorContext | EditorNode | `editor/editor_node.cpp` |
| Inspector | EditorInspector | `editor/editor_inspector.cpp` |
| Component editors | EditorProperties | `editor/editor_properties.cpp` — `_property_changed` |
| Picking / selection | Canvas item editor | `editor/plugins/canvas_item_editor_plugin.cpp` — `_gui_input_viewport` |
| Hierarchy panel | SceneTreeDock | `editor/scene_tree_dock.cpp` — `_tool_selected` |
| Gizmos | CanvasItemEditor gizmos | `editor/plugins/canvas_item_editor_plugin.cpp` — search `gizmo` |
| Play/Pause/Stop | EditorRun | `editor/editor_run.cpp`, `editor/editor_node.cpp` — `_run_native` |

**Remember:** Godot's editor is plugin-based with docks. Adapt *interaction patterns* to our immediate-mode UI.
