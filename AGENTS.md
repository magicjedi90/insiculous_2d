@training.md

# Insiculous 2D - AI Agent Notes

**Reference:** Use `training.md` for detailed API, patterns, and examples
**This file:** Project status, architecture, and high-level guidance

## Project Status (July 2026)

### Core Systems Complete
- **ECS**: HashMap-based per-type storage, 192 tests, type-safe queries
- **Renderer**: WGPU 28.0.0, instanced sprites, 74 tests
- **Physics**: Rapier2d integration, 64 tests, presets
- **UI**: Immediate-mode, 80 tests, fontdue integration
- **Input**: Event-based, 62 tests, generic action mapping (`InputMapping<A>`)
- **Audio**: Rodio backend, 21 tests (spatial audio components exist in ecs but have no runtime system yet)
- **Engine Core**: Game API, managers, scene serializer, generic pickups, shared arcade scaffolding (`MenuInput`, `spawn_background`, `default_playfield_grid`, `RENDER_UNIT`), 201 tests
- **Editor**: Dockable panels, viewport, inspector, hierarchy, 255 tests
- **Editor Integration**: `run_game_with_editor()` wrapper + inspector writeback + play/pause/stop + scene save/load, 66 tests

### Key Metrics
- **Total Tests**: 1048/1048 passing (100% success rate), 0 ignored
- **Code Quality**: every doc example compiles and runs (window/GPU-bound ones are compile-only `no_run`); 1 tracked TODO in production code (`scene_loader.rs` — the ARCH-006/GPP-06 dynamic-component gap, deliberate)
- Games (in `../games/`): breakout 41 tests, pong 5, space_invaders 25, snake 31 — all clippy-clean

### Current Priority
**The 20 Games Challenge** drives the roadmap (see `PROJECT_ROADMAP.md`): Pong ☑, Breakout ☑, Space Invaders ☑, and Snake ☑ shipped; **Asteroids (game 5) is next** (no engine gaps — rotation/thrust, screen wrap, and asteroid splitting are game code over existing physics). After the Phase A arcade five: Gap 1 `CameraFollow`, Gap 3 `Tilemap`. Editor: Phase 1 complete; Phase 2 (Ideal Editor UI) in progress (2F Status Bar done, 2G Theme started).

### Technical Debt (live docs — open work only)
- Root `TECH_DEBT.md` — workspace rollup with per-crate open counts; detail in `crates/*/TECH_DEBT.md` and `../games/TECH_DEBT.md`
- `log_archive.md` — resolved/completed history (incl. the closed Jul 2026 Game Programming Patterns audit); when you resolve an item, MOVE it there (never leave ✅/strikethrough entries in the live docs)

## AI-Friendly Development Principles

This engine is designed to be developed collaboratively with AI agents. Follow these principles:

### Everything Must Be CLI-Testable
- **All logic must be testable without a GPU or window.** Every test runs headless — including doc tests. Doc examples that genuinely need a window/GPU/device use ` ```no_run ` (compile-checked, not executed); never ` ```ignore `.
- **`cargo test --workspace`** is the single command to validate the entire engine. It must always pass.
- **`cargo test -p <crate>`** tests individual crates in isolation. Use this for faster iteration on a single system.
- **No manual testing required.** If a feature can't be verified by `cargo test`, it needs a test. AI agents can't click buttons or look at screens.
- **Prefer unit tests over integration tests.** Unit tests are faster and give better error localization. Integration tests are for cross-crate interactions.
- **Test names describe behavior**, not implementation: `test_selection_toggle_adds_and_removes` not `test_toggle_method`.

### Code Must Be Readable by AI
- **Explicit over implicit.** No hidden side effects, magic numbers, or clever tricks. AI agents read code linearly.
- **Small, focused files.** Files over 600 lines should be split. AI context windows are limited.
- **Consistent patterns.** Use the established patterns (Manager pattern, Component pattern, etc.) so AI can predict structure.
- **Strong typing.** Enums over strings, newtypes over primitives. Let the compiler catch errors AI might miss.
- **Doc comments on public APIs.** AI agents use these to understand intent without reading implementation.

### Verification Before Claims
- **Run `cargo test --workspace` before claiming any work is done.**
- **Run `cargo check --workspace` to catch compile errors fast** (faster than full test suite).
- **Check for warnings with `cargo clippy --workspace`** when doing cleanup work.
- **Never claim "tests pass" without actually running them.**

### Workflow for AI Agents
1. **Read `PROJECT_ROADMAP.md`** for current priorities and task breakdown
2. **Read `training.md`** for API patterns and coding guidelines
3. **Read the relevant `TECH_DEBT.md`** in the crate you're working on
4. **Write tests first** when implementing new features
5. **Run `cargo test -p <crate>`** after each change to catch regressions fast
6. **Run `cargo test --workspace`** before considering work complete

## Recurring Themes

### ChaosMode (engine_core)
Project-wide "Normal / Insane / Ridiculous / Insiculous" intensity selector
carried on `GameConfig.chaos_mode` and mirrored on `GameContext.chaos_mode`.
The engine ships *no* gameplay logic for the variants — each game interprets
them per its own mechanics (Pong: Insane doubles ball speed per paddle hit,
Ridiculous starts with 2 balls, Insiculous = both). Helpers: `ChaosMode::ALL`,
`is_insane()`, `is_ridiculous()`, `label()`. Games that let the player pick at
runtime keep their own field as the source of truth; the engine field is for
games that set the mode once at startup via `GameConfig::with_chaos_mode()`.

## Architecture

### Manager Pattern (engine_core)
`GameRunner` is a thin orchestrator (`game.rs`, ~594 lines) over five focused managers:
- `GameLoopManager` - Frame timing (the ONLY frame timer)
- `UIManager` - UI lifecycle and draw-command collection
- `RenderManager` - Renderer/sprite pipeline lifecycle
- `WindowManager` - Window creation and size tracking
- `SceneManager` - Scene loading and stack management

Supporting modules: `game_config.rs`, `contexts.rs` (GameContext/RenderContext), `ui_integration.rs` (UI→renderer bridge), `glyph_texture_cache.rs`, `behavior_runner.rs`. (Refactoring history: `log_archive.md`.)

### Editor Integration
The `editor_integration` crate bridges `engine_core` and `editor` without circular deps:
- `EditorGame<G: Game>` — transparent wrapper that implements `Game`, intercepts all methods to add editor chrome
- `run_game_with_editor(game, config)` — public entry point, wraps game and enforces min window size (1024x720)
- `panel_renderer/` — panel content rendering (scene view, hierarchy, inspector)
- `EditorPlayState` (`Editing`/`Playing`/`Paused`): game logic runs only during Playing; `WorldSnapshot` typed-clone capture on Play, restore on Stop; inspector read-only while Playing

**Dependency graph:**
```
engine_core ──→ ecs, renderer, input, physics, audio, ui
editor ──→ ecs, ui, input, renderer, physics, common      (NO engine_core dep)
editor_integration ──→ editor, engine_core, ecs, ui, input, renderer, common
insiculous_2d (root) ──→ editor_integration (optional, behind "editor" feature)
```

Notes: Escape is NOT a hard-coded exit — it flows to `Game::on_key_pressed()`. `editor_demo.rs` wraps the full PlatformerGame (synced with hello_world.rs). A standalone editor binary exists: `cargo run --bin editor --features editor -- /path/to/project`.

## Quick Reference

**Commands:**
```bash
cargo check --workspace              # Fast compile check (no tests)
cargo test --workspace               # Run all 1048 tests
cargo test -p editor                 # Run editor tests only
cargo test -p editor_integration     # Run editor integration tests
cargo test -p ecs                    # Run ECS tests only
cargo clippy --workspace             # Lint check
cargo run --example hello_world      # Run platformer demo
cargo run --example editor_demo --features editor  # Run editor demo
cargo run --bin editor --features editor -- ../games/pong  # Standalone editor on a project
```

**Key Files:**
- `AGENTS.md` - This file (high-level guidance)
- `training.md` - Detailed API, patterns, examples
- `PROJECT_ROADMAP.md` - LIVE: tasks, priorities, engine gaps
- `TECH_DEBT.md` + `crates/*/TECH_DEBT.md` + `../games/TECH_DEBT.md` - LIVE: open debt only
- `log_archive.md` - Resolved/completed history (move finished items here)
- `examples/hello_world.rs` - Working game demonstration
- `examples/editor_demo.rs` - Editor demo (requires `--features editor`)
- `src/bin/editor.rs` - Standalone editor binary
- `../games/` - Sibling dir: one cargo project per game (pong, breakout, space_invaders)

**Test Status:**
```
$ cargo test --workspace
passed: 1048/1048 (100%)
ignored: 0
failed: 0
```
