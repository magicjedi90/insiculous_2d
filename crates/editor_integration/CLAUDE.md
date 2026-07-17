# Editor Integration Crate — Agent Context

You are working in editor_integration. This bridges engine_core and editor without circular deps.
**This is where editor features get wired up to the running game.**

## Architecture
```
EditorGame<G: Game>  — transparent wrapper implementing Game trait
├── inner: G         — the actual game
├── editor: EditorContext
├── font_handle, ui state, play state, gizmo drag tracking
└── Intercepts: init(), update(), on_key_pressed()

run_game_with_editor(game, config) → wraps game in EditorGame, calls run_game()
```

## Dependency Graph
```
engine_core ──→ ecs, renderer, input, physics, audio, ui
editor ──→ ecs, ui, input, renderer, physics, common  (NO engine_core)
editor_integration ──→ editor, engine_core, ecs, ui, input, renderer, common
```

## File Map
- `editor_game/` — EditorGame<G> wrapper, split by feature:
  - `mod.rs` — struct + slim `Game` impl (`update()` = ~30 lines of named phases) + `run_game_with_editor`
  - `menu_actions.rs` — menu bar dispatch + shared delete/duplicate helpers
  - `scene_io.rs` — save/load/new scene (load failures surface on status bar)
  - `shortcuts.rs` — keyboard shortcuts + play state transitions
  - `viewport_interaction.rs` — picking, rectangle selection, gizmo drag
- `entity_ops.rs` — Pure entity CRUD (`&mut World` + `&mut Selection`, no UI). Component dispatch lives in `editor::ComponentKind` (registry macro)
- `panel_renderer/` — Panel contents: `mod.rs` (dispatch, scene view, hierarchy), `inspector.rs` (thin shell: registry-generated `editor::edit_all_components()` for editing, `inspect_all_components` read-only during play, add-component popup)
- `constants.rs` — `DEFAULT_SCENE_PATH`, min window size, `MIN_ENTITY_SCALE`, `DUPLICATE_OFFSET`
- `lib.rs` — Public re-exports

## Key Patterns
- **Camera sync (Jul 2026)**: the editor viewport is the single source of truth for the view. `EditorGame::render` overrides `ctx.camera` with `viewport.to_window_render_camera(window_size)` every frame; while Playing, `sync_viewport_from_main_camera` mirrors the game's main-camera entity onto the viewport (editing pan/zoom saved on Play, restored on Stop). Never sync the other direction.
- **Scale tool scales colliders**: physics ignores Transform2D.scale, so the gizmo scale branch also calls `scale_collider` and records one `MacroCommand` (transform+collider) per drag.
- **Asset browser** (`panel_renderer/asset_browser.rs`): scan-on-open + Rescan, lazy thumbnails (≤4 loads/frame), click-to-assign, drag-drop (ghost via ui overlay; viewport drop assigns on sprite hit, spawns on empty space — both undoable).
- `EditorGame::update()` — main orchestration. Editor input → conditional game update (only if Playing) → render panels
- Input routing: Editing/Paused → editor gets input. Playing → game gets input, editor hotkeys still work.
- Inspector writeback: generated per-component by `editor_component_registry!` (editor crate) — `edit_*()` returns `Option<ComponentEdit<T>>` → `editor::apply_component_edit()` writes to world and records undo via `try_merge_or_push` (continuous edits merge by `field_hint`)
- Play/Stop: snapshot world on Play (typed clone via `WorldSnapshot`), restore on Stop
- Save/Load: Ctrl+S / Ctrl+Shift+S / Ctrl+O / Ctrl+N — uses `scene_serializer::world_to_scene_data` for save, `SceneLoader` for load. Hardcoded paths (no file picker yet)
- Status messages: `editor.status_bar.show_message("Saved")` after successful operations
- Minimum window size: 1024x720 enforced for editor usability

## Phase 1 Status
Phase 1A–1H **complete**: entity CRUD, component add/remove, undo/redo, play/pause/stop, scene save/load, theme, status bar.
Currently in Phase 2 (Ideal Editor UI). See `PROJECT_ROADMAP.md`.

## Known Tech Debt
See `TECH_DEBT.md` (all files < 600 lines since June 2026; remaining: no file picker, menu-label string matching)

## Testing
- 76 passing (incl. 1 compile-only doc test), 0 ignored — `cargo test -p editor_integration` (component-dispatch tests moved to the editor crate with the registry)
- `entity_ops` is fully headless-testable (no UI dependency)

## Godot Oracle — When Stuck
Use `WebFetch` to read from `https://github.com/godotengine/godot/blob/master/`

This crate maps to Godot's editor plugin + node integration layer:
- `editor/editor_node.cpp` — how Godot's editor wraps the running scene
- `editor/scene_tree_dock.cpp` — entity CRUD operations (create, delete, duplicate, reparent)
- `editor/plugins/canvas_item_editor_plugin.cpp` — viewport interaction, picking, gizmo wiring
- `editor/editor_inspector.cpp` — how property changes flow back to objects
- `editor/editor_undo_redo_manager.cpp` — command pattern equivalent
