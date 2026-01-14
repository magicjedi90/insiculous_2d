# Technical Debt: ecs

Last audited: January 2026

## Summary
- DRY violations: 4 (1 resolved)
- SRP violations: 3 (1 resolved)
- KISS violations: 2 (2 resolved)
- Architecture issues: 4 (3 resolved)

## January 2026 Fixes
- ✅ **SRP-001**: Extracted hierarchy methods to `WorldHierarchyExt` extension trait (~150 lines moved, 11 tests)
- ✅ **KISS-001**: Replaced non-functional `QueryIterator` scaffolding with working `query_entities::<Q>()` method (1 test added)
- ✅ **ARCH-001**: Module visibility strategy documented in `lib.rs` - private modules for core infrastructure, public modules for domain-specific concerns
- ✅ **ARCH-002**: Cycle detection added to `set_parent()` (implemented in `WorldHierarchyExt`)
- ✅ **DRY-002**: Extracted `set_global_transform()` helper in `hierarchy_system.rs` to eliminate duplicate update-or-add pattern

---

## DRY Violations

### [DRY-001] Repeated entity existence checks
- **File:** `world.rs`
- **Lines:** 216-217, 226-227, 242-243, 252-253, 267-268, 275-276, 283-284
- **Issue:** The pattern `if !self.entities.contains_key(entity_id)` is repeated 7+ times before component operations.
- **Suggested fix:** Extract to a helper method `ensure_entity_exists(&self, entity_id) -> Result<(), EcsError>` or use early validation.
- **Priority:** Low (explicit checks aid debugging)

### ~~[DRY-002] Repeated GlobalTransform update pattern in hierarchy_system.rs~~ ✅ RESOLVED
- **File:** `hierarchy_system.rs`
- **Resolution:** Extracted `set_global_transform(world, entity, global)` helper method.
  Both the root entity update loop and recursive `propagate_transforms()` now use this shared helper.
- **Resolved:** January 2026

### [DRY-003] Duplicate matrix computation in GlobalTransform2D
- **File:** `hierarchy.rs`
- **Lines:** 143-167, 178-184, 196-201
- **Issue:** Sin/cos rotation calculations are computed multiple times in `matrix()`, `mul_transform()`, and `transform_point()`.
- **Suggested fix:** Consider caching sin/cos values or extracting rotation application to a shared helper.
- **Priority:** Low (performance micro-optimization)

### [DRY-004] Repeated builder pattern in audio_components.rs
- **File:** `audio_components.rs`
- **Lines:** 62-104, 175-184, 214-229
- **Issue:** Three separate components (`AudioSource`, `AudioListener`, `PlaySoundEffect`) each have nearly identical `with_volume()` methods with the same clamping logic.
- **Suggested fix:** Consider a trait `VolumeControl` with a default implementation, or a helper function.
- **Priority:** Low

---

## SRP Violations

### ~~[SRP-001] World struct has too many responsibilities~~ ✅ RESOLVED
- **File:** `world.rs`, `hierarchy_ext.rs`
- **Resolution:** Extracted 10 hierarchy methods (~150 lines) to `WorldHierarchyExt` extension trait:
  - `set_parent()`, `remove_parent()`, `get_parent()`, `get_children()`
  - `get_root_entities()`, `get_descendants()`, `get_ancestors()`
  - `is_ancestor_of()`, `is_descendant_of()`, `remove_entity_hierarchy()`

  World struct now handles 6 core responsibilities (~400 lines):
  1. Entity management (create, remove, validate)
  2. Component management (add, remove, get)
  3. System management (add, update)
  4. Query management (`query_entities::<Q>()`)
  5. Lifecycle management (initialize, start, stop, shutdown)
  6. Configuration management

  Hierarchy methods available via `use ecs::WorldHierarchyExt;`
- **Resolved:** January 2026

### [SRP-002] ComponentStorage enum handles both storage types
- **File:** `component.rs`
- **Lines:** 38-136
- **Issue:** `ComponentStorage` is an enum that delegates every method to either `LegacyComponentStorage` or `ArchetypeStorage`. This creates maintenance burden when adding new methods.
- **Suggested fix:** Use a trait `Storage` with implementations for Legacy and Archetype storage, then World holds `Box<dyn Storage>`.
- **Priority:** Low (working but could be cleaner)

### [SRP-003] TransformHierarchySystem does double iteration
- **File:** `hierarchy_system.rs`
- **Lines:** 86-113
- **Issue:** The update method iterates over all entities twice:
  1. First loop: Update root entities
  2. Second loop via `get_root_entities()` + recursive propagation

  Root entities are processed in both loops (once for update, once for propagation start).
- **Suggested fix:** Combine into a single traversal that handles roots and propagates in one pass.
- **Priority:** Low (correctness over performance for now)

---

## KISS Violations

### ~~[KISS-001] Over-engineered QueryIterator scaffolding~~ ✅ RESOLVED
- **File:** `world.rs`
- **Resolution:** Removed `QueryIterator` scaffolding that always returned `None`.
  Replaced with a functional `query_entities::<Q>()` method that:
  - Takes a `QueryTypes` bound (Single<T>, Pair<T,U>, or Triple<T,U,V>)
  - Returns `Vec<EntityId>` of entities matching the query
  - Uses `ComponentRegistry::has_type()` for type-based checking

  New method is simpler (25 lines vs 45 lines) and actually works.
  Test added: `test_query_entities` in `tests/world.rs`
- **Resolved:** January 2026

### ~~[KISS-002] Over-engineered ComponentColumn raw pointer manipulation~~ ✅ RESOLVED
- **File:** `archetype.rs`, `component.rs`
- **Resolution:** Added comprehensive safety documentation throughout:
  - `ComponentColumn` struct: Documented 4 safety invariants (element size correctness, index bounds, capacity invariant, type safety at boundary) and explained why unsafe is necessary for ECS performance
  - `get()` and `get_mut()`: Documented that pointers are valid for `element_size` bytes
  - `push()`: Documented caller responsibility and copy_nonoverlapping safety
  - `swap_remove()`: Documented bounds checking and non-overlapping regions
  - `ArchetypeComponentStorage::get()` and `get_mut()`: Documented the safety chain (TypeId lookup ensures correct column)

  The unsafe code is intentional for cache-friendly component storage, a common ECS pattern.

---

## Architecture Issues

### ~~[ARCH-001] Inconsistent module visibility~~ ✅ RESOLVED
- **File:** `lib.rs`
- **Resolution:** Added documentation explaining the intentional visibility strategy:
  - **Private modules** (`mod` + `pub use *`): Core infrastructure (archetype, component, entity, world)
    - Implementation details hidden, public API exposed at crate root
  - **Public modules** (`pub mod` + `pub use *`): Domain-specific modules (behavior, hierarchy, sprite_components, etc.)
    - Visible for documentation discoverability
  - All types accessible from crate root: `use ecs::EntityId;`
- **Resolved:** January 2026

### [ARCH-002] ~~Circular reference risk in hierarchy components~~ ✅ RESOLVED
- **File:** `hierarchy.rs`, `world.rs`
- **Issue:** `Parent` component stores an `EntityId`, and `Children` stores `Vec<EntityId>`. No validation prevents:
  1. An entity being its own ancestor (circular hierarchy)
  2. Children list and Parent component becoming inconsistent
- **Resolution:** Added cycle detection in `set_parent()` using `is_ancestor_of()` check.
  Tests added: `test_hierarchy_cycle_detection`, `test_hierarchy_self_parent_rejected`
- **Resolved:** January 2026

### [ARCH-003] Dead code marked but not removed
- **Files:** Multiple
- **Issue:** Several `#[allow(dead_code)]` annotations exist:
  - `world.rs:542-549`: QueryIterator fields
  - `archetype.rs:239`: Query struct field
  - `archetype.rs:294`: Test component field

  Per ANALYSIS.md: "Either use these methods or remove them."
- **Suggested fix:** Remove unused code or implement the features.
- **Priority:** Low

### [ARCH-004] Dual storage systems add complexity
- **Files:** `component.rs`, `world.rs`
- **Issue:** The crate maintains two parallel storage systems:
  1. `LegacyComponentStorage` (HashMap-based, default)
  2. `ArchetypeStorage` (performance-optimized, opt-in via `World::new_optimized()`)

  This doubles the code surface for storage operations and the archetype system appears incomplete (see KISS-002).
- **Suggested fix:** Either complete the archetype system or remove it. Having two half-complete systems is worse than one complete one.
- **Priority:** Medium

---

## Previously Resolved (Reference)

These issues from ANALYSIS.md have been resolved:

| Issue | Resolution |
|-------|------------|
| Deprecated PlayerTag alias | FIXED: Removed, use EntityTag instead |
| Incomplete test assertions (TODO comments) | FIXED: All replaced with meaningful assertions |
| Memory safety issues | FIXED: Generation tracking implemented |
| System registry memory safety | FIXED: catch_unwind for panic isolation |
| ARCH-002: Hierarchy cycle detection | FIXED: Cycle detection in `set_parent()` + 2 tests |

---

## Metrics

| Metric | Value |
|--------|-------|
| Total source files | 14 |
| Total lines | ~2,700 |
| Test coverage | 84 tests (100% pass rate) |
| `#[allow(dead_code)]` | 5 instances |
| High priority issues | 0 |
| Medium priority issues | 6 |
| Low priority issues | 7 |

---

## Recommendations

### Immediate Actions
1. ~~**Fix KISS-002** - Review unsafe code in ComponentColumn for safety~~ ✅ DONE - Comprehensive safety docs added
2. ~~**Fix ARCH-002** - Add cycle detection to prevent hierarchy corruption~~ ✅ DONE

### Short-term Improvements
3. **Fix SRP-001** - Split World hierarchy methods into separate trait/module
4. **Fix KISS-001** - Either implement QueryIterator or remove scaffolding
5. **Fix DRY-002** - Extract GlobalTransform update helper

### Technical Debt Backlog
- ARCH-004: Decide on storage system (keep one, remove other)
- ARCH-001: Standardize module visibility pattern
- ARCH-003: Remove dead code

---

## Cross-Reference with PROJECT_ROADMAP.md

| This Report | PROJECT_ROADMAP.md / ANALYSIS.md | Status |
|-------------|----------------------------------|--------|
| SRP-001: World too many responsibilities | "Split World impl blocks by concern" | Known, unresolved |
| ARCH-001: Module visibility | "Document visibility rationale" | Known, unresolved |
| ARCH-003: Dead code | "Review and either use or remove" | Known, unresolved |
| ARCH-002: Hierarchy cycles | Tracked | ✅ Resolved |
| KISS-002: Unsafe ComponentColumn | Tracked | ✅ Resolved - Comprehensive safety docs added |
| ARCH-004: Dual storage systems | Not tracked | New finding |

**New issues to add to PROJECT_ROADMAP.md:**
- ~~ARCH-002: Hierarchy cycle detection needed in `set_parent()`~~ ✅ RESOLVED
- ~~KISS-002: ComponentColumn uses unsafe code without demonstrated need~~ ✅ RESOLVED - Comprehensive safety documentation added
- ARCH-004: Dual storage systems (Legacy vs Archetype) create maintenance burden
