# Physics Crate Analysis

## Current State (Updated: January 2026)
The physics crate provides 2D physics simulation for the Insiculous 2D game engine using rapier2d as the underlying physics engine.

**Test Count: 22 tests** (all passing)

---

## Critical Issues Identified

### Medium Severity

#### 1. ~~Dead Code in PhysicsWorld~~ - FIXED (January 2026)
**Location**: `src/world.rs`
**Status**: RESOLVED - Coordinate conversion methods (`pixels_to_meters_scalar`, `meters_to_pixels`, `meters_to_pixels_scalar`) are now public API methods, removing dead code warnings and making them available for game developers.

#### 2. No Tests for Friction/Restitution Values
**Location**: Test suite
**Issue**: No tests verify that friction and restitution (bounciness) values are applied correctly.

**Impact**: Physics material behavior unvalidated.

**Recommended Fix**: Add tests for:
- High friction surfaces (player sticks to walls)
- Low friction surfaces (ice/oil sliding)
- Bouncy surfaces (trampolines)
- Zero restitution (no bounce)

#### 3. No Kinematic Body Tests
**Location**: Test suite
**Issue**: No tests for kinematic body behavior (platform moving independently).

**Impact**: Moving platform behavior unvalidated.

**Recommended Fix**: Add tests for:
- Kinematic body movement
- Collision with dynamic bodies
- Position/rotation updates

#### 4. No Sensor Trigger Tests
**Location**: Test suite
**Issue**: No tests for sensor colliders (trigger areas without physical response).

**Impact**: Collectible/trigger area behavior unvalidated.

**Recommended Fix**: Add tests for:
- Sensor detection without physical collision
- Enter/exit trigger events
- Multiple entities in sensor area

---

## Test Coverage Analysis

**Total Tests**: 22 (all passing)

### Test Breakdown
```
src/
├── lib.rs:        2 tests (basic integration)
├── components.rs: 5 tests (RigidBody, Collider creation)
├── world.rs:      6 tests (PhysicsWorld operations)
├── system.rs:     5 tests (PhysicsSystem simulation)
└── presets.rs:    4 tests (preset configurations)
```

### Test Quality Assessment

**Strengths:**
- Good coverage of component creation patterns
- Physics world operations tested
- Preset configurations validated

**Gaps:**
- No friction/restitution tests
- No kinematic body tests
- No sensor trigger tests
- No collision response validation
- No edge case testing (high speed, tunneling)

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
  - Sensor mode for trigger areas
  - Collision groups/filters

- **CollisionEvent/CollisionData**: Collision event information
  - Entity pairs
  - Contact points with normals and depth

### Physics World (`world.rs`)
Wrapper around rapier2d that manages:
- Rigid body and collider sets
- Physics pipeline stepping
- Entity to rapier handle mapping
- Coordinate conversion (pixels <-> meters)
- Collision event collection
- Raycasting

**Key design decisions:**
- Uses 100 pixels per meter scale factor
- Fixed timestep simulation (1/60s default)
- Automatic handle cleanup when entities removed

### Physics System (`system.rs`)
ECS system that:
- Syncs ECS components to rapier physics world
- Steps physics simulation with fixed timestep
- Syncs physics results back to ECS transforms
- Provides collision callbacks

---

## Integration with Engine

1. **Optional Dependency**: Physics is a default feature that can be disabled
2. **Prelude Export**: Physics types available via `engine_core::prelude::*`
3. **ECS Integration**: Components work with existing Transform2D

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
Collider::player_box()           // Standard player (80x80)
Collider::platform(width, height) // Platforms with high friction
Collider::pushable_box(w, h)     // Pushable with low friction
Collider::bouncy(width, height)  // Trampolines, bumpers
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

### Velocity-Based Control (Recommended for Platformers)
```rust
let current_vel = physics.physics_world().get_body_velocity(player);

let target_vel_x = if input.is_key_pressed(KeyCode::KeyD) { 120.0 }
                   else if input.is_key_pressed(KeyCode::KeyA) { -120.0 }
                   else { 0.0 };

physics.physics_world_mut().set_body_velocity(
    player,
    Vec2::new(target_vel_x, current_vel.y),
    0.0
);

if input.is_key_pressed(KeyCode::Space) {
    physics.apply_impulse(player, Vec2::new(0.0, 420.0));
}
```

### Force-Based Control
```rust
physics.apply_force(player, Vec2::new(force_x, 0.0));
```

---

## ~~Dead Code Identified~~ - FIXED (January 2026)

### #[allow(dead_code)] Suppressions - RESOLVED

| Location | Code | Status |
|----------|------|--------|
| ~~`world.rs:166`~~ | `pixels_to_meters_scalar()` | Now public API |
| ~~`world.rs:172`~~ | `meters_to_pixels()` | Now public API |
| ~~`world.rs:178`~~ | `meters_to_pixels_scalar()` | Now public API |

**Resolution**: These coordinate conversion methods are now part of the public API for `PhysicsWorld`, allowing game developers to convert between pixel and meter coordinates when needed.

---

## Recommended Fixes (Priority Order)

### Immediate (High Priority)
1. Add friction/restitution tests
2. Add kinematic body tests
3. Add sensor trigger tests

### Short-term (Medium Priority)
4. Review and remove/use dead code in world.rs
5. Add collision response validation tests
6. Add high-speed (tunneling) edge case tests

### Long-term (Features)
7. Debug visualization for colliders
8. Joint support (hinges, springs)
9. Physics layers/groups for selective collision
10. Performance optimization for large entity counts

---

## Production Readiness Assessment

### Stable
- Dynamic, Static, Kinematic body types
- Box, Circle, Capsule collider shapes
- Collision detection and response
- Raycasting support
- Fixed timestep simulation
- Physics presets for common game types
- Velocity-based and force-based movement
- Proper pixel<->meter conversion (100 px/m)

### Gaps
- No friction/restitution tests
- No kinematic body tests
- No sensor trigger tests
- Some dead code to review

---

## Conclusion

The physics crate provides **functional 2D physics** with good presets and ECS integration. Test coverage is adequate for core functionality but missing important edge cases.

**Status**: Production-ready for basic physics. Add friction, kinematic, and sensor tests to validate advanced features.

Run `cargo run --example hello_world` for a physics platformer demo with WASD movement, SPACE to jump, R to reset.
