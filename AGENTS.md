@training.md

# Insiculous 2D - AI Agent Notes

**Reference:** Use `training.md` for detailed API, patterns, and examples
**This file:** Project status, architecture, and high-level guidance

## Project Status (February 2026)

### Core Systems Complete
- **ECS**: HashMap-based per-type storage, 110 tests, type-safe queries
- **Renderer**: WGPU 28.0.0, instanced sprites, 62 tests
- **Physics**: Rapier2d integration, 28 tests, presets
- **UI**: Immediate-mode, 53 tests, fontdue integration
- **Input**: Event-based, 56 tests, action mapping
- **Audio**: Rodio backend, 3 tests, spatial audio
- **Engine Core**: Game API, managers, 67 tests
- **Editor**: Dockable panels, viewport, inspector, hierarchy, 148 tests

### Key Metrics
- **Total Tests**: 557/557 passing (100% success rate)
- **Test Quality**: 0 TODOs, 155+ meaningful assertions
- **Code Quality**: 30 ignored tests (GPU/window), 0 failures

### Current Priority
**Phase 1: Functional Editor** - See `PROJECT_ROADMAP.md` for full details.
The editor foundation (UI, viewport, inspector, hierarchy) is built. Now wiring it up:
dev mode integration, property editing writeback, play/pause/stop, entity CRUD,
undo/redo, scene save/load, hierarchy drag-and-drop reparenting.

### Technical Debt (Tracked in PROJECT_ROADMAP.md)
See `PROJECT_ROADMAP.md` Technical Debt section for prioritized list

## AI-Friendly Development Principles

This engine is designed to be developed collaboratively with AI agents. Follow these principles:

### Everything Must Be CLI-Testable
- **All logic must be testable without a GPU or window.** The 30 ignored tests are the only exceptions (they require GPU/window). Everything else runs headless.
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

## Architecture

### Manager Pattern + File Refactoring (January 2026) - COMPLETE

**SRP Refactoring**:
- `GameRunner.update_and_render()`: 110+ lines -> 25 lines
- Extracted 7 focused methods (5-25 lines each, pure orchestration)

**Managers Extracted** (5 managers):
- `GameLoopManager` - Frame timing
- `UIManager` - UI lifecycle
- `RenderManager` - Renderer/sprites
- `WindowManager` - Window management
- `SceneManager` - Scene loading and management

**Files Extracted** (5 modules, 674 lines, 12 tests):
- `game_config.rs` (92 lines, 2 tests) - Game configuration
- `contexts.rs` (74 lines) - GameContext, RenderContext, GlyphCacheKey
- `ui_integration.rs` (194 lines) - UI-to-Renderer bridge
- `scene_manager.rs` (153 lines, 5 tests) - Scene management
- `behavior_runner.rs` (optimized) - Performance improvements (-85%)

**game.rs BEFORE**: 862 lines (mixed concerns, poor SRP/DRY)
**game.rs AFTER**: 553 lines (-36%, focused on orchestration)

## Quick Reference

**Commands:**
```bash
cargo check --workspace              # Fast compile check (no tests)
cargo test --workspace               # Run all 557 tests
cargo test -p editor                 # Run editor tests only
cargo test -p ecs                    # Run ECS tests only
cargo clippy --workspace             # Lint check
cargo run --example hello_world      # Run platformer demo
cargo run --example editor_demo      # Run editor demo
```

**Key Files:**
- `AGENTS.md` - This file (high-level guidance)
- `training.md` - Detailed API, patterns, examples
- `PROJECT_ROADMAP.md` - Single source of truth for tasks, priorities, tech debt
- `crates/*/TECH_DEBT.md` - Per-crate technical debt details
- `examples/hello_world.rs` - Working game demonstration
- `examples/editor_demo.rs` - Editor UI demonstration (to be replaced by feature flag)

**Test Status:**
```
$ cargo test --workspace
passed: 557/557 (100%)
ignored: 30 (GPU/window)
failed: 0
```
