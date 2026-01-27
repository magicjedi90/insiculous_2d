@training.md

# Insiculous 2D - AI Agent Notes

**Reference:** Use `training.md` for detailed API, patterns, and examples
**This file:** Project status, architecture, and high-level guidance

## Project Status (January 2026)

### ✅ Core Systems Complete
- **ECS**: Archetype-based, 84 tests, type-safe queries
- **Renderer**: WGPU 28.0.0, instanced sprites, 62 tests  
- **Physics**: Rapier2d integration, 24 tests, presets
- **UI**: Immediate-mode, 42 tests, fontdue integration
- **Input**: Event-based, 56 tests, action mapping
- **Audio**: Rodio backend, 8 tests, spatial audio
- **Engine Core**: Game API, managers, 60 tests

### 🎯 Key Metrics
- **Total Tests**: 356 (338 passing, 100% success rate)
- **Test Quality**: 0 TODOs, 155+ meaningful assertions
- **Performance**: **COMPLETED** - Managers extracted, game.rs SRP refactoring, Behavior optimization (-85% allocations) ✅
- **Code Quality**: 18 ignored tests (GPU/window), 0 failures

### ⚠️ Technical Debt (Tracked in PROJECT_ROADMAP.md)
See `PROJECT_ROADMAP.md` Technical Debt section for prioritized list

## Architecture

### Manager Pattern + File Refactoring (January 2026) ✅ COMPLETE

**SRP Refactoring**:
- `GameRunner.update_and_render()`: 110+ lines → 25 lines
- Extracted 7 focused methods (5-25 lines each, pure orchestration)
- **STATUS**: game.rs SRP refactoring COMPLETE (Phases 1-3)

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

## Development Workflow

### Pair Programming with AI
1. **Start here:** Check PROJECT_ROADMAP.md for current priorities
2. **Reference training.md:** For API details, patterns, examples
3. **Follow patterns:** Use established patterns, don't invent new ones
4. **Test first:** Write tests before implementation

### Human Developer Workflow
1. **Quick start:** See training.md Simple Game API pattern
2. **Examples:** Run `cargo run --example hello_world`
3. **API docs:** Check training.md pattern reference
4. **Roadmap:** See PROJECT_ROADMAP.md for priorities and tech debt

## Quick Reference

**Commands:**
```bash
cargo test --workspace              # Run all tests
cargo run --example hello_world    # Run demo
```

**Key Files:**
- `AGENTS.md` - This file (high-level guidance)
- `training.md` - Detailed API, patterns, examples
- `PROJECT_ROADMAP.md` - Single source of truth for tasks, tech debt, priorities
- `examples/hello_world.rs` - Working demonstration

**Test Status:**
```
$ cargo test --workspace
passed: 338/338 (100%)
ignored: 18 (GPU/window)
failed: 0
status: production ready, performant ✅
```
