# Task Queue - Insiculous 2D

**Instructions for agents:** Claim a task by creating `current_tasks/TASK-XXX.lock` with your agent ID and timestamp. Work the task, push, then remove the lock and move the task to PROGRESS.md.

**Priority order:** Work top-to-bottom. Higher tasks are higher priority.

---

## Phase 1C½: Viewport Entity Picking (HIGH PRIORITY)

### TASK-001: Wire up viewport click-to-select
**Crate:** editor_integration
**Effort:** M
**Spec:**
- In `EditorGame::update()`, call `viewport_input.handle_input_simple()` and `picker.pick_at_screen_pos()`
- Convert screen click position to world coordinates via `screen_to_world()`
- Build `PickableEntity` list from entities with `GlobalTransform2D` + `Sprite`
- Set `editor.selection` primary entity on pick result
- Hierarchy panel should highlight the selected entity
- Inspector should show selected entity's components
- Only active during `Editing` and `Paused` states (not during `Playing`)
**Tests:**
- Test that clicking in viewport area sets selection
- Test that clicking empty space clears selection
- Test that pick is disabled during Playing state
**Files:** `crates/editor_integration/src/editor_game.rs`, `crates/editor/src/picking.rs`

### TASK-002: Wire up rectangle selection in viewport
**Crate:** editor_integration
**Effort:** M
**Spec:**
- `SelectionRect` already implemented in `picking.rs`
- Integrate viewport input for drag start/end
- Drag in viewport selects all entities within rectangle
- Shift+drag adds to existing selection
- Visual feedback: draw selection rectangle during drag
**Tests:**
- Test that drag selects multiple entities within bounds
- Test that shift+drag adds to selection
- Test that selection rect is only active in Editing/Paused
**Files:** `crates/editor_integration/src/editor_game.rs`, `crates/editor/src/picking.rs`

---

## Phase 1D: Entity Operations

### TASK-003: Create empty entity from editor
**Crate:** editor_integration, editor
**Effort:** S
**Spec:**
- Wire Entity menu > Create Empty to create a new entity with Transform2D at viewport center
- New entity gets a default Name component ("Entity N" where N is entity count)
- Newly created entity is automatically selected
- Hierarchy panel updates to show new entity
**Tests:**
- Test that Create Empty adds entity with Transform2D
- Test that new entity is auto-selected
- Test that entity gets default name
**Files:** `crates/editor_integration/src/editor_game.rs`, `crates/editor/src/context.rs`

### TASK-004: Create entity with sprite
**Crate:** editor_integration, editor
**Effort:** S
**Spec:**
- Wire Entity menu > Create Sprite to create entity with Transform2D + Sprite
- Sprite gets default white texture (texture_handle 0)
- Position at viewport center
- Auto-select after creation
**Tests:**
- Test that Create Sprite adds entity with Transform2D + Sprite
- Test sprite has default texture handle
**Files:** `crates/editor_integration/src/editor_game.rs`

### TASK-005: Create physics entity
**Crate:** editor_integration, editor
**Effort:** S
**Spec:**
- Wire Entity menu > Create Physics Body to create entity with Transform2D + Sprite + RigidBody + Collider
- Use `RigidBody::pushable()` and `Collider::box_collider(50.0, 50.0)` defaults
- Position at viewport center, auto-select
**Tests:**
- Test that Create Physics Body adds all 4 components
- Test that physics components use correct defaults
**Files:** `crates/editor_integration/src/editor_game.rs`

### TASK-006: Delete entity
**Crate:** editor_integration, editor, ecs
**Effort:** M
**Spec:**
- Delete key or Entity menu > Delete removes selected entity
- Remove entity and all components from world
- Reparent children to deleted entity's parent (or make them root entities)
- Clear from editor selection
- Handle case where no entity is selected (no-op)
**Tests:**
- Test that delete removes entity from world
- Test that children are reparented
- Test that selection is cleared after delete
- Test that delete with no selection is a no-op
**Files:** `crates/editor_integration/src/editor_game.rs`, `crates/ecs/src/world.rs`

### TASK-007: Duplicate entity with Ctrl+D
**Crate:** editor_integration, editor, ecs
**Effort:** M
**Spec:**
- Ctrl+D duplicates the selected entity
- Deep copy all components (Transform2D, Sprite, RigidBody, Collider, etc.)
- Offset position by (20, 20) so duplicate is visible
- Duplicate children recursively (preserve hierarchy)
- Select the duplicate after creation
**Tests:**
- Test that duplicate creates new entity with same components
- Test that position is offset
- Test that children are duplicated
- Test that duplicate is auto-selected
**Files:** `crates/editor_integration/src/editor_game.rs`, `crates/ecs/src/world.rs`

---

## Phase 1E: Component Add/Remove

### TASK-008: Add Component button in inspector
**Crate:** editor, editor_integration
**Effort:** M
**Spec:**
- Below existing components in inspector, show "Add Component" button
- Clicking opens a dropdown/list of available component types from ComponentRegistry
- Selecting a type adds that component with default values to the selected entity
- Only show components the entity doesn't already have
**Tests:**
- Test that Add Component button appears in inspector
- Test that selecting a component type adds it to entity
- Test that already-present components are filtered out
**Files:** `crates/editor/src/editable_inspector.rs`, `crates/editor_integration/src/panel_renderer.rs`

### TASK-009: Remove Component button
**Crate:** editor, editor_integration
**Effort:** S
**Spec:**
- Each component section in inspector gets an (X) remove button in its header
- Clicking removes that component from the entity
- If removing RigidBody, also remove Collider (dependency)
- Transform2D cannot be removed (it's required)
**Tests:**
- Test that remove button removes component
- Test that removing RigidBody cascades to Collider
- Test that Transform2D remove is prevented
**Files:** `crates/editor/src/editable_inspector.rs`, `crates/editor_integration/src/panel_renderer.rs`

---

## Technical Debt (MEDIUM PRIORITY)

### TASK-010: Split SpritePipeline into focused managers
**Crate:** renderer
**Effort:** L
**Spec:**
- `SpritePipeline` at `renderer/src/sprite.rs:225-254` manages 13 GPU resources
- Split into: PipelineResources, BufferManager, CameraManager, TextureBindGroupManager
- Each struct owns its resources, SpritePipeline orchestrates
- All existing tests must continue to pass
**Tests:**
- All existing renderer tests pass
- No new public API changes (internal refactor only)
**Files:** `crates/renderer/src/sprite.rs`

### TASK-011: Split FontManager responsibilities
**Crate:** ui
**Effort:** M
**Spec:**
- `FontManager` at `ui/src/font.rs:100-315` handles loading, storage, rasterization, caching, layout
- Split into: FontLoader, GlyphCache, TextLayoutEngine
- FontManager becomes a facade that delegates
**Tests:**
- All existing UI tests pass
- No new public API changes
**Files:** `crates/ui/src/font.rs`

### TASK-012: Fix TransformHierarchySystem double iteration
**Crate:** ecs
**Effort:** S
**Spec:**
- `hierarchy_system.rs:87-118` does separate root and child passes
- Reorganize to single pass with better data flow
- Must handle arbitrary hierarchy depth correctly
**Tests:**
- All existing hierarchy tests pass
- Add test for deep hierarchy (5+ levels) correctness
**Files:** `crates/ecs/src/hierarchy_system.rs`

### TASK-013: Remove dead code with #[allow(dead_code)]
**Crate:** renderer
**Effort:** S
**Spec:**
- 4 documented but unused items in `sprite.rs`, `sprite_data.rs`, `texture.rs`
- Determine if each is genuinely unused or just not yet wired
- Remove truly unused items, wire up items that should be used
**Tests:**
- `cargo clippy --workspace` clean after changes
- All existing tests pass
**Files:** `crates/renderer/src/sprite.rs`, `crates/renderer/src/sprite_data.rs`, `crates/renderer/src/texture.rs`

### TASK-014: Deduplicate CameraUniform
**Crate:** common, renderer
**Effort:** S
**Spec:**
- CameraUniform exists in both `common` and `renderer` crates
- Use `common::CameraUniform` everywhere, remove renderer copy
- Update all imports
**Tests:**
- All tests pass
- `cargo clippy --workspace` clean
**Files:** `crates/common/src/`, `crates/renderer/src/`

---

## New Features (LOWER PRIORITY)

### TASK-015: Implement spatial hash grid for collision broad-phase
**Crate:** physics
**Effort:** L
**Spec:**
- Add `SpatialHashGrid` struct for O(1) broad-phase collision detection
- Grid cell size configurable (default: 2x largest collider)
- Insert/remove/query operations
- Integrate with existing `PhysicsSystem` as optional broad-phase
- Useful for large entity counts (100+) where rapier's built-in may be slower
**Tests:**
- Test insert and query returns correct neighbors
- Test remove cleans up correctly
- Test with overlapping entities across cell boundaries
- Benchmark vs brute-force for 100, 500, 1000 entities
**Files:** `crates/physics/src/spatial_hash.rs` (new), `crates/physics/src/lib.rs`

### TASK-016: Add missing input timing + dead zone tests
**Crate:** input
**Effort:** S
**Spec:**
- Gamepad dead zone normalization is untested
- Frame-accurate event timing is unvalidated
- Add comprehensive tests for both
**Tests:**
- Test dead zone threshold (values below threshold → 0)
- Test dead zone normalization (values above threshold normalized to 0..1)
- Test frame timing: just_pressed only true for one frame
- Test frame timing: just_released only true for one frame
**Files:** `crates/input/tests/` or `crates/input/src/gamepad.rs`

### TASK-017: Add missing physics friction/kinematic/sensor tests
**Crate:** physics
**Effort:** S
**Spec:**
- Core physics materials and triggers are untested
- Add coverage for friction/restitution settings
- Add coverage for kinematic body behavior
- Add coverage for sensor (trigger) colliders
**Tests:**
- Test friction coefficient affects sliding behavior
- Test restitution affects bounce height
- Test kinematic body moves via velocity, not forces
- Test sensor collider detects overlap without physical response
**Files:** `crates/physics/tests/` or `crates/physics/src/`

### TASK-018: Implement basic particle system
**Crate:** engine_core (or new crate)
**Effort:** L
**Spec:**
- `ParticleEmitter` component with configurable: emission rate, lifetime, velocity range, color
- `ParticleSystem` that updates particles each frame
- Integration with renderer for efficient particle drawing (instanced)
- Presets: explosion, smoke, sparkle, rain
**Tests:**
- Test emitter spawns particles at correct rate
- Test particles die after lifetime
- Test velocity range produces correct spread
- Test particle count respects max limit
**Files:** TBD (discuss in BLOCKERS.md if unsure about crate placement)
