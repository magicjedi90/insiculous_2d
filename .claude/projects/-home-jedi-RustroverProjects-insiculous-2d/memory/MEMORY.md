/# Insiculous 2D - Key Learnings

## Project Principles
- **AI-friendly development** is a core value. Everything must be CLI-testable (`cargo test --workspace`).
- **Editor-first** - The user prioritizes editor features over scripting, animation, or platform support.
- **Unity/Godot parity** is the target for editor feature set.

## Current State (February 2026)
- 560 tests, 100% pass rate, 30 ignored (GPU/window)
- Editor foundation built but **inspector is read-only** - editable widgets exist but aren't wired to ECS writeback
- Editor runs as `examples/editor_demo.rs` - needs to be a feature flag on engine_core instead
- Only translate gizmo writes back; rotate/scale gizmos are not functional yet

## Key Files
- `PROJECT_ROADMAP.md` - Single source of truth for priorities and tech debt
- `AGENTS.md` - AI agent workflow and principles
- `training.md` - API patterns and coding guidelines
- `crates/editor/` - Editor crate (148 tests, 6854 lines across 14 source files)

## Architecture Notes
- Editor uses immediate-mode UI (same as game UI)
- Component editors return `*EditResult` structs with change flags and new values
- `EditorContext` is the central editor state (context.rs, 628 lines)
- `ComponentRegistry` enables dynamic component creation by name

## Gotchas
- `World::default()` uses Legacy storage, not Archetype (known tech debt PATTERN-002)
- `EngineApplication` is deprecated but still exists alongside `GameRunner` (tech debt)
- Glyph texture cache includes color in key (wastes memory)
