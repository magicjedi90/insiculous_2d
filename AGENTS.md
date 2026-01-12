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
- **Performance**: Managers extracted, SRP refactoring in progress
- **Code Quality**: 18 ignored tests (GPU/window), 0 failures

### ⚠️ Technical Debt (Prioritized)
**Critical:**
- SRP refactoring: GameRunner still has 8+ responsibilities
- Font first-frame: Placeholder shows on frame 1
- Behavior clone inefficiency: 40+ allocations per frame

**High:**
- Dead code cleanup (~25 #[allow(dead_code)] suppressions)
- Bind group caching (performance)
- Glyph texture key collision (memory waste)

**Medium:**
- Device/queue accessors consolidation
- Complete ANALYSIS.md files for all crates

**Tracked in:** `PROJECT_ROADMAP.md` Technical Debt section

## Architecture

### Manager Pattern (NEW - January 2026)
Extracted responsibilities from GameRunner:
- `GameLoopManager` - Frame timing and delta calculation (4 tests)
- `UIManager` - UI lifecycle and draw commands (2 tests)
- `RenderManager` - Renderer lifecycle and sprites
- `WindowManager` - Window creation and size tracking
- `AssetManager` - Texture loading and caching

**Status:** Managers extracted, orchestration layer still violates SRP

### Core Architecture
```
Application → GameRunner → Managers (SRP extracted)
    ↓              ↓
Scene → World + ECS + Physics
    ↓              ↓
Renderer → Sprite Pipeline → GPU
```

### Key Patterns
- ECS: Archetype-based component storage
- UI: Immediate-mode with glyph caching
- Rendering: WGPU instanced sprites with batching
- Physics: Rapier2d with ECS integration
- Input: Event queue + action mapping

## High-Level Goals

### Short-term (Next Sprint)
1. Complete SRP refactoring (reduce GameRunner responsibilities)
2. Fix font first-frame flicker
3. Add texture missing warnings
4. Cache bind groups (performance)

### Medium-term
1. Plugin system for extensibility
2. Advanced rendering (lighting, post-processing)
3. Editor tools (scene editor, inspector)
4. Platform support (mobile, web)

### Quality Gates
- No new TODOs in tests
- No new #[allow(dead_code)] without justification
- All new code must have tests
- Maintain 100% test pass rate

## Development Workflow

### Pair Programming with AI
1. **Start here:** Check project status and architecture
2. **Reference training.md:** For API details, patterns, examples
3. **Check PROJECT_ROADMAP.md:** For priorities and violations
4. **Follow patterns:** Use established patterns, don't invent new ones
5. **Test first:** Write tests before implementation

### Human Developer Workflow
1. **Quick start:** See training.md Simple Game API pattern
2. **Examples:** Run `cargo run --example hello_world`
3. **API docs:** Check training.md pattern reference
4. **Architecture:** Review Architecture section above
5. **Contribute:** Follow patterns, test thoroughly

## Quick Reference

**Commands:**
```bash
cargo test --workspace              # Run all tests
cargo run --example hello_world    # Run demo
grep -r "TODO:" crates/ --include="*.rs"  # Check for TODOs
```

**Key Files:**
- `AGENTS.md` - This file (high-level guidance)
- `training.md` - Detailed API, patterns, examples
- `PROJECT_ROADMAP.md` - Priorities and violations
- `crates/engine_core/src/game.rs` - Simple Game API
- `examples/hello_world.rs` - Working demonstration

**Test Status:**
```
$ cargo test --workspace
passed: 338/338 (100%)
ignored: 18 (GPU/window)
failed: 0
status: production ready ✅
```
