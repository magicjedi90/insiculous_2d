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
| `GameContext` | Access `input`, `world` (ECS), `assets`, `audio`, `ui`, `delta_time`, `window_size` |
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
│   ├── engine_core/    # Game trait, application lifecycle, scenes, behaviors
│   ├── renderer/       # WGPU 28.0.0 sprite rendering with instancing
│   ├── ecs/            # Entity Component System (dual storage: HashMap + archetype)
│   ├── input/          # Keyboard, mouse, gamepad handling with action mapping
│   ├── physics/        # rapier2d 2D physics integration
│   ├── audio/          # Rodio-based sound and music playback
│   ├── ui/             # Immediate-mode UI with fontdue font rendering
│   ├── common/         # Shared types (colors, transforms, math utilities)
│   └── editor/         # Visual editor (in development)
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

### Physics Integration

```rust
use engine_core::prelude::*;

// Create physics entity
let player = world.create_entity();
world.add_component(&player, Transform2D::new(Vec2::ZERO))?;
world.add_component(&player, RigidBody::player_platformer())?;
world.add_component(&player, Collider::player_box())?;
world.add_component(&player, Sprite::new(0).with_color(Vec4::new(0.2, 0.4, 1.0, 1.0)))?;

// Physics system handles movement, gravity, collisions
```

### Audio System

```rust
// In init()
self.jump_sound = ctx.audio.load_sound("assets/jump.wav").ok();

// In update() - play sound
if ctx.input.is_key_just_pressed(KeyCode::Space) {
    if let Some(sound) = &self.jump_sound {
        ctx.audio.play_with_settings(sound, SoundSettings::default());
    }
}

// Background music with crossfade
ctx.audio.play_music("assets/music.ogg")?;
ctx.audio.set_volume(0.5);
```

### UI System

```rust
fn update(&mut self, ctx: &mut GameContext) {
    // Panel background
    ctx.ui.panel(Rect::new(10.0, 10.0, 200.0, 150.0));
    
    // Button
    if ctx.ui.button("play_btn", "Play", Rect::new(20.0, 30.0, 180.0, 40.0)) {
        self.start_game();
    }
    
    // Slider
    self.volume = ctx.ui.slider("volume_slider", self.volume, Rect::new(20.0, 90.0, 180.0, 20.0));
    
    // Label
    ctx.ui.label(&format!("Volume: {:.0}%", self.volume * 100.0), Vec2::new(20.0, 120.0));
}
```

### Scene Files (RON)

Load entities and components from scene files:

```rust
use engine_core::SceneLoader;
use std::path::Path;

fn init(&mut self, ctx: &mut GameContext) {
    let scene_path = Path::new("assets/scenes/level1.scene.ron");
    match SceneLoader::load_and_instantiate(scene_path, &mut ctx.world, ctx.assets) {
        Ok(instance) => {
            println!("Loaded scene '{}' with {} entities", instance.name, instance.entity_count);
            // Access named entities
            if let Some(player) = instance.get_entity("player") {
                // Work with player entity...
            }
        }
        Err(e) => eprintln!("Failed to load scene: {}", e),
    }
}
```

Example scene file format:
```ron
Scene(
    name: "Level 1",
    entities: [
        Entity(
            name: Some("player"),
            components: [
                Transform2D(
                    position: (0.0, 100.0),
                    rotation: 0.0,
                    scale: (1.0, 1.0),
                ),
                Sprite(
                    texture_handle: 0,
                    color: (0.2, 0.4, 1.0, 1.0),
                ),
                Behavior(PlayerPlatformer()),
            ],
        ),
    ],
)
```

### Entity Behaviors

Attach behaviors to entities for common game logic:

```rust
use ecs::behavior::Behavior;

// Player-controlled platformer (WASD + Space to jump)
world.add_component(&player, Behavior::PlayerPlatformer {
    move_speed: 120.0,
    jump_impulse: 420.0,
    jump_cooldown: 0.3,
    tag: "player".to_string(),
})?;

// AI that follows the player
world.add_component(&enemy, Behavior::ChaseTagged {
    target_tag: "player".to_string(),
    detection_range: 200.0,
    chase_speed: 80.0,
    lose_interest_range: 300.0,
})?;

// Process behaviors in update()
self.behaviors.update(&mut ctx.world, ctx.input, ctx.delta_time, self.physics.as_mut());
```

Available behaviors:
- `PlayerPlatformer` - WASD movement with jumping
- `PlayerTopDown` - WASD movement in all directions
- `ChaseTagged` - Chase entities with a specific tag
- `FollowEntity` - Follow a named entity
- `Patrol` - Patrol between two points
- `Collectible` - Items that can be collected

### Hierarchy/Scene Graph

Create parent-child relationships between entities:

```rust
use ecs::WorldHierarchyExt;

// Set parent-child relationship
world.set_parent(child_entity, parent_entity)?;

// Get children of an entity
let children = world.get_children(parent_entity);

// Get all descendants (children, grandchildren, etc.)
let descendants = world.get_descendants(root_entity);

// Get all root entities (no parent)
let roots = world.get_root_entities();

// Check for cycles (prevented automatically)
```

Child transforms are automatically updated when parent transforms change.

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

// Check action activation
if ctx.input.is_action_just_activated(&GameAction::Action1) {
    // Primary action triggered
}
```

## Test Summary

| Crate | Tests | Coverage |
|-------|-------|----------|
| ECS | 113 | Entity lifecycle, components, queries, hierarchy, systems |
| Input | 56 | Key states, action mapping, events, thread safety |
| Engine Core | 68 | Game API, lifecycle, scenes, behaviors, timing |
| Physics | 28 | Body types, colliders, simulation, presets |
| UI | 53 | Widgets, interaction, draw commands, font rendering |
| Audio | 3 | Sound loading, playback, settings |
| Renderer | 62 | Sprite batching, instancing, camera, textures |
| **Total** | **~383** | **All passing** |

Run all tests: `cargo test --workspace`

## Project Status

**Current State:** Production ready, performant

- **Test Status:** ~383 tests passing, 18 ignored (GPU/window tests), 0 failures
- **Test Quality:** 0 TODOs, 155+ meaningful assertions
- **Architecture:** Manager pattern implemented, game.rs SRP refactoring complete
- **Performance:** Behavior system optimized (-85% allocations)

See [PROJECT_ROADMAP.md](PROJECT_ROADMAP.md) for detailed technical debt tracking and priorities.

## Examples

- **hello_world** - Physics platformer with UI, audio, ECS, scene files, and behaviors
- **behavior_demo** - Demonstrates all built-in entity behaviors
- **editor_demo** - Visual editor for scene editing (in development)

Run an example:
```bash
cargo run --example hello_world
cargo run --example behavior_demo
```

## Design Patterns Used

This codebase implements patterns from [Robert Nystrom's Game Programming Patterns](https://gameprogrammingpatterns.com/):

| Pattern | Implementation |
|---------|---------------|
| **Component** | ECS with entity-component storage |
| **Update Method** | Single game loop with centralized update |
| **Command** | Input events and UI draw commands as data |
| **Manager Pattern** | GameLoopManager, UIManager, RenderManager, WindowManager |

## Requirements

- Rust stable (1.70+)
- Cargo
- GPU with Vulkan, Metal, or DX12 support

## License

See LICENSE file for details.
