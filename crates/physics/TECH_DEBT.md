# Technical Debt: physics

Last audited: June 2026 (audit remediation pass)

## Summary
- DRY violations: 0 (3 resolved)
- SRP violations: 1
- KISS violations: 0 (1 resolved)
- Architecture issues: 0 (2 resolved/documented)
- Open: SRP-001 (low), API-001 (low), MISSING-001 remainder (low)

## June 2026 Audit Remediation
All correctness findings fixed; final state: **58 passing lib tests, 0 failed**.

- ✅ **Stale event re-emission**: `step()` no longer clears the event buffer;
  `clear_collision_events()` added, `PhysicsSystem::update()` clears once
  before the sub-step loop. Zero-step frames emit nothing.
- ✅ **Sub-step event loss**: `step()` APPENDS events, so all catch-up
  sub-steps' events survive a single update.
- ✅ **Contact points were collider-local**: now transformed through
  collider1's world isometry (point and normal) before meters→pixels.
- ✅ **`apply_force` persisted forever**: forces reset after the step loop
  (`PhysicsWorld::reset_forces()`); skipped on zero-step frames so a force
  applied then still acts on the next stepped frame. Documented as one-update.
- ✅ **Dead `RigidBody::apply_impulse`/`apply_force` component methods
  removed**: they mutated a `velocity` field only read at body creation
  (silent no-op on live bodies).
- ✅ **MISSING-001 (partial)**: `pixels_per_meter` validated in `with_scale`
  and at `PhysicsWorld::new` (non-finite or <= 0 → warn + default 100.0).
  Gravity NaN and negative collider dimensions still unvalidated (low).
- ✅ **CollisionPair bit-packing overflow**: canonical ordering now compares
  `(value, generation)` tuples instead of `value | (generation << 32)`.
- ✅ **`raycast` direction**: normalized internally; zero/non-finite direction
  returns `None`; distance always in pixels along the ray.
- ✅ **Kinematic dead config**: removed ignored `linvel`/`angvel` on
  `kinematic_position_based` bodies.
- ✅ **Non-test `unwrap()`**: `NonZeroUsize::new(8).unwrap()` replaced with
  `config.solver_iterations.max(1)` + `NonZeroUsize::MIN` fallback (derives
  from config with a floor of 1 instead of silently substituting 8).
- ✅ **Iteration names matched to rapier**: `velocity_iterations` /
  `position_iterations` renamed to `solver_iterations` / `friction_iterations`
  (they map to `num_solver_iterations` / `num_additional_friction_iterations`).
  Only physics-internal code used the field names, so the rename was safe.
- ✅ **Unused deps removed**: `toml`, `common`, `thiserror` (after
  `PhysicsError` deletion) dropped from Cargo.toml.
- ✅ **Dead public API deleted**: `PhysicsError` (never constructed),
  `MovementConfig` (consumed by nothing; damping literals inlined into the
  two presets that referenced it). `Collider::player_box()` now takes
  `(width, height)` instead of hardcoding 80x80.
- ✅ **DRY-002/DRY-003**: `body(entity)`/`body_mut(entity)` private helpers
  replace seven duplicated two-level lookups; translation/rotation builder
  setup deduplicated across the three RigidBodyType match arms.
- ✅ **600-line rule**: `physics_world.rs` (977) split into
  `physics_world/{mod,bodies,stepping,queries,tests}.rs`;
  `physics_system.rs` (775) split into
  `physics_system/{mod,sync,update,tests}.rs`. All files < 601 lines.
- ✅ **Test gaps closed**: sensor intersection events, callbacks firing on a
  real collision, world-space contact points, scale validation, raycast
  normalization, event delivery contracts, and pinned physics+hierarchy
  behavior (physics entities must be root entities — see CLAUDE.md).
- ℹ️ **`CollisionCallback` keeps `Send + Sync`**: required because
  `ecs::System` has `Any + Send + Sync` supertraits and `PhysicsSystem`
  implements it (stored as `Box<dyn System>` in `SystemRegistry`).

## January 2026 Fixes
- ✅ **ARCH-002**: PhysicsSystem now supports multiple collision callbacks via:
  - `with_collision_callback()` - builder pattern, chainable
  - `add_collision_callback()` - mutable method
  - `clear_collision_callbacks()` - remove all callbacks
  - `collision_callback_count()` - query number of registered callbacks
  - Tests added: `test_multiple_collision_callbacks`, `test_add_collision_callback`

**Overall Assessment:** The physics crate is well-designed with clean rapier2d integration. Most issues are minor and relate to code organization rather than functionality.

---

## DRY Violations

### ~~[DRY-001] Repeated pixel-to-meter conversion pattern~~ ✅ RESOLVED
- **File:** `physics_world.rs`
- **Resolution:** Refactored all 12+ locations to consistently use the helper methods:
  - `pixels_to_meters(Vec2)` / `pixels_to_meters_scalar(f32)` for converting to meters
  - `meters_to_pixels(Vec2)` / `meters_to_pixels_scalar(f32)` for converting to pixels
- **Methods updated:** `add_collider`, `step`, `get_contact_points_from_pair`, `get_body_transform`, `get_body_velocity`, `set_body_transform`, `set_kinematic_target`, `set_body_velocity`, `apply_impulse`, `apply_force`, `raycast`
- **Benefit:** Single point of change for conversion logic, improved readability

### ~~[DRY-002] Repeated body builder pattern in add_rigid_body~~ ✅ RESOLVED (June 2026)
- **File:** `physics_world/bodies.rs`
- **Resolution:** The match arms now produce only the per-type builder; the
  shared `.translation(...)/.rotation(...)/.build()` chain is applied once
  after the match. (Same fix as DRY-003 — they were the same finding.)

---

## SRP Violations

### [SRP-001] PhysicsWorld handles too many rapier types
- **File:** `physics_world.rs`
- **Lines:** 73-108
- **Issue:** `PhysicsWorld` manages 13 rapier2d types:
  1. RigidBodySet
  2. ColliderSet
  3. PhysicsPipeline
  4. IslandManager
  5. DefaultBroadPhase
  6. NarrowPhase
  7. ImpulseJointSet
  8. MultibodyJointSet
  9. CCDSolver
  10. QueryPipeline
  11. IntegrationParameters
  12. PhysicsConfig
  13. 4 entity-handle mappings

  While this is somewhat necessary due to rapier2d's API, the struct could be organized better.
- **Suggested fix:** Consider grouping rapier types into sub-structs:
  - `RapierSimulation` - pipeline, islands, phases, solver
  - `RapierSets` - body set, collider set, joint sets
  - `EntityMapping` - the 4 hash maps
- **Priority:** Low (working, rapier API somewhat requires this)

---

## KISS Violations

### ~~[KISS-001] Collision event handling is incomplete~~ ✅ RESOLVED
- **File:** `physics_world.rs`
- **Resolution:** Implemented proper collision start/stop tracking by:
  1. Added `CollisionPair` type with canonical ordering for consistent comparison
  2. Added `previous_collisions: HashSet<CollisionPair>` to track active collisions between frames
  3. `started` flag is now `true` only when collision is new (not in previous frame)
  4. `stopped` flag is now `true` when collision ended (was in previous but not current)
  5. Added 4 new tests: `test_collision_started_event`, `test_collision_ongoing_not_started`, `test_collision_stopped_event`, `test_collision_pair_canonical_order`

---

## Architecture Issues

### ~~[ARCH-001] PhysicsSystem has pass-through methods~~ ✅ DOCUMENTED
- **File:** `physics_system.rs`
- **Resolution:** Added module-level documentation explaining the API design:
  - Pass-through methods exist intentionally for **API ergonomics**
  - `physics_system.set_velocity(...)` is cleaner than `physics_system.physics_world_mut().set_velocity(...)`
  - Users who need advanced operations can still access `PhysicsWorld` via `physics_world()` / `physics_world_mut()`
- **Resolved:** January 2026
- **Follow-up (April 2026):** Collapsed `set_body_velocity` + `apply_impulse` into a single `set_velocity` on the game-facing `PhysicsSystem` API. Every callsite in the workspace was semantically "launch this body at velocity V"; `apply_impulse` was a silent footgun on same-frame spawns. `PhysicsWorld::apply_impulse` remains for the rare genuine mass-aware momentum delta (used by `behavior_runner` for jump impulses).

### ~~[ARCH-002] Collision callback stored as `Option<Box<dyn FnMut>>`~~ ✅ RESOLVED
- **File:** `physics_system.rs`
- **Resolution:** Changed from `Option<CollisionCallback>` to `Vec<CollisionCallback>`:
  - Multiple systems can now register for collision notifications
  - All registered callbacks are invoked for each collision event
  - Callbacks are invoked in registration order
  - New methods: `add_collision_callback()`, `clear_collision_callbacks()`, `collision_callback_count()`
  - Tests added for multiple callback registration and clearing
- **Resolved:** January 2026

---

## Previously Resolved (Reference)

These issues have been resolved:

| Issue | Resolution |
|-------|------------|
| Dead code in PhysicsWorld | FIXED: `pixels_to_meters_scalar`, `meters_to_pixels`, `meters_to_pixels_scalar` are now public API |
| KISS-001: Collision events incomplete | FIXED: Proper start/stop detection with frame-to-frame collision tracking |
| DRY-001: Repeated pixel-to-meter conversion | FIXED: All 12+ locations now use helper methods consistently |
| ARCH-002: Single collision callback | FIXED: `Vec<CollisionCallback>` now supports multiple listeners |

---

## Remaining Gaps (from ANALYSIS.md)

These are **test coverage gaps**, not code quality issues:
- No friction/restitution tests
- No kinematic body tests
- ~~No sensor trigger tests~~ ✅ `test_sensor_collider_fires_intersection_events`
- ~~No collision response validation tests~~ ✅ collision pushes boxes apart (`lib.rs` integration test)
- No high-speed tunneling tests

---

## Metrics (June 2026)

| Metric | Value |
|--------|-------|
| Total source files | 13 (incl. split module dirs) |
| Largest file | 600 lines (`physics_system/tests.rs`) |
| Test coverage | 58 lib tests (all passing), 3 ignored doctests |
| High priority issues | 0 |
| Medium priority issues | 0 |
| Low priority issues | 3 (SRP-001, API-001, MISSING-001 remainder) |

---

## Recommendations

### Immediate Actions
None required - all high/medium priority issues resolved.

### Short-term Improvements
1. **Add tests** for friction, kinematic bodies, and sensors (per ANALYSIS.md)
2. **Document** single-callback limitation (ARCH-002)

### Technical Debt Backlog
- ~~DRY-001: Consider newtype wrappers for Pixels/Meters (optional)~~ ✅ RESOLVED - using helper methods
- SRP-001: Consider reorganizing PhysicsWorld internals (optional)
- ARCH-001: Decide on pass-through method policy

---

## Cross-Reference with PROJECT_ROADMAP.md / ANALYSIS.md

| This Report | ANALYSIS.md | Status |
|-------------|-------------|--------|
| Dead code warning | "Coordinate conversion methods" | RESOLVED - Now public API |
| KISS-001: Collision events | Not tracked | ✅ RESOLVED |
| DRY-001: Pixel/meter conversion | Not tracked | ✅ RESOLVED - Helper methods used |
| Test gaps | "No friction/kinematic/sensor tests" | Feature gap (not debt) |

---

## Code Quality Notes

### Strengths
1. **Clean rapier2d integration** - Well-wrapped physics engine
2. **Excellent presets** - Ready-to-use configurations for common game types
3. **Good ECS integration** - PhysicsSystem properly syncs with World
4. **Fixed timestep** - Proper physics simulation with accumulator
5. **Comprehensive builder API** - Both RigidBody and Collider have clean builders
6. **Good documentation** - Both code and ANALYSIS.md are well-documented

### Minor Observations
- `presets.rs` extends existing structs with `impl Collider {}` blocks - clean pattern
- Coordinate conversion (100 px/m) is well-documented
- Raycasting is properly implemented with query pipeline

---

## New Findings (February 2026 Audit)

4 new issues (0 High, 1 Medium, 3 Low)

### MISSING-001: No validation for degenerate physics values — ✅ PARTIALLY RESOLVED (June 2026)
- **File:** `src/physics_world/mod.rs`, `src/components.rs`
- **Resolved:** `pixels_per_meter` is validated in `with_scale()` AND at
  `PhysicsWorld::new()` (covers struct-literal construction): non-finite or
  <= 0 values warn and fall back to `DEFAULT_PIXELS_PER_METER` (100.0).
  Tests: `test_zero_scale_falls_back_to_default_and_produces_finite_positions`,
  `test_invalid_scale_in_struct_literal_is_sanitized_at_world_creation`.
- **Remaining:** gravity can be NaN, negative collider dimensions accepted
- **Priority:** Low | **Effort:** Small

### API-001: Missing getter methods for PhysicsSystem timing config
- **File:** `src/physics_system.rs:43-47`
- **Issue:** No public getters for fixed_timestep, max_delta_time, time_accumulator
- **Suggested fix:** Add getter methods
- **Priority:** Low | **Effort:** Trivial

### ~~DRY-003: Repeated builder setup in add_rigid_body~~ ✅ RESOLVED (June 2026)
- **File:** `src/physics_world/bodies.rs`
- **Resolution:** Common `.translation(...).rotation(...).build()` extracted
  after the body-type match; each arm now contains only type-specific setup.
  The dead `linvel`/`angvel` config on kinematic bodies was removed at the
  same time (rapier ignores velocities on position-based kinematic bodies).

### SRP-002: Collider clamping inconsistent with builder pattern
- **File:** `src/components.rs:267-276`
- **Issue:** friction/restitution clamped but shape dimensions not validated
- **Suggested fix:** Centralize validation in validate() method
- **Priority:** Low | **Effort:** Small
