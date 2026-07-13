# Engine Core Crate — Agent Context

Core engine: Game trait, run_game(), managers, scene loading/saving, asset management.

## Key Types
- `Game` trait — `init()`, `update()`, `on_key_pressed()` — the public API for games
- `GameConfig` — window title, size, clear color, **`chaos_mode`**
- `run_game(game, config)` — entry point, creates window + event loop
- `GameContext` — passed to Game methods: world, input, assets, ui, physics, delta_time, **chaos_mode**
- `ChaosMode` — cross-game Normal/Insane/Ridiculous/Insiculous theme (engine carries the selection, games define the meaning)
- Managers: `GameLoopManager`, `UIManager`, `RenderManager`, `WindowManager`, `SceneManager`

## File Map
- `game.rs` — Game trait, GameConfig, run_game(), GameRunner orchestration (594 lines)
- `glyph_texture_cache.rs` — GlyphTextureCache: UI glyph bitmap → GPU texture cache (extracted from GameRunner)
- `game_config.rs` — GameConfig struct
- `game_loop_manager.rs` — Frame timing and delta
- `ui_manager.rs` — UI lifecycle and draw commands
- `render_manager.rs` — Renderer lifecycle
- `window_manager.rs` — Window creation
- `scene.rs` — Scene lifecycle / world coordination
- `scene_manager.rs` — Scene loading and entity instantiation
- `scene_loader.rs` — RON → World deserialization; `SceneInstance` retains the prefab table and offers runtime `spawn_prefab(world, assets, name, overrides)` (Prototype pattern, override semantics; failed spawns leave no debris)
- `scene_serializer.rs` — World → SceneData (inverse of scene_loader, used by editor save)
- `scene_data.rs` — SceneData / PrefabData / EntityData structs (schema incl. `ComponentData::EntityTag`, Sprite `emissive`)
- `behavior_data.rs` — `BehaviorData` + the `Behavior`↔`BehaviorData` From impl pair (re-exported via `scene_data`)
- `texture_ref.rs` — scene texture reference resolution (`#white`, `#solid:RRGGBB`, file paths); `TextureResolver` trait is the GPU seam (AssetManager = production impl, tests stub it)
- `assets.rs` — Asset loading (textures, fonts); tracks `handle_to_path` for save
- `behavior_runner.rs` — Entity behavior system
- `lifecycle.rs` — FSM for scene lifecycle
- `timing.rs` — Timer utilities
- `contexts.rs` — GameContext, RenderContext
- `chaos_mode.rs` — `ChaosMode` enum + helpers (`ALL`, `is_insane`, `is_ridiculous`, `label`)
- `pickups.rs` — generic pickup/collectible tracking (`Pickups<K>` keyed by a game-defined kind, `EffectTimer` for timed effects); collection = started-collision events vs a collector set, once per pickup. Used by BOTH Pong (floating power-ups, balls collect) and Breakout (falling drops, paddle collects) — engine owns the mechanism, games own the meaning
- `ui_integration.rs` — UI-to-renderer bridge
- `prelude.rs` — Re-exports for `use engine_core::prelude::*`

## Save/Load Pipeline
- Editor calls `world_to_scene_data(world, name, physics, texture_path_fn)` from `scene_serializer.rs`
- Texture handle → path resolved via `AssetManager.handle_to_path` (populated by `load_texture()`)
- Inverse path: `SceneLoader::load_and_instantiate(path, world, assets)` from `scene_loader.rs`
- Loader attaches a `Name` component for named entities (in addition to `SceneInstance.named_entities`), so names survive an editor load→save round-trip

## Testing
- 186 passing (incl. 8 doc tests, 3 of them compile-only `no_run`), 0 ignored — `cargo test -p engine_core`

## Godot Oracle
- Game loop: `main/main.cpp` — `iteration()` method
- Scene loading: `scene/resources/packed_scene.cpp`
- Asset management: `core/io/resource_loader.cpp`
