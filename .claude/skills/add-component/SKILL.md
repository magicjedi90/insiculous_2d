---
name: add-component
description: Wire a new ECS component through every required integration point (ecs registry, scene RON save/load, editor inspector/undo). Use whenever adding a new component type to the engine, or when a component exists but is missing from the editor or scene files.
---

# Add a New ECS Component

A component is not "done" when the struct compiles. It has up to 5 integration
points. Skipping one causes silent failures (component vanishes on scene save,
invisible in inspector, undo broken). Work through them in order and check off
each one explicitly in your summary.

## Step 1 — Define the component (crates/ecs)

Put it in `crates/ecs/src/` (own file if substantial, e.g. next to
`sprite_components.rs` / `audio_components.rs`). Required derives:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, DeriveComponentMeta)]
pub struct MyComponent { pub value: f32 }
```

Also implement/derive `Default` (the editor's "Add Component" uses it).
Alternative: `define_component!` macro (includes Default with field defaults).
Re-export from `crates/ecs/src/lib.rs` following the existing pattern.

## Step 2 — Global name registry (crates/ecs/src/component_registry.rs)

Add `registry.register::<MyComponent>();` next to the existing registrations
(~line 99, in the global/default registry builder). This enables create-by-name
from JSON, which scene loading and dynamic tooling rely on.

Test: `registry.is_registered("MyComponent")` — copy an existing registry test.

## Step 3 — Scene RON schema (crates/engine_core)

Only if the component should persist in `.scene.ron` files:

1. `scene_data.rs` — add a variant to the `ComponentData` enum (~line 123),
   mirroring existing variants' field style.
2. `scene_loader.rs` — handle the new variant when instantiating entities.
3. `scene_serializer.rs` — add an extraction block in `extract_components()`
   (this is the ONLY World→RON pipeline; do not create another).
4. Add a round-trip test in scene_serializer's test module (copy
   `test_entity_with_sprite` shape: build world → serialize → assert variant).

## Step 4 — Editor visibility (crates/editor/src/stored_component.rs)

**One line** in the `editor_component_registry!` invocation (~line 180). Pick the
section:
- `hidden` — undo-capture only (e.g. GlobalTransform2D, Name)
- `builtin` — inspected, never removable (Transform2D)
- `removable` — normal case; tag with a `ComponentCategory`

This generates StoredComponent/ComponentKind/capture/inspect dispatch. Do NOT
add match arms anywhere else — if you find yourself editing a ComponentKind
match by hand, you're duplicating what the macro generates.

## Step 5 — Editable in inspector (only if fields should be editable, not just visible)

1. `crates/editor/src/commands/set_commands.rs` — add an
   `impl_set_component_command!` invocation (generates `SetMyComponentCommand`).
2. `crates/editor_integration/src/panel_renderer/inspector.rs` — wire the edit
   path through `apply_component_edit()` (the single writeback path; it
   preserves `try_merge_or_push` undo merging via `field_hint`).

## Verify

```bash
cargo test -p ecs && cargo test -p engine_core && cargo test -p editor && cargo test -p editor_integration
cargo test --workspace   # must be 0 failed, 0 ignored
cargo clippy --workspace --all-targets   # 0 warnings
```

In your summary, state which of steps 2–5 you did and which you deliberately
skipped (and why — e.g. "runtime-only component, no scene persistence").
