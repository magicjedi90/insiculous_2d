# Physics Crate — Agent Context

You are working in the physics crate. Rapier2d integration with ECS components and presets.

## Architecture
```
PhysicsSystem
├── PhysicsWorld (rapier2d wrapper)
│   ├── RigidBodySet, ColliderSet
│   ├── IntegrationParameters
│   └── PhysicsPipeline
├── ECS sync: Transform2D ↔ RigidBody position (each frame)
└── Collision callbacks (multiple listeners supported)
```

## Update Flow
1. `PhysicsSystem::update(world, delta_time)`
2. Sync ECS Transform2D → rapier body positions
3. Step rapier simulation
4. Sync rapier body positions → ECS Transform2D
5. Fire collision callbacks for all contact events

## File Map
- `lib.rs` — PhysicsSystem, public exports
- `physics_world.rs` — Rapier2d world wrapper
- `physics_system.rs` — ECS update system, sync logic
- `components.rs` — RigidBody, Collider ECS components
- `presets.rs` — Pre-configured physics: `RigidBody::player_platformer()`, `Collider::platform(w, h)`, etc.

## Key Patterns
- All rapier types stay inside `PhysicsWorld` — ECS components are our own types
- Body handles stored in RigidBody component for rapier lookup
- Multiple collision callbacks: `physics.add_collision_callback(|c| { ... })`
- Presets: `player_platformer()`, `pushable()`, `platform()`, `bouncy()`

## Known Tech Debt
- Missing friction/kinematic/sensor tests
- PhysicsWorld handles too many rapier types (future SRP refactor)

## Testing
- 28 tests, run with `cargo test -p physics`
- Pure math/simulation — no GPU needed

## Godot Oracle — When Stuck
Use `WebFetch` to read from `https://github.com/godotengine/godot/blob/master/`

| Our Concept | Godot Equivalent | File |
|-------------|-----------------|------|
| PhysicsSystem::update | Physics step | `servers/physics_2d/godot_step_2d.cpp` — `step` |
| PhysicsWorld | Physics server | `servers/physics_2d/godot_physics_server_2d.cpp` |
| RigidBody presets | Body types | `scene/2d/physics_body_2d.cpp` — RigidBody2D, CharacterBody2D |
| Collider (is_sensor) | Area2D | `scene/2d/area_2d.cpp` — overlap detection |
| Collision callbacks | Contact monitoring | `scene/2d/physics_body_2d.cpp` — `_body_enter_tree` |
| Broad-phase | BVH broad-phase | `servers/physics_2d/godot_broad_phase_2d.cpp` |

**Remember:** We use Rapier2d — study Godot's *API design* and *body type organization*, not its solver.
