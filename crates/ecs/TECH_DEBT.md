# Technical Debt: ecs

Last audited: February 2026

## Summary
- DRY violations: 4 (1 resolved)
- SRP violations: 3 (2 resolved)
- KISS violations: 2 (2 resolved)
- Architecture issues: 4 (4 resolved)
- Pattern drift: 2 (2 resolved)

## February 2026 Fixes
- ✅ **PATTERN-001**: Broken archetype storage removed entirely. ECS now uses single HashMap-based per-type storage. Proper archetype storage deferred as future ground-up rewrite.
- ✅ **PATTERN-002**: Dual-storage system removed. `World::new_optimized()`, `ComponentStorage` enum, `ArchetypeStorage`, and `archetype.rs` deleted. Single storage path via `ComponentRegistry` → `ComponentStore`.
- ✅ **SRP-002**: `ComponentStorage` enum removed. Single `ComponentStore` type (no delegation/match arms).
- ✅ **ARCH-003**: Archetype dead code removed (`archetype.rs` deleted, query types extracted to `query.rs`).
- ✅ **ARCH-004**: Dual storage systems removed. Single HashMap-based storage path.

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

### ~~[SRP-002] ComponentStorage enum handles both storage types~~ ✅ RESOLVED
- **File:** `component.rs`
- **Resolution:** `ComponentStorage` enum deleted along with `ArchetypeStorage` and `LegacyComponentStorage`. Single `ComponentStore` type now used directly by `ComponentRegistry`. No delegation or match arms.
- **Resolved:** February 2026

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
- **File:** `archetype.rs` (now deleted)
- **Resolution:** Originally resolved with safety documentation (January 2026). The entire `archetype.rs` file including `ComponentColumn`, `Archetype`, and `ArchetypeStorage` was deleted in February 2026 as part of the dual-storage removal. The unsafe raw pointer code is no longer in the codebase.

---

## Pattern Drift Issues (Robert Nystrom Patterns Audit - January 2026)

### ~~[PATTERN-001] ECS Component Pattern: Archetype scaffolding with trait-object interface~~ ✅ RESOLVED
- **File:** `component.rs` (formerly also `archetype.rs`)
- **Resolution:** Broken archetype storage code removed entirely (`archetype.rs` deleted, `ArchetypeStorage` and `ComponentColumn` removed from `component.rs`). ECS now honestly uses HashMap-based per-type storage via `ComponentStore`. The `Component` trait still requires `as_any()` for downcasting — this is inherent to HashMap-based storage with `Box<dyn Component>` and will only be removable when a proper archetype storage is built from scratch (future work).
- **Resolved:** February 2026

### ~~[PATTERN-002] ECS Default Storage: Legacy mode contradicts archetype claims~~ ✅ RESOLVED
- **File:** `world.rs`, `component.rs`
- **Resolution:** Dual-storage system removed. `WorldConfig::use_archetype_storage` field deleted, `World::new_optimized()` method deleted, `ComponentStorage` enum deleted. Single storage path: `World::new()` → `ComponentRegistry::new()` → `HashMap<TypeId, ComponentStore>`. Documentation updated from "archetype-based" to "HashMap-based per-type storage".
- **Resolved:** February 2026

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

### ~~[ARCH-003] Dead code marked but not removed~~ ✅ RESOLVED
- **Files:** `archetype.rs` (deleted), `world.rs`
- **Resolution:** `archetype.rs` deleted entirely (contained most dead code). Query types extracted to `query.rs`. Dead code annotations in `world.rs` for archetype-related fields removed along with the fields themselves.
- **Resolved:** February 2026

### ~~[ARCH-004] Dual storage systems add complexity~~ ✅ RESOLVED
- **Files:** `component.rs`, `world.rs`
- **Resolution:** Dual storage system removed. `ArchetypeStorage`, `LegacyComponentStorage`, and `ComponentStorage` enum all deleted. Single `ComponentStore` (HashMap-based) used by `ComponentRegistry`. `World::new_optimized()` removed.
- **Resolved:** February 2026

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
| Total source files | 13 |
| Total lines | ~2,100 |
| Test coverage | 110 tests (100% pass rate) |
| `#[allow(dead_code)]` | 0 instances |
| High priority issues | 0 |
| Medium priority issues | 3 |
| Low priority issues | 5 |

---

## Recommendations

### All Immediate Actions Resolved
1. ~~**Fix KISS-002** - Review unsafe code in ComponentColumn for safety~~ ✅ DONE
2. ~~**Fix ARCH-002** - Add cycle detection to prevent hierarchy corruption~~ ✅ DONE
3. ~~**Fix SRP-001** - Split World hierarchy methods into separate trait/module~~ ✅ DONE
4. ~~**Fix KISS-001** - Either implement QueryIterator or remove scaffolding~~ ✅ DONE
5. ~~**Fix DRY-002** - Extract GlobalTransform update helper~~ ✅ DONE
6. ~~**ARCH-004** - Decide on storage system~~ ✅ DONE - Dual storage removed
7. ~~**ARCH-003** - Remove dead code~~ ✅ DONE - archetype.rs deleted

### Remaining Technical Debt
- SRP-003: TransformHierarchySystem double iteration
- DRY-001: Repeated entity existence checks
- DRY-003: Duplicate matrix computation in GlobalTransform2D
- DRY-004: Repeated builder pattern in audio_components.rs

---

## Cross-Reference with PROJECT_ROADMAP.md

| This Report | PROJECT_ROADMAP.md / ANALYSIS.md | Status |
|-------------|----------------------------------|--------|
| SRP-001: World too many responsibilities | "Split World impl blocks by concern" | ✅ Resolved |
| ARCH-001: Module visibility | "Document visibility rationale" | ✅ Resolved |
| ARCH-003: Dead code | "Review and either use or remove" | ✅ Resolved |
| ARCH-002: Hierarchy cycles | Tracked | ✅ Resolved |
| KISS-002: Unsafe ComponentColumn | Tracked | ✅ Resolved (code deleted) |
| ARCH-004: Dual storage systems | Tracked | ✅ Resolved |
| PATTERN-001: Archetype trait-object interface | Tracked | ✅ Resolved (code deleted) |
| PATTERN-002: Legacy storage default | Tracked | ✅ Resolved (single storage) |

---

## Future Enhancements (Not Technical Debt)

These features would enhance the ECS but are not required for current functionality:

### Proper Archetype Storage (Ground-Up Rewrite)
- Single shared `ArchetypeStorage` replacing `ComponentRegistry`
- Typed `push<T>`/`get<T>` without `Box<dyn Component>`
- Proper Drop handling via stored drop functions
- Archetype migration on component add/remove
- Dense columnar arrays (`Vec<T>`) with cache locality

### System Scheduling
- Add system dependency graph for automatic execution ordering
- Parallel system execution for multi-core optimization
- System groups for organizing related systems

### Component Introspection
- Component reflection for runtime type information
- Dynamic component addition/removal based on string names
- Editor integration for visual component editing

### Performance Optimizations
- Memory pooling for entity and component allocations
- Component pack optimization for cache locality
