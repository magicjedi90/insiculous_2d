# Editor Crate — Agent Context

You are working in the editor crate. UI panels, tools, inspector, hierarchy — the editor's data and widgets.
This crate has NO dependency on engine_core. It depends on: ecs, ui, input, renderer, physics, common.

## Architecture
```
EditorContext (selection, tool state, play state, camera, theme, status_bar, command_history)
├── Panels: SceneView, Hierarchy, Inspector, AssetBrowser, Console
├── Dock layout: dock.rs (multi-panel docking)
├── Menu / Toolbar / StatusBar (top + bottom chrome)
├── Tools: Select, Move, Rotate, Scale (Q/W/E/R shortcuts)
├── Gizmos: Translate, Rotate, Scale handles
├── Picking: EntityPicker, SelectionRect, screen_to_world()
├── Inspector: Generic serde-based + per-component editors with writeback
├── Undo/Redo: CommandHistory + EditorCommand trait, StoredComponent for restore
├── Theme: EditorTheme with 30+ color tokens (mockup-derived)
└── Play state: EditorPlayState (Editing/Playing/Paused), WorldSnapshot
```

## File Map
### State + chrome
- `context/` — EditorContext struct (selection, tools, state, theme, command_history); tests in `context/tests.rs`
- `lib.rs` — Public re-exports
- `theme.rs` — EditorTheme (color tokens, gizmo/grid/inspector style converters)
- `dock.rs` — Multi-panel docking
- `layout.rs` — Layout helpers
- `menu.rs` — Top menu bar
- `toolbar.rs` — Tool selection toolbar
- `status_bar.rs` — Bottom status bar (22px); `show_message`/`show_error`/`clear_message`
- `play_controls.rs`, `play_state.rs` — Play/Pause/Stop widget + state enum
- `editor_input.rs` — Editor-only input (hotkeys, etc.)
- `editor_preferences.rs` — Persisted editor prefs (camera, zoom, last scene)

### Inspector / components
- `inspector.rs` — Generic `inspect_component()` (read-only, serde-based)
- `editable_inspector.rs` — Editable field widgets (sliders, Vec2, checkboxes, color)
- `field_style.rs` — `FieldId` (widget-ID mapping), `EditableFieldStyle` (layout dims + colors), `EditResult<T>`
- `component_editors.rs` — Per-component editors: `edit_transform2d()`, `edit_sprite()`, etc. Return `Option<ComponentEdit<T>>`; field ranges in `mod ranges`

### Scene + selection
- `selection.rs` — Selection set (primary + multi-select)
- `hierarchy.rs` — Hierarchy panel tree view
- `viewport.rs`, `viewport_input.rs` — Scene viewport with camera pan/zoom
- `picking.rs` — EntityPicker, PickableEntity, SelectionRect, screen_to_world()
- `gizmo.rs` — Transform gizmos (translate, rotate, scale handles)
- `grid.rs` — Background grid rendering

### Persistence + commands
- `commands/` — EditorCommand trait + CommandHistory (`mod.rs`), entity commands, component commands, `impl_set_component_command!` macro for the 5 Set*Commands (`set_commands.rs`); `push_already_executed`, `try_merge_or_push`
- `stored_component.rs` — **Component registry macro (single source of truth). ADD NEW EDITOR-VISIBLE COMPONENTS HERE** — one line in `editor_component_registry!` generates StoredComponent, ComponentKind (add/capture/remove/is_present/display_name/category), capture_all_components, and inspect_all_components
- `world_snapshot.rs` — WorldSnapshot save/restore (used by play/stop)
- Scene save/load file I/O lives in `editor_integration` (via `engine_core::scene_serializer`), not in this crate

## Key Patterns
- Inspector uses `serde_json::to_value()` to extract component fields generically
- Component editors return `Option<ComponentEdit<T>>` (full new value + `field_hint` for undo merging) that the integration crate applies via `apply_component_edit()`
- `EditorPlayState::Editing` → editable, `Playing` → read-only inspector, `Paused` → editable
- Selection: `editor.selection.primary()` returns the main selected EntityId
- Gizmo drag tracking: `gizmo_drag_start` field captures initial transform, then a single `TransformGizmo` command is pushed on release
- Theme is on `EditorContext.theme` (public field); call `theme.gizmo_palette()`, `inspector_style()`, `editable_field_style()`, `grid_colors()` instead of hardcoding colors. Menu/Toolbar/Hierarchy `render()` take `&EditorTheme`

## Testing
- 229 passing (incl. 3 doc tests), 0 ignored — `cargo test -p editor`

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
| Undo/Redo | EditorUndoRedoManager | `editor/editor_undo_redo_manager.cpp` |
| Dock layout | EditorDockManager | `editor/editor_dock_manager.cpp` |

**Remember:** Godot's editor is plugin-based with docks. Adapt *interaction patterns* to our immediate-mode UI.
