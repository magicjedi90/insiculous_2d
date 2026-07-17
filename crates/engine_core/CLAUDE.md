# Engine Core Crate — Agent Context

Core engine: Game trait, run_game(), managers, scene loading/saving, asset management.

## Key Types
- `Game` trait — `init()`, `update()`, `on_key_pressed()` — the public API for games
- `GameConfig` — window title, size, clear color, **`chaos_mode`**
- `run_game(game, config)` — entry point, creates window + event loop
- `GameContext` — passed to Game methods: world, input, **players** (per-player
  `InputSettings`: `ctx.players.is_active(PlayerId::P1, GameAction::Action1, ctx.input)`,
  `move_x/move_y`), assets, ui, physics, delta_time, **chaos_mode**, **time_scale**
  (read-write; scales engine-side particle stepping only — set 0.0 while paused),
  **exit_requested** (write true → clean engine shutdown, same path as window close)
- `ChaosMode` — cross-game Normal/Insane/Ridiculous/Insiculous theme (engine carries the selection, games define the meaning)
- Managers: `GameLoopManager`, `UIManager`, `RenderManager`, `WindowManager`, `SceneManager`

## File Map
- `game.rs` — Game trait, run_game(), GameRunner orchestration (~530 lines; the render
  tail lives in the child module `game/render.rs` — new render passes go in their own
  module like `tilemap_render.rs`)
- `game/render.rs` — GameRunner's frame-render tail (`render_frame`, batch-ref sorting,
  particle append); child module of `game` so no field visibility changes were needed
- `gamepad_backend.rs` — gilrs hardware poll (`GamepadBackend::new_or_disabled()`,
  `pump()` drained right before `process_queued_events()`); pure translation fns
  (button/axis tables, 0.15 dead-zone rescale, hat-switch dpad synthesis on ±0.5
  crossings). gilrs stick +Y = up; needs `libudev-dev` on Linux at build time
- `input_settings_io.rs` — JSON load/save for player input bindings (versioned
  Vec-of-entries DTO; missing file → defaults written for hand-editing; corrupt/wrong
  version → warn + defaults, never panics). Wired to `GameConfig::input_settings_path`
  (load at startup, save on CloseRequested)
- `glyph_texture_cache.rs` — GlyphTextureCache: UI glyph bitmap → GPU texture cache (extracted from GameRunner)
- `game_config.rs` — GameConfig struct (incl. `input_settings_path`)
- `game_loop_manager.rs` — Frame timing and delta
- `ui_manager.rs` — UI lifecycle and draw commands
- `render_manager.rs` — Renderer lifecycle; `sync_main_camera(world)` copies the main-camera entity's Transform2D position onto the render camera each frame (position only; no-op without a `Camera { is_main_camera: true }` entity)
- `tilemap_render.rs` — expands `Tilemap` + `Transform2D` entities into the game sprite batcher (called at the top of the default `Game::render`; one batch per tileset)
- `window_manager.rs` — Window creation
- `scene.rs` — Scene lifecycle / world coordination
- `scene_manager.rs` — Scene loading and entity instantiation
- `scene_loader.rs` — RON → World deserialization; `SceneInstance` retains the prefab table and offers runtime `spawn_prefab(world, assets, name, overrides)` (Prototype pattern, override semantics; failed spawns leave no debris)
- `scene_serializer.rs` — World → SceneData (inverse of scene_loader, used by editor save)
- `scene_data.rs` — SceneData / PrefabData / EntityData structs (schema incl. `ComponentData::EntityTag`, Sprite `emissive`)
- `behavior_data.rs` — `BehaviorData` + the `Behavior`↔`BehaviorData` From impl pair (re-exported via `scene_data`)
- `texture_ref.rs` — scene texture reference resolution (`#white`, `#solid:RRGGBB`, file paths); `TextureResolver` trait is the GPU seam (AssetManager = production impl, tests stub it)
- `assets.rs` — Asset loading (textures, fonts); tracks `handle_to_path` for save; `game_root_from()` + the `game_root!()` macro (asset/save anchoring — macro so the game crate's manifest dir is baked in)
- `behavior_runner.rs` — Entity behavior system
- `lifecycle.rs` — FSM for scene lifecycle
- `timing.rs` — Timer utilities
- `contexts.rs` — GameContext, RenderContext
- `chaos_mode.rs` — `ChaosMode` enum + helpers (`ALL`, `is_insane`, `is_ridiculous`, `label`)
- `chaos_theme.rs` — `ChaosTheme` per-mode presentation tokens (bg/structure/accent/grid colors, banner, particle mult); engine owns structure + default palette, games override via struct-update syntax
- `pause.rs` — `PauseMenu`/`PauseAction`: shared pause mechanism (Menu/Esc/Start
  toggles, Resume/Restart/Quit-to-Title/Exit-Game items; games map actions onto their
  own start_game/reset_to_title/`ctx.exit_requested` and skip their whole gameplay
  update while active;
  `time_scale()` feeds `ctx.time_scale` so engine particles freeze too). Takes
  `&InputSettings + &InputHandler` (NOT GameContext) so it's headless-testable
- `menu_panel.rs` — `MenuPanel`/`MenuStyle`: shared menu window chrome (opaque
  themed panel, border, accent separator + corner ticks, ▶-cursor highlight
  rows, hint footer, input-blocking overlay variant). Flair is rect-based;
  the ▶ cursor is verified in the games' shared font.ttf
- `menu_input.rs` — `MenuInput` shared menu-screen input (W/S+arrows up/down, Space/Enter
  confirm, Esc back — plus EVERY connected gamepad: dpad/left-stick edge up/down, A/Start
  confirm, B back) + wraparound `navigate`; used by every game's title/select screens
- `spawn_helpers.rs` — shared entity recipes (`spawn_background` full-window backdrop); `RENDER_UNIT = 80.0` (pixels per world unit) lives at the crate root and is used by the render path in `game.rs`
- `pickups.rs` — generic pickup/collectible tracking (`Pickups<K>` keyed by a game-defined kind, `EffectTimer` for timed effects); collection = started-collision events vs a collector set, once per pickup. Used by BOTH Pong (floating power-ups, balls collect) and Breakout (falling drops, paddle collects) — engine owns the mechanism, games own the meaning
- `ui_integration.rs` — UI-to-renderer bridge. **Camera-relative**: UI sprites are positioned/scaled against the render camera so UI stays at fixed screen pixels when the camera moves/zooms (camera-follow games, editor). Emits SDF shapes: rounded rects, single-sprite borders, true circles, and `DrawCommand::Image` textured quads
- `prelude.rs` — Re-exports for `use engine_core::prelude::*`

## Save/Load Pipeline
- Editor calls `world_to_scene_data(world, name, physics, texture_path_fn)` from `scene_serializer.rs`
- Texture handle → path resolved via `AssetManager.handle_to_path` (populated by `load_texture()`)
- Inverse path: `SceneLoader::load_and_instantiate(path, world, assets)` from `scene_loader.rs`
- Loader attaches a `Name` component for named entities (in addition to `SceneInstance.named_entities`), so names survive an editor load→save round-trip

## Testing
- 245 passing (incl. 10 doc tests, 4 of them compile-only `no_run`), 0 ignored — `cargo test -p engine_core`

## Godot Oracle
- Game loop: `main/main.cpp` — `iteration()` method
- Scene loading: `scene/resources/packed_scene.cpp`
- Asset management: `core/io/resource_loader.cpp`
