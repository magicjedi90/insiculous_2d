# Architecture Review: Unified Component Registry

**Date:** 2026-01-26
**Status:** Phase 4 Complete (All Phases Done)
**Scope:** Extensibility improvements for components, scene serialization, and editor inspection

---

## Executive Summary

This document captures findings from an architecture review focused on extensibility and maintenance burden. The review used animation system implementation as a lens to identify architectural gaps.

**Key Finding:** Three systems (scene serialization, editor inspection, behaviors) all use hardcoded enums requiring multi-file changes for extensions. A unified component registry would reduce this to single-file definitions.

---

## Bugs Found

### 1. Animation Frames Not Rendered

**Location:** `crates/ecs/src/sprite_system.rs:68`

**Problem:** `SpriteRenderSystem.convert_sprite()` accepts an `_animation` parameter but never uses it. Animation frame selection is tracked correctly in `SpriteAnimation`, but the current frame's texture region is never applied to the rendered sprite.

```rust
// Current (broken) - note underscore prefix indicating unused
fn convert_sprite(&self, ..., _animation: Option<&SpriteAnimation>) -> RendererSprite {
    // _animation.current_frame_region() is never called
}
```

**Fix:** Apply the current animation frame's texture region during sprite conversion.

**Priority:** High - core functionality broken

### 2. UI Slider/Button Issues

**Location:** `crates/ui/src/context.rs` (slider), calling code patterns

**Problem:** Likely a value update pattern issue in the calling code, not the UI system itself. The slider returns a new value each frame that must be stored back:

```rust
// BUG: Value returned but never stored
let value = ui.slider("id", current_value, bounds);
// current_value never updated - slider appears "stuck"

// CORRECT:
current_value = ui.slider("id", current_value, bounds);
```

**Additional Risk:** If a widget is hidden mid-drag (conditional rendering), `active_widget` state persists until the widget is rendered again.

**Priority:** Medium - investigate calling code in hello_world.rs

---

## Extensibility Concerns

### Current State: Multi-File Changes Required

Adding a new component (e.g., `AnimationClip`) currently requires changes in:

| File | Change Required |
|------|-----------------|
| `ecs/src/*.rs` | Define struct with derives |
| `engine_core/src/scene_data.rs` | Add `ComponentData` enum variant |
| `engine_core/src/scene_loader.rs` | Add `component_type_name()` match arm |
| `engine_core/src/scene_loader.rs` | Add `add_component_to_entity()` match arm |
| `examples/editor_demo.rs` | Add hardcoded inspector UI |

**Total: 4-5 files, ~50-100 lines of boilerplate per component**

### Systems Affected

1. **Scene Serialization** - Fixed `ComponentData` enum, manual conversion logic
2. **Editor Inspection** - No reflection, each component hardcoded with `if let Some()`
3. **Behavior System** - Same enum pattern, 4 files for new behavior types

### Root Cause

No component metadata system. Each system independently hardcodes knowledge about component types instead of discovering them from a central registry.

---

## Proposed Solution: Unified Component Registry

### Design Philosophy

- **"Simplicity strives"** - Minimize boilerplate, maximize discoverability
- **Define once, works everywhere** - Single source of truth for component definitions
- **Option C approach** - Declarative macro (`macro_rules!`), not proc macro
- **Data/logic separation** - Components are pure data; behaviors stay separate

### What the Registry Provides

```rust
// Single definition in one file
register_component! {
    /// Player animation clip data
    AnimationClip {
        name: String,
        fps: f32 = 10.0,           // default value for scene files
        frames: Vec<[f32; 4]>,
        looping: bool = true,
    }
}
```

The macro generates:
1. **Struct** with `#[derive(Debug, Clone, Serialize, Deserialize)]`
2. **Default impl** using specified default values
3. **ComponentMeta trait** providing type name, field names, field types

### How Each System Uses the Registry

| System | Current | With Registry |
|--------|---------|---------------|
| **ECS** | Manual struct definition | Macro generates struct |
| **Scene Loader** | Match on enum variant | Lookup by type name string |
| **Editor Inspector** | Hardcoded per-component UI | Loop through field metadata |
| **Serialization** | Manual RON mapping | Automatic from struct fields |

### File Structure

```
crates/ecs/src/
  component_registry.rs    <- NEW: macro definition + registry
  components/
    mod.rs                 <- re-exports all components
    transform.rs           <- register_component! { Transform2D { ... } }
    sprite.rs              <- register_component! { Sprite { ... } }
    animation.rs           <- register_component! { AnimationClip { ... } }
```

### What This Doesn't Cover (Intentionally)

- **Behaviors** - They have logic, not just data. Keep as separate enum system.
- **Custom editor widgets** - Field type determines widget (f32 → slider, bool → checkbox). Custom widgets need annotations (future enhancement).
- **Validation rules** - Min/max values, required fields (future enhancement).
- **Full reflection** - Overkill for current needs. Can evolve to proc macro later if component count grows significantly.

---

## Implementation Priority

### Phase 1: Fix Broken Functionality ✅ COMPLETE
1. ~~**Animation rendering bug** - Apply current frame in SpriteRenderSystem~~
2. ~~**UI investigation** - Check hello_world.rs slider/button value patterns~~

### Phase 2: Component Registry Foundation ✅ COMPLETE
3. ~~**Create registry module** - `component_registry.rs` with `define_component!` macro~~
4. ~~**Migrate existing components** - Transform2D, Sprite, Camera, SpriteAnimation~~
5. ~~**Global registry** - Built-in components registered at startup~~

### Phase 3: Editor Integration ✅ COMPLETE
6. ~~**Generic component inspector** - Serde-based field extraction, auto-displays all components~~
7. ~~**Button text rendering fix** - Buttons now render actual font glyphs~~
8. ~~**Gizmo movement fix** - Correct coordinate system, entities move with mouse~~

### Phase 4: Polish ✅ COMPLETE
9. ~~**Documentation** - Update training.md with new patterns~~
10. ~~**Mark deprecated code** - application.rs marked deprecated with migration guide~~

---

## Design Patterns Alignment

This design aligns with established game programming patterns:

- **Component Pattern** - Components remain pure data containers
- **Type Object Pattern** - Component types defined as data (registry), not hardcoded classes
- **Data Locality** - Unified definitions enable consistent memory layout
- **Data-Driven Design** - Scene files stay pure configuration

---

## Risks and Mitigations

| Risk | Mitigation |
|------|------------|
| Macro complexity | Start with `macro_rules!` (debuggable), evolve to proc macro only if needed |
| Migration effort | Migrate components incrementally, keep old system working during transition |
| Editor field types | Start with simple display (labels), add editable widgets incrementally |
| Performance | Registry lookup is initialization-time only, not per-frame |

---

## Success Criteria

- [x] Adding a new component requires changes to only 1 file (derive macro for ComponentMeta)
- [x] Editor displays any registered component without code changes (serde-based inspector)
- [x] Scene files load components via registry lookup (Dynamic variant + factory)
- [x] Animation system renders frames correctly (fixed in commit 7c98289, test added)
- [x] UI widgets in hello_world.rs work as expected (button text fixed)
- [x] Gizmo moves entities correctly (coordinate fix applied)

---

## Appendix: Current Architecture Strengths

The review also confirmed these aspects are well-designed:

- **Crate separation** - Clean dependency graph, leaf crates have zero internal deps
- **Manager pattern** - Recent refactoring reduced game.rs from 862 to 552 lines
- **Test coverage** - 422 tests, 100% passing (18 GPU/window ignored)
- **ECS core** - Archetype-based storage, type-safe queries work well
- **Renderer** - Sprite instancing, texture regions, batching all production-ready
- **Behavior system** - Data-driven, serializable, good for current 7 behavior types

No changes needed to these systems.
