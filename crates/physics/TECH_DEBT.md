# Technical Debt: physics

Last audited: January 2026

## Summary
- DRY violations: 2
- SRP violations: 1
- KISS violations: 1
- Architecture issues: 2

**Overall Assessment:** The physics crate is well-designed with clean rapier2d integration. Most issues are minor and relate to code organization rather than functionality.

---

## DRY Violations

### [DRY-001] Repeated pixel-to-meter conversion pattern
- **File:** `physics_world.rs`
- **Lines:** Throughout (163-180, 188-189, 242-264, 344-347, 403-412, 454-448, etc.)
- **Issue:** The pattern `value / self.config.pixels_per_meter` or `value * ppm` is repeated dozens of times throughout the file. Each physics method manually handles the conversion.
- **Suggested fix:** Consider using newtype wrappers like `Pixels(f32)` and `Meters(f32)` with `From` implementations, or at minimum inline helper methods for common cases like position/velocity.
- **Priority:** Low (working, just verbose)

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

### [KISS-001] Collision event handling is incomplete
- **File:** `physics_world.rs`
- **Lines:** 352-370
- **Issue:** The `step()` method creates an empty event handler:
  ```rust
  let event_handler = ();
  ```
  Then manually iterates contact pairs to collect collision events. This bypasses rapier's proper event system which provides `started` and `stopped` events. Current implementation always sets `started: true, stopped: false`.
- **Suggested fix:** Implement a proper `EventHandler` to track collision start/stop transitions, or document that only "currently colliding" events are supported.
- **Priority:** Medium (affects gameplay - can't detect collision end)

---

## Architecture Issues

### [ARCH-001] PhysicsSystem has pass-through methods
- **File:** `physics_system.rs`
- **Lines:** 72-100
- **Issue:** Several methods on `PhysicsSystem` are simple pass-throughs to `PhysicsWorld`:
  ```rust
  pub fn set_gravity(&mut self, gravity: Vec2) {
      self.physics_world.set_gravity(gravity);
  }
  pub fn gravity(&self) -> Vec2 {
      self.physics_world.gravity()
  }
  pub fn apply_impulse(&mut self, entity: EntityId, impulse: Vec2) {
      self.physics_world.apply_impulse(entity, impulse);
  }
  ```
  Users could just call `physics_system.physics_world().method()` directly.
- **Suggested fix:** Either:
  1. Keep pass-throughs for cleaner API (acceptable)
  2. Remove them and document users should use `physics_world()`
  3. Use `Deref` to `PhysicsWorld` (unusual but works)
- **Priority:** Low (convenience methods are fine)

### [ARCH-002] Collision callback stored as `Option<Box<dyn FnMut>>`
- **File:** `physics_system.rs`
- **Lines:** 14, 27
- **Issue:** The collision callback uses a complex type:
  ```rust
  type CollisionCallback = Box<dyn FnMut(&CollisionData) + Send + Sync>;
  collision_callback: Option<CollisionCallback>,
  ```
  This only supports a single callback. If multiple systems need collision notifications, they can't both register.
- **Suggested fix:** Consider:
  1. Use `Vec<CollisionCallback>` for multiple callbacks
  2. Provide a pub/sub pattern
  3. Document single-callback limitation
- **Priority:** Low (current limitation is acceptable for simple games)

---

## Previously Resolved (Reference)

These issues from ANALYSIS.md have been resolved:

| Issue | Resolution |
|-------|------------|
| Dead code in PhysicsWorld | FIXED: `pixels_to_meters_scalar`, `meters_to_pixels`, `meters_to_pixels_scalar` are now public API |

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
| Total lines | ~1,100 |
| Test coverage | 22 tests (all passing) |
| Rapier2d types managed | 13 |
| High priority issues | 0 |
| Medium priority issues | 1 |
| Low priority issues | 5 |

---

## Recommendations

### Immediate Actions
1. **Fix KISS-001** - Implement proper collision start/stop detection or document limitation

### Short-term Improvements
2. **Add tests** for friction, kinematic bodies, and sensors (per ANALYSIS.md)
3. **Document** single-callback limitation (ARCH-002)

### Technical Debt Backlog
- DRY-001: Consider newtype wrappers for Pixels/Meters (optional)
- SRP-001: Consider reorganizing PhysicsWorld internals (optional)
- ARCH-001: Decide on pass-through method policy

---

## Cross-Reference with PROJECT_ROADMAP.md / ANALYSIS.md

| This Report | ANALYSIS.md | Status |
|-------------|-------------|--------|
| Dead code warning | "Coordinate conversion methods" | RESOLVED - Now public API |
| KISS-001: Collision events | Not tracked | New finding |
| Test gaps | "No friction/kinematic/sensor tests" | Feature gap (not debt) |

**New issues to add to PROJECT_ROADMAP.md:**
- KISS-001: Collision event handling doesn't detect collision end (only "currently colliding")

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
