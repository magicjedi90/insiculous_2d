# Editor Integration Crate — Agent Context

You are working in editor_integration. This bridges engine_core and editor without circular deps.
**This is where editor features get wired up to the running game.**

## Architecture
```
EditorGame<G: Game>  — transparent wrapper implementing Game trait
├── inner: G         — the actual game
├── editor: EditorContext
├── font_handle, ui state, play state
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
- `editor_game.rs` — EditorGame<G> wrapper, main update loop, input routing, play state
- `panel_renderer.rs` — Renders panel contents (scene view, hierarchy, inspector, asset browser)

## Key Patterns
- `EditorGame::update()` — the main orchestration point. Editor update → conditional game update → render panels
- Input routing: Editing/Paused → editor gets input. Playing → game gets input, editor hotkeys still work.
- Inspector writeback: component editors return edit results → applied via `world.get_mut::<T>()`
- Play/Stop: snapshot world on Play, restore on Stop via `WorldSnapshot`
- Minimum window size: 1024x720 enforced for editor usability

## What Needs Wiring (Current Gaps)
- Viewport click-to-select: `EntityPicker` + `ViewportInputHandler` exist but aren't called in update()
- Rectangle selection: `SelectionRect` exists but no viewport drag integration
- Entity create/delete/duplicate: menu items exist but no handlers

## Testing
- 17 tests, run with `cargo test -p editor_integration`

## Godot Oracle — When Stuck
Use `WebFetch` to read from `https://github.com/godotengine/godot/blob/master/`

This crate maps to Godot's editor plugin + node integration layer:
- `editor/editor_node.cpp` — how Godot's editor wraps the running scene
- `editor/scene_tree_dock.cpp` — entity CRUD operations (create, delete, duplicate, reparent)
- `editor/plugins/canvas_item_editor_plugin.cpp` — viewport interaction, picking, gizmo wiring
- `editor/editor_inspector.cpp` — how property changes flow back to objects
