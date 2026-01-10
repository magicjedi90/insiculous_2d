# Physics Crate Analysis

## Overview

The physics crate provides 2D physics simulation for the Insiculous 2D game engine using rapier2d as the underlying physics engine.

## Architecture

### Components (`components.rs`)

The physics components integrate with the ECS system:

- **RigidBody**: Defines physical properties of an entity
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

Key design decisions:
- Uses 100 pixels per meter scale factor
- Fixed timestep simulation (1/60s default)
- Automatic handle cleanup when entities removed

### Physics System (`system.rs`)

ECS system that:
- Syncs ECS components to rapier physics world
- Steps physics simulation with fixed timestep
- Syncs physics results back to ECS transforms
- Provides collision callbacks

## Integration with Engine

The physics crate integrates with engine_core:

1. **Optional Dependency**: Physics is a default feature that can be disabled
2. **Prelude Export**: Physics types available via `engine_core::prelude::*`
3. **ECS Integration**: Components work with existing Transform2D

## Usage Pattern

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

## Tests

18 unit tests covering:
- Component creation and builder patterns
- Physics world operations
- Simulation stepping
- Raycasting
- ECS integration

## Movement Patterns

### Velocity-Based Control (Recommended for Platformers)
For precise, responsive movement, set velocity directly rather than applying forces:

```rust
// Get current velocity to preserve vertical component
let current_vel = physics.physics_world().get_body_velocity(player);

// Set horizontal velocity directly (e.g., 120 px/s = 2 pixels/frame at 60 FPS)
let target_vel_x = if input.is_key_pressed(KeyCode::KeyD) { 120.0 }
                   else if input.is_key_pressed(KeyCode::KeyA) { -120.0 }
                   else { 0.0 };

physics.physics_world_mut().set_body_velocity(
    player,
    Vec2::new(target_vel_x, current_vel.y),  // Preserve vertical velocity
    0.0
);

// Use impulses for jumping (instantaneous velocity change)
if input.is_key_pressed(KeyCode::Space) {
    physics.apply_impulse(player, Vec2::new(0.0, 420.0));
}
```

### Force-Based Control
For physics-driven movement with momentum, use forces with appropriate damping:

```rust
// Higher damping = quicker stops, but needs higher force to overcome
// Terminal velocity ≈ force / (damping * pixels_per_meter)
physics.apply_force(player, Vec2::new(force_x, 0.0));
```

## Collider Sizing

Collider dimensions should match sprite visual sizes. The default sprite renders at 80×80 pixels (1.0 scale × 80), so:

- Default entity: `Collider::box_collider(80.0, 80.0)`
- Scaled entity (e.g., ground with scale 10.0×0.5): `Collider::box_collider(800.0, 40.0)`

## Physics Presets

The `presets` module provides tested, ready-to-use configurations so developers don't have to guess at values.

### PhysicsConfig Presets
```rust
// Standard platformer (gravity -980, high solver iterations)
PhysicsConfig::platformer()

// Top-down game (no gravity)
PhysicsConfig::top_down()

// Moon-like floaty physics
PhysicsConfig::low_gravity()

// Space (no gravity, low iterations)
PhysicsConfig::space()
```

### RigidBody Presets
```rust
// Player body with rotation locked, CCD, and proper damping
RigidBody::player_platformer()
RigidBody::player_top_down()

// Objects that can be pushed around
RigidBody::pushable()

// Objects that tumble and roll
RigidBody::physics_prop()
```

### Collider Presets
```rust
// Standard player (80x80, matches default sprite)
Collider::player_box()

// Platforms with high friction
Collider::platform(width, height)

// Pushable objects with low friction
Collider::pushable_box(width, height)

// Special surfaces
Collider::bouncy(width, height)   // Trampolines, bumpers
Collider::slippery(width, height) // Ice, oil
```

### MovementConfig Presets
```rust
// Get tested movement values
let movement = MovementConfig::platformer();
let speed = movement.move_speed;     // 120 px/s (~2 pixels/frame)
let jump = movement.jump_impulse;    // 420 (satisfying jump height)
let damping = movement.damping;      // 5.0 (responsive stops)

// Other presets
MovementConfig::platformer_fast()  // Action games
MovementConfig::top_down()         // Top-down perspective
MovementConfig::floaty()           // Space/underwater feel
```

## Future Improvements

- Debug visualization for colliders
- Joint support (hinges, springs)
- Physics layers/groups for selective collision
- Performance optimization for large entity counts
