# Insiculous 2D

A lightweight, modular 2D game engine for Rust. Focus on game logic, not boilerplate.

## Quick Start

```bash
git clone https://github.com/yourusername/insiculous_2d.git
cd insiculous_2d
cargo run --example hello_world
```

## Create Your First Game

```rust
use engine_core::prelude::*;

struct MyGame {
    player: Option<EntityId>,
}

impl Game for MyGame {
    fn init(&mut self, ctx: &mut GameContext) {
        // Create a player entity with position and sprite
        let player = ctx.world.create_entity();
        ctx.world.add_component(&player, Transform2D::new(Vec2::new(0.0, 0.0))).ok();
        ctx.world.add_component(&player, Sprite::new(0).with_color(Vec4::new(0.2, 0.4, 1.0, 1.0))).ok();
        self.player = Some(player);
    }

    fn update(&mut self, ctx: &mut GameContext) {
        // Move player with WASD
        if let Some(player) = self.player {
            if let Some(transform) = ctx.world.get_mut::<Transform2D>(player) {
                let speed = 200.0 * ctx.delta_time;
                if ctx.input.is_key_pressed(KeyCode::KeyW) { transform.position.y += speed; }
                if ctx.input.is_key_pressed(KeyCode::KeyS) { transform.position.y -= speed; }
                if ctx.input.is_key_pressed(KeyCode::KeyA) { transform.position.x -= speed; }
                if ctx.input.is_key_pressed(KeyCode::KeyD) { transform.position.x += speed; }
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    run_game(MyGame { player: None }, GameConfig::new("My Game"))
}
```

That's it! The engine handles window creation, event loops, rendering, and input automatically.

## Game API Reference

| Type | Purpose |
|------|---------|
| `Game` | Trait to implement - `init()`, `update()`, optionally `render()` |
| `GameConfig` | Window settings: title, size, clear color, FPS |
| `GameContext` | Access `input`, `world` (ECS), `delta_time`, `window_size` |
| `run_game()` | Single function to start everything |

### GameConfig Options

```rust
let config = GameConfig::new("My Game")
    .with_size(1280, 720)           // Window dimensions
    .with_clear_color(0.1, 0.1, 0.2, 1.0)  // Background color
    .with_fps(60);                   // Target frame rate
```

### Game Trait Methods

```rust
impl Game for MyGame {
    // Required: called every frame
    fn update(&mut self, ctx: &mut GameContext) { }

    // Optional: called once at startup
    fn init(&mut self, ctx: &mut GameContext) { }

    // Optional: custom sprite rendering (default extracts from ECS)
    fn render(&mut self, ctx: &mut RenderContext) { }

    // Optional: key event callbacks
    fn on_key_pressed(&mut self, key: KeyCode, ctx: &mut GameContext) { }
    fn on_key_released(&mut self, key: KeyCode, ctx: &mut GameContext) { }

    // Optional: window events
    fn on_resize(&mut self, width: u32, height: u32) { }
    fn on_exit(&mut self) { }
}
```

## Architecture

```
insiculous_2d/
├── crates/
│   ├── engine_core/    # Game trait, application lifecycle, scenes
│   ├── renderer/       # WGPU 28.0.0 sprite rendering
│   ├── ecs/            # Archetype-based Entity Component System
│   └── input/          # Keyboard, mouse, gamepad handling
└── examples/           # Demo applications
```

## Advanced Usage

For more control, you can use the lower-level APIs directly.

### Direct ECS Access

```rust
// Create entities with components
let entity = world.create_entity();
world.add_component(&entity, Transform2D::new(Vec2::new(100.0, 200.0)))?;
world.add_component(&entity, Sprite::new(0).with_color(Vec4::ONE))?;

// Query components
if let Some(transform) = world.get_mut::<Transform2D>(entity) {
    transform.position.x += 10.0;
}

// Iterate all entities
for entity_id in world.entities() {
    if let Some(sprite) = world.get::<Sprite>(entity_id) {
        // Process sprite...
    }
}
```

### Custom Rendering

Override `render()` to control sprite rendering:

```rust
fn render(&mut self, ctx: &mut RenderContext) {
    // Add sprites manually
    let sprite = renderer::Sprite::new(TextureHandle { id: 0 })
        .with_position(Vec2::new(100.0, 100.0))
        .with_scale(Vec2::new(64.0, 64.0))
        .with_color(Vec4::new(1.0, 0.0, 0.0, 1.0));

    ctx.sprites.add_sprite(&sprite);

    // Or extract from specific entities
    if let Some(transform) = ctx.world.get::<Transform2D>(self.player) {
        // Custom sprite logic...
    }
}
```

### Input System

```rust
// In update()
if ctx.input.is_key_pressed(KeyCode::Space) {
    // Key is held down
}
if ctx.input.is_key_just_pressed(KeyCode::Enter) {
    // Key was just pressed this frame
}

// Mouse position
let mouse_pos = ctx.input.mouse_position();

// Action-based input (with default WASD bindings)
if ctx.input.is_action_active(&GameAction::MoveLeft) {
    // Move left
}
```

### Manual Application Control

For full control over the event loop:

```rust
use winit::event_loop::EventLoop;
use engine_core::EngineApplication;

let event_loop = EventLoop::new()?;
let scene = Scene::new("main");
let mut app = EngineApplication::with_scene(scene);
event_loop.run_app(&mut app)?;
```

## Test Summary

| Crate | Tests | Coverage |
|-------|-------|----------|
| ECS | 60 | Archetype storage, queries, entity lifecycle |
| Input | 56 | Key states, action mapping, events |
| Engine Core | 29 | Lifecycle, scenes, game API |
| Renderer | 0 | Visual testing via examples |

Run all tests: `cargo test --workspace`

## Requirements

- Rust stable (1.70+)
- Cargo
- GPU with Vulkan, Metal, or DX12 support

## License

See LICENSE file for details.
