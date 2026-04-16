# Physics Crate Analysis

> **Audit: 2026-04-15** — Removed completed items: fixed dead-code section
> (coordinate conversion methods are now public API), stale test counts,
> pre-April movement examples that referenced the removed `apply_impulse`
> pass-through on `PhysicsSystem`, resolved collision start/stop work
> (commit f46463e), resolved sensor intersection detection (c2672ab),
> resolved deferred velocity handling (9155bb5), and resolved clear/re-sync
> support (33dd712). Kept: architecture notes, unified velocity API
> rationale, presets inventory, remaining test-coverage gaps
> (friction/restitution/kinematic-movement/sensor-trigger scenarios),
> and long-term enhancement list.

## Current State (Updated: April 2026)

The physics crate provides 2D physics simulation for the Insiculous 2D game
engine using rapier2d as the underlying physics engine.

**Test Count:** 44 unit tests + 3 ignored doctests — all passing.

### Summary
- Rapier2D-backed physics system with ECS components, presets, and fixed
  timestep updates.
- Exposes `PhysicsWorld`, `PhysicsSystem`, and helpers for common
  body/collider setups.
- Integrates through `engine_core` as a default feature.
- Unified velocity API (`PhysicsSystem::set_velocity()`) replaces the older
  `apply_impulse` pass-through — velocity is buffered safely for same-frame
  spawns via `pending_velocities`.

### Strengths
- Clear component API with presets for platformer/top-down workflows.
- In-crate tests validate base integration, simulation flow, collision
  start/stop events, and clear/re-sync behavior.
- Coordinate conversion helpers are part of the public API
  (`pixels_to_meters_scalar`, `meters_to_pixels`, `meters_to_pixels_scalar`).
- Multiple collision callbacks supported (`add_collision_callback`).
- Sensor intersection events routed through the narrow phase.
- Kinematic position-based bodies supported via `set_kinematic_target`.

---

## Remaining Test Coverage Gaps

### 1. No Tests for Friction/Restitution Values
**Location:** Test suite
**Issue:** No tests verify that friction and restitution values are applied
correctly end-to-end. Preset values are spot-checked (e.g. `bouncy()` sets
`restitution = 0.9`) but no simulation-level assertions exist.

**Impact:** Physics material behavior unvalidated.

**Recommended Fix:** Add tests for:
- High friction surfaces (player sticks to walls)
- Low friction surfaces (ice/oil sliding)
- Bouncy surfaces (trampolines — assert resulting bounce velocity)
- Zero restitution (no bounce)

### 2. No Kinematic Body Movement Tests
**Location:** Test suite
**Issue:** `set_kinematic_target` is implemented and kinematic bodies sync
back to ECS, but no test exercises kinematic platform movement or its
interaction with dynamic bodies (the classic moving-platform scenario).

**Impact:** Moving platform behavior unvalidated.

**Recommended Fix:** Add tests for:
- Kinematic body movement via `set_kinematic_target`
- Dynamic bodies riding/colliding with a moving kinematic platform
- Position/rotation updates propagating back to `Transform2D`

### 3. No Sensor Trigger Tests
**Location:** Test suite
**Issue:** `Collider::as_sensor()` exists and sensor intersection events
are collected in `physics_world.rs` (post-commit c2672ab), but no test
asserts trigger-volume behavior.

**Impact:** Collectible/trigger area behavior unvalidated.

**Recommended Fix:** Add tests for:
- Sensor detection firing an event without a physical collision response
- Enter/exit transitions (`started` / `stopped` flags on `CollisionEvent`)
- Multiple entities inside a sensor area simultaneously

---

## Architecture

### Components (`components.rs`)
Physics components integrate with the ECS system:

- **RigidBody**: Defines physical properties
  - Body type (Dynamic, Static, Kinematic)
  - Velocity and angular velocity
  - Gravity scale, damping
  - Rotation lock

- **Collider**: Defines collision shape and properties
  - Shapes: Box, Circle, CapsuleX, CapsuleY
  - Friction and restitution
  - Sensor mode for trigger areas (`as_sensor()`)
  - Collision groups/filters

- **CollisionEvent/CollisionData**: Collision event information
  - Entity pairs
  - `started` / `stopped` flags for enter/exit detection
  - Contact points with normals and depth (empty for sensors)

### Physics World (`physics_world.rs`)
Wrapper around rapier2d that manages:
- Rigid body and collider sets
- Physics pipeline stepping
- Entity to rapier handle mapping
- Coordinate conversion (pixels <-> meters) — **public API**
- Collision event collection (contact + sensor intersection)
- Raycasting
- Clear/reset for editor play-mode stop

**Key design decisions:**
- Uses 100 pixels per meter scale factor
- Fixed timestep simulation (1/60s default)
- Automatic handle cleanup when entities removed
- Kinematic position-based bodies (not velocity-based) for predictable
  platform movement

### Physics System (`physics_system.rs`)
ECS system that:
- Syncs ECS components to rapier physics world
- Steps physics simulation with fixed timestep (with a max-delta guard
  against spiral-of-death)
- Syncs physics results back to ECS transforms (Dynamic + Kinematic)
- Supports multiple collision callbacks
- Buffers velocities for entities not yet synced to Rapier
  (`pending_velocities`) — this is why `set_velocity` is safe on
  same-frame spawns

### Velocity API Design (April 2026)

`PhysicsSystem::set_velocity(entity, linear, angular)` is the single,
universal "launch / move this body at velocity V" API. The older
`apply_impulse` pass-through on `PhysicsSystem` was removed because:

1. Every in-workspace callsite was semantically "start this body at
   velocity V" rather than a mass-aware momentum delta.
2. Having two functions for the same intent was a footgun — `apply_impulse`
   silently no-ops on same-frame spawns, while `set_velocity` defers and
   applies once the body syncs.

`PhysicsWorld::apply_impulse` remains for the rare case that genuinely
needs mass-aware impulse semantics on a live body.

---

## Integration with Engine

1. **Optional Dependency**: Physics is a default feature that can be disabled
2. **Prelude Export**: Physics types available via `engine_core::prelude::*`
3. **ECS Integration**: Components work with existing Transform2D
4. **Editor Integration**: `physics_system.clear()` called on play-mode stop
   to reset simulation state while preserving config and callbacks

```rust
use engine_core::prelude::*;

// Create physics system
let mut physics = PhysicsSystem::with_config(
    PhysicsConfig::new(Vec2::new(0.0, -980.0))
);

// Create entity with physics
let entity = world.create_entity();
world.add_component(&entity, Transform2D::new(pos));
world.add_component(&entity, RigidBody::new_dynamic());
world.add_component(&entity, Collider::box_collider(32.0, 32.0));

// In game loop
physics.update(&mut world, delta_time);
```

---

## Physics Presets

### PhysicsConfig Presets
```rust
PhysicsConfig::platformer()   // Standard platformer (gravity -980)
PhysicsConfig::top_down()     // Top-down game (no gravity)
PhysicsConfig::low_gravity()  // Moon-like floaty physics
PhysicsConfig::space()        // Space (no gravity, low iterations)
```

### RigidBody Presets
```rust
RigidBody::player_platformer() // Rotation locked, CCD, proper damping
RigidBody::player_top_down()   // No gravity effect
RigidBody::pushable()          // Objects that can be pushed
RigidBody::physics_prop()      // Objects that tumble and roll
```

### Collider Presets
```rust
Collider::player_box()            // Standard player (80x80)
Collider::platform(width, height) // Platforms with high friction
Collider::pushable_box(w, h)      // Pushable with low friction
Collider::bouncy(width, height)   // Trampolines, bumpers
Collider::slippery(width, height) // Ice, oil
```

### MovementConfig Presets
```rust
MovementConfig::platformer()      // 120 px/s speed, 420 jump impulse
MovementConfig::platformer_fast() // Action games
MovementConfig::top_down()        // Top-down perspective
MovementConfig::floaty()          // Space/underwater feel
```

---

## Movement Patterns

### Velocity-Based Control (Recommended for All Movement)

`set_velocity` is the canonical "move this body" API. It is safe on bodies
that were just created (the velocity is buffered and applied once the body
syncs to Rapier). Use it for player input, jump impulses, projectile
launch, knockback — any "start this body at velocity V" case.

```rust
let current_vel = physics.physics_world()
    .get_body_velocity(player)
    .map(|(v, _)| v)
    .unwrap_or(Vec2::ZERO);

let target_vel_x = if input.is_key_pressed(KeyCode::KeyD) { 120.0 }
                   else if input.is_key_pressed(KeyCode::KeyA) { -120.0 }
                   else { 0.0 };

// Horizontal movement: preserve vertical velocity (gravity/jump).
physics.set_velocity(player, Vec2::new(target_vel_x, current_vel.y), 0.0);

// Jump: overwrite vertical velocity directly.
if input.is_key_just_pressed(KeyCode::Space) {
    physics.set_velocity(player, Vec2::new(current_vel.x, 420.0), 0.0);
}
```

### Force-Based Control
```rust
physics.apply_force(player, Vec2::new(force_x, 0.0));
```

### Mass-Aware Impulse (rare)
For genuine momentum-delta semantics on a live body, reach into the
physics world directly:
```rust
physics.physics_world_mut().apply_impulse(entity, impulse);
```

---

## Future Enhancements

Features that would enhance the physics system but are not required for
current functionality:

### Physics Features
- Comprehensive friction/restitution tests (see gap #1 above)
- Kinematic platform movement tests (see gap #2 above)
- Sensor/trigger volume tests (see gap #3 above)
- High-speed movement edge case testing (tunneling prevention via CCD
  is configured on the player preset but not validated under tests)
- Collision response validation and debugging tools

### Long-term
- Debug visualization for colliders
- Joint support (hinges, springs, distance joints)
- Physics layers/groups for selective collision (`InteractionGroups`)
- Performance optimization for large entity counts
- SRP refactor of `PhysicsWorld` — it currently owns many rapier types
  (body set, collider set, island manager, broad/narrow phase, pipeline,
  query pipeline, event handler). Splitting simulation state from query
  state would ease testing.

---

## Production Readiness Assessment

### Stable
- Dynamic, Static, Kinematic body types
- Box, Circle, Capsule collider shapes
- Collision detection with start/stop event flags
- Sensor intersection events
- Raycasting support
- Fixed timestep simulation with spiral-of-death guard
- Physics presets for common game types
- Unified velocity-based movement (`set_velocity`), force-based movement,
  and a mass-aware impulse escape hatch on `PhysicsWorld`
- Proper pixel<->meter conversion (100 px/m, public API)
- Clear/re-sync for editor play-mode lifecycle

### Gaps
- No friction/restitution simulation-level tests
- No kinematic platform movement tests
- No sensor trigger tests
- `PhysicsWorld` aggregates many rapier concerns (future SRP refactor)

---

## Conclusion

The physics crate provides **functional 2D physics** with good presets,
ECS integration, a unified velocity API, and editor-friendly reset
support. Test coverage is adequate for core functionality but missing
material and trigger scenarios.

**Status:** Production-ready for basic physics. Add friction, kinematic
platform, and sensor tests to validate advanced features.

Run `cargo run --example hello_world` for a physics platformer demo with
WASD movement, SPACE to jump, R to reset.
