# Insiculous 2D

A lightweight, modular 2D game engine for Rust. Focus on game logic, not boilerplate.

## Quick Start

```bash
git clone https://github.com/yourusername/insiculous_2d.git
cd insiculous_2d

# Run the example (2 sprites - move with WASD, ESC to exit)
cargo run --example hello_world
```

**Requirements:** Rust stable, Cargo, Git

## Architecture

```
insiculous_2d/
├── crates/
│   ├── engine_core/    # Application lifecycle, scenes, game loop
│   ├── renderer/       # WGPU 28.0.0 sprite rendering
│   ├── ecs/            # Archetype-based Entity Component System
│   └── input/          # Keyboard, mouse, gamepad handling
└── examples/           # Demo applications
```

**Data Flow:**
1. Input events captured by `InputHandler`
2. `EngineApplication` manages scene stack
3. Each `Scene` contains an ECS `World`
4. Systems update game state via `SystemRegistry`
5. `Renderer` draws sprites with WGPU

## Technical Implementation

### Renderer Crate

**WGPU 28.0.0 Sprite Rendering Pipeline:**
- Instanced rendering with automatic texture batching
- Orthographic 2D camera with view/projection matrices
- Built-in 1x1 white texture for colored sprites
- `TexelCopyBufferLayout` for texture uploads

```rust
// Create pipeline and batcher
let sprite_pipeline = SpritePipeline::new(renderer.device(), 1000);
let mut batcher = SpriteBatcher::new(1000);

// Add sprites
batcher.add_sprite(&Sprite::new(texture_handle)
    .with_position(Vec2::new(100.0, 200.0))
    .with_color(Vec4::new(1.0, 0.0, 0.0, 1.0))
    .with_scale(Vec2::new(64.0, 64.0)));

// Render
let batches: Vec<SpriteBatch> = batcher.batches().values().cloned().collect();
renderer.render_with_sprites(&mut sprite_pipeline, &camera, &textures, &batch_refs)?;
```

**Key Types:**
- `Renderer` - Surface management, device/queue
- `SpritePipeline` - Vertex/instance/index buffers, shaders
- `SpriteBatcher` - Groups sprites by texture handle
- `Camera2D` - Viewport, position, zoom, rotation

### ECS Crate

**Archetype-Based Component Storage:**
- Entities with same component types stored in dense arrays
- Cache-friendly iteration patterns
- Type-safe queries: `Single<T>`, `Pair<A,B>`, `Triple<A,B,C>`

```rust
// Legacy storage
let world = World::new();

// Optimized archetype storage
let world = World::new_optimized();

// Entity operations
let entity = world.create_entity();
world.add_component(entity, Position { x: 0.0, y: 0.0 })?;
world.add_component(entity, Velocity { dx: 1.0, dy: 0.0 })?;

// Type-safe queries
for (entity, (pos, vel)) in world.query::<Pair<Position, Velocity>>() {
    // Process entities with both Position and Velocity
}
```

**60 tests passing** - archetype storage, component queries, entity lifecycle

### Input Crate

**Unified Input Handling:**
- Event queue buffers inputs between frames
- Action mapping binds keys/buttons to game actions
- Thread-safe wrapper for concurrent access

```rust
let input = InputHandler::new();

// Process events from winit
input.handle_window_event(&event);

// Query state
if input.is_action_active(&GameAction::MoveLeft) {
    player.position.x -= speed;
}
if input.is_key_just_pressed(KeyCode::Space) {
    player.jump();
}

// Frame boundary
input.update(); // Clears "just pressed" states
```

**Default Bindings:** WASD movement, mouse, gamepad support

**56 tests passing** - key states, action mapping, event queue

### Engine Core Crate

**Application Lifecycle:**
- Scene stack with push/pop for game states
- Fixed timestep game loop with accumulator
- Proper initialization/shutdown phases

```rust
let scene = Scene::new("main");
let game_loop = GameLoop::new(GameLoopConfig::default());
let app = EngineApplication::new(scene, game_loop);

// Scene management
app.push_scene(pause_menu);
app.pop_scene();

// System registration
app.schedule.register(PhysicsSystem);
app.schedule.register(RenderSystem);
```

**Lifecycle States:** `Uninitialized → Initialized → Started → Running → Stopped → Shutdown`

**29 tests passing** - lifecycle transitions, scene management, error recovery

## Test Summary

| Crate | Tests | Coverage |
|-------|-------|----------|
| ECS | 60 | Archetype storage, queries, entity lifecycle |
| Input | 56 | Key states, action mapping, events |
| Engine Core | 29 | Lifecycle, scenes, error recovery |
| Renderer | 0 | Visual testing via examples |

Run all tests: `cargo test --workspace`

## Contributing

### Standards
- Single Responsibility Principle (SRP)
- Don't Repeat Yourself (DRY)
- Descriptive names for all identifiers
- Document public APIs

### Commits
```
<type>(<scope>): <description>
```
Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

Example: `feat(renderer): add sprite batching system`

### Pull Requests
- [ ] Code follows project standards
- [ ] Tests added/updated
- [ ] Documentation updated
- [ ] Tested on multiple platforms

## License

See LICENSE file for details.
