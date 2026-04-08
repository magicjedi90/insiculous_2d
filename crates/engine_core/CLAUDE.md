# Engine Core Crate — Agent Context

Core engine: Game trait, run_game(), managers, scene loading, asset management.

## Key Types
- `Game` trait — `init()`, `update()`, `on_key_pressed()` — the public API for games
- `GameConfig` — window title, size, clear color
- `run_game(game, config)` — entry point, creates window + event loop
- `GameContext` — passed to Game methods: world, input, assets, ui, physics, delta_time
- Managers: `GameLoopManager`, `UIManager`, `RenderManager`, `WindowManager`, `SceneManager`

## File Map
- `game.rs` — Game trait, GameConfig, run_game(), GameRunner orchestration (553 lines)
- `game_loop_manager.rs` — Frame timing and delta
- `ui_manager.rs` — UI lifecycle and draw commands
- `render_manager.rs` — Renderer lifecycle
- `window_manager.rs` — Window creation
- `scene_manager.rs` — Scene loading and entity instantiation
- `assets.rs` — Asset loading (textures, fonts)
- `behavior_runner.rs` — Entity behavior system
- `contexts.rs` — GameContext, RenderContext
- `ui_integration.rs` — UI-to-renderer bridge

## Testing
- 67 tests, run with `cargo test -p engine_core`

## Godot Oracle
- Game loop: `main/main.cpp` — `iteration()` method
- Scene loading: `scene/resources/packed_scene.cpp`
- Asset management: `core/io/resource_loader.cpp`
