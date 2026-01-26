# Technical Debt: physics

Last audited: January 2026

## Summary
- DRY violations: 1 (1 resolved)
- SRP violations: 1
- KISS violations: 0 (1 resolved)
- Architecture issues: 2 (2 resolved/documented)

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

### [DRY-002] Repeated body builder pattern in add_rigid_body
- **File:** `physics_world.rs`
- **Lines:** 192-221
- **Issue:** The `add_rigid_body()` method has repeated builder calls across Dynamic, Static, and Kinematic variants:
  ```rust
  .translation(vector![pos.x, pos.y])
  .rotation(rotation)
  ```
  This is duplicated in all three match arms.
- **Suggested fix:** Extract common builder setup to a helper, then specialize per body type.
- **Priority:** Low

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
  - `physics_system.apply_impulse(...)` is cleaner than `physics_system.physics_world_mut().apply_impulse(...)`
  - Users who need advanced operations can still access `PhysicsWorld` via `physics_world()` / `physics_world_mut()`
- **Resolved:** January 2026

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
- No sensor trigger tests
- No collision response validation tests
- No high-speed tunneling tests

---

## Metrics

| Metric | Value |
|--------|-------|
| Total source files | 6 |
| Total lines | ~1,200 |
| Test coverage | 28 tests (all passing) |
| Rapier2d types managed | 14 (including CollisionPair) |
| High priority issues | 0 |
| Medium priority issues | 0 |
| Low priority issues | 3 |

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
