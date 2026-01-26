# Insiculous 2D - AI Pair Programming Guide

## Context
Insiculous 2D is a lightweight, modular game engine designed for creating 2D games with Rust. It aims to provide a simple yet powerful API that allows developers to focus on game logic rather than boilerplate code. The engine prioritizes performance, cross-platform compatibility, and a clean, intuitive interface. The architecture follows a component-based design with clear separation of concerns between systems.

## Directory Map
- **crates/engine_core/** - Core functionality including Game API, lifecycle management, and scene coordination
  - `game.rs` - **Simple Game API** - `Game` trait, `GameConfig`, `run_game()` orchestration
  - `game_loop_manager.rs` - Game loop timing and frame delta management (NEW)
  - `ui_manager.rs` - UI lifecycle and draw command collection (NEW)
  - `render_manager.rs` - Renderer lifecycle and sprite pipeline management
  - `window_manager.rs` - Window creation and size tracking
  - `assets.rs` - Asset loading and management (textures, fonts, etc.)
  - `scene.rs` - Scene lifecycle management and world coordination
  - `scene_loader.rs` - Scene deserialization from RON files
  - `scene_data.rs` - Scene data structures (prefabs, entities, components)
  - `behavior_runner.rs` - Behavior system for entity logic
  - `lifecycle.rs` - FSM for scene lifecycle management
  - `timing.rs` - Timer utilities for frame timing and delta calculation
  - `application.rs` - Deprecated - use Game API instead

- **crates/renderer/** - WGPU-based rendering system with instancing and batching
  - `renderer.rs` - Core WGPU rendering functionality
  - `sprite.rs` - Sprite pipeline with instanced rendering and batching
  - `sprite_data.rs` - GPU data structures (Vertex, Instance, Camera uniforms)
  - `texture.rs` - Texture manager with caching and atlas building
  - `shader/` - WGSL shaders for sprite rendering

- **crates/ecs/** - Entity Component System for game object management
  - `lib.rs` - ECS world, entity management, component storage
  - `component.rs` - Component trait and archetype-based storage
  - `hierarchy_system.rs` - Parent-child transform propagation
  - `sprite_components.rs` - Sprite, Transform2D, Camera2D components

- **crates/input/** - Comprehensive input handling with event queuing
  - `lib.rs` - InputHandler facade and thread-safe wrapper
  - `input_handler.rs` - Main input state and event processing
  - `input_mapping.rs` - Action-based input binding system
  - `keyboard.rs` - Keyboard state (pressed, just_pressed, just_released)
  - `mouse.rs` - Mouse position, buttons, wheel tracking
  - `gamepad.rs` - Gamepad button and axis management
  - `thread_safe_input.rs` - Arc<Mutex<>> wrapper for multi-threaded access

- **crates/ui/** - Immediate-mode UI framework
  - `lib.rs` - UIContext, widget creation, theming
  - `context.rs` - Immediate-mode UI entry point
  - `font.rs` - Font loading and glyph rasterization via fontdue
  - `draw.rs` - Draw command generation
  - `interaction.rs` - Widget state and mouse interaction
  - `rect.rs` - Rectangle utilities for layout and hit detection
  - `style.rs` - Color and Theme definitions

- **crates/physics/** - Rapier2d-based 2D physics integration
  - `lib.rs` - Physics system integration with ECS
  - `physics_world.rs` - Rapier2d world wrapper
  - `physics_system.rs` - ECS system for physics updates
  - `components.rs` - RigidBody, Collider ECS components
  - `presets.rs` - Pre-configured physics presets for common scenarios

- **crates/audio/** - Audio playback via rodio
  - `lib.rs` - AudioManager facade
  - `manager.rs` - Sound loading, playback, volume control
  - `ecs/audio_components.rs` - AudioSource, AudioListener, PlaySoundEffect

- **crates/common/** - Shared types and utilities
  - `math.rs` - Common mathematical constants and utilities

- **examples/** - Working demonstrations of engine features
  - `hello_world.rs` - Physics platformer with UI, audio, ECS, and scene graph

## Coding Guidelines

### Single Responsibility Principle (SRP)
Each module, struct, and function should have a single, well-defined responsibility. For example, the `Renderer` handles graphics rendering, while `InputHandler` manages input, with no overlap between them.

### Don't Repeat Yourself (DRY)
Extract common functionality into reusable components. Use traits for shared behavior and create utility functions for repeated operations.

### Explicit Naming
Use descriptive names that clearly communicate purpose:
- Functions should be verbs: `start()`, `update()`, `render()`
- Boolean methods should be questions: `is_running()`, `has_component()`
- Getters should use noun form: `device()`, `keyboard()`
- Mutable getters should use `_mut` suffix: `keyboard_mut()`

### Rust Idioms
- Prefer `impl From<T>` over custom conversion functions
- Use `Option<T>` for values that might be absent
- Use `Result<T, E>` for operations that might fail
- Implement traits like `Default` where appropriate
- Use `#[derive]` for common traits when possible
- Prefer ownership with borrowing over raw pointers
- Use strong typing over stringly-typed code

## Patterns Reference

### Manager Pattern (NEW - January 2026)
Extracts responsibilities from monolithic classes into focused managers following SRP:

```rust
// BEFORE: GameRunner had 15+ responsibilities
// AFTER: Responsibilities distributed to focused managers

// GameLoopManager - Frame timing and delta calculation
let mut loop_mgr = GameLoopManager::new();
let delta_time = loop_mgr.update();

// UIManager - UI lifecycle and draw commands  
let mut ui_mgr = UIManager::new();
ui_mgr.begin_frame(&input, window_size);
let commands = ui_mgr.end_frame();

// RenderManager - Renderer lifecycle
let mut render_mgr = RenderManager::new();
render_mgr.init(&window, clear_color)?;
render_mgr.render(&batches, &textures)?;

// WindowManager - Window management
let mut win_mgr = WindowManager::new(config);
let window = win_mgr.create(event_loop)?;
```

**Files:** `game_loop_manager.rs`, `ui_manager.rs`, `render_manager.rs`, `window_manager.rs`

### Simple Game API Pattern
The primary way to create games. Implement the `Game` trait:

```rust
use engine_core::prelude::*;

struct MyGame;

impl Game for MyGame {
    fn init(&mut self, ctx: &mut GameContext) {
        // Load assets, create entities, set initial state
        let player = ctx.world.create_entity();
        ctx.world.add_component(&player, Transform2D::new(Vec2::ZERO)).ok();
        ctx.world.add_component(&player, Sprite::new(0)).ok();
        
        // Load textures
        let player_tex = ctx.assets.load_texture("player.png")?;
    }

    fn update(&mut self, ctx: &mut GameContext) {
        // Game logic - run every frame
        // Access input, update entities, handle collisions
        if ctx.input.is_key_pressed(KeyCode::KeyD) {
            // Move player
        }
        
        // UI
        ctx.ui.label("Score: 100", Vec2::new(10.0, 10.0));
        if ctx.ui.button("play_btn", "Play", button_rect) {
            // Button clicked
        }
    }
}

fn main() {
    run_game(MyGame, GameConfig::new("My Game")).unwrap();
}
```

**Files:** `game.rs`, `game_loop.rs`, `GameContext` struct

### Archetype-Based ECS Pattern
High-performance component storage using archetypes:

```rust
// Components are stored densely by archetype
world.add_component(&entity, Transform2D::new(pos)).ok();
world.add_component(&entity, Sprite::new(texture_handle)).ok();

// Type-safe queries - get entities with specific components
use ecs::{Single, Pair, Triple};
let transforms = world.query_entities::<Single<Transform2D>>();
let sprites_with_transform = world.query_entities::<Pair<Transform2D, Sprite>>();
let full_entities = world.query_entities::<Triple<Transform2D, Sprite, RigidBody>>();

// Type-safe component access
if let Some((transform, sprite)) = world.get_two_mut::<(Transform2D, Sprite)>(entity) {
    transform.position += sprite.velocity * delta_time;
}

// Efficient entity iteration
for entity in world.entities() {
    // Process entities
}

// Hierarchy methods via WorldHierarchyExt trait
use ecs::WorldHierarchyExt;
world.set_parent(child, parent)?;
let children = world.get_children(parent);
let descendants = world.get_descendants(root);
let roots = world.get_root_entities();
```

**Files:** `ecs/lib.rs`, `ecs/component.rs`, `ecs/entity.rs`, `ecs/world.rs`, `ecs/hierarchy_ext.rs`

### Immediate-Mode UI Pattern
UI described every frame rather than retained state:

```rust
// Begin frame
ui.begin_frame(&input, window_size);

// Create UI elements
ui.panel(UIRect::new(10.0, 10.0, 200.0, 150.0));
ui.label("Volume:", Vec2::new(20.0, 30.0));
let volume = ui.slider("volume", current_volume, slider_rect);

// If button clicked, returns true
if ui.button("play", "Play", button_rect) {
    // Handle click
}

// End frame (collects draw commands)
ui.end_frame();
```

**Files:** `ui/lib.rs`, `ui/context.rs`, `ui/draw.rs`

### Asset Manager Pattern
Centralized asset loading with caching:

```rust
// Load textures
let texture = ctx.assets.load_texture("player.png")?;

// Create programmatic textures
let white = ctx.assets.create_solid_color(1, 1, [255, 255, 255, 255])?;
let checkerboard = ctx.assets.create_checkerboard(
    64, 64, [100, 100, 100, 255], [150, 150, 150, 255], 8
)?;

// Use assets in ECS
world.add_component(&entity, Sprite::new(texture.id)).ok();
```

**Files:** `assets.rs`, `texture.rs`

### Generic Component Inspector Pattern
Display any Serialize component without hardcoding field display logic:

```rust
use editor::inspector::{inspect_component, InspectorStyle};

// Works with any component that implements Serialize
let style = InspectorStyle::default();

// Display component fields automatically
if let Some(transform) = world.get::<Transform2D>(entity) {
    y = inspect_component(ui, "Transform2D", transform, x, y, &style);
}
if let Some(sprite) = world.get::<Sprite>(entity) {
    y = inspect_component(ui, "Sprite", sprite, x, y, &style);
}

// Inspector automatically:
// - Extracts fields via JSON serialization (serde)
// - Handles nested objects (Vec2, Vec3, Vec4)
// - Displays arrays with count and inline formatting
// - Formats floats with 2 decimal precision
// - Recursively renders complex types with indentation
```

**Benefits:**
- Single implementation handles all component types
- New components automatically work in inspector
- No per-component display code needed

**Files:** `editor/src/inspector.rs`

### Scene Serialization Pattern
Load entire game levels from RON files:

```rust
// Scene definition (hello_world.scene.ron)
SceneData(
    name: "Hello World",
    physics: Some(PhysicsSettings(
        gravity: (0.0, -980.0),
        pixels_per_meter: 100.0,
    )),
    prefabs: {
        "Player": PrefabData(
            components: [
                Transform2D(position: (0.0, 0.0)),
                Sprite(texture: "#white", color: (0.2, 0.4, 1.0, 1.0)),
                RigidBody(body_type: Dynamic, linear_damping: 5.0, can_rotate: false),
            ],
        ),
    },
    entities: [
        EntityData(
            name: Some("player"),
            prefab: Some("Player"),
            overrides: [Transform2D(position: (-200.0, 100.0))],
        ),
    ],
)

// Load in code
SceneLoader::load_and_instantiate("scene.ron", &mut world, &mut assets)?;
```

**Files:** `scene_loader.rs`, `scene_data.rs`, `assets/scenes/*.ron`

### Physics Integration Pattern
ECS-friendly 2D physics via rapier2d:

```rust
// Add physics components
world.add_component(&entity, RigidBody::player_platformer()).ok();
world.add_component(&entity,
    Collider::player_box()).ok()
)?;

// Physics presets for common scenarios
RigidBody::player_platformer()      // Dynamic with damping
RigidBody::pushable()               // Dynamic, can be pushed
Collider::platform(width, height)   // Static ground/platform
Collider::bouncy()                  // High restitution

// Multiple collision callbacks (all listeners receive each collision)
let mut physics = PhysicsSystem::new()
    .with_collision_callback(|collision| {
        println!("Audio: collision!");
    })
    .with_collision_callback(|collision| {
        println!("Particles: spawn sparks!");
    });

// Or add callbacks later
physics.add_collision_callback(|collision| {
    println!("Score system: check for pickup!");
});

// Update physics
physics_system.update(&mut world, delta_time)?;
```

**Files:** `physics/lib.rs`, `physics/components.rs`, `physics/presets.rs`, `physics/physics_system.rs`

### Input Mapping Pattern
Action-based input bindings:

```rust
// Define actions
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum GameAction {
    MoveUp,
    Jump,
    Shoot,
}

// Bind inputs to actions
let mut input = InputHandler::new();
input.bind_action(GameAction::MoveUp, InputSource::Key(KeyCode::KeyW));
input.bind_action(GameAction::MoveUp, InputSource::Key(KeyCode::ArrowUp));
input.bind_action(GameAction::Jump, InputSource::Key(KeyCode::Space));

// Check actions (instead of raw inputs)
if input.is_action_active(&GameAction::Jump) {
    // Player jumped
}
```

**Files:** `input/input_mapping.rs`, `input/input_handler.rs`

### Resource Acquisition Is Initialization (RAII)
Used throughout for resource management:

```rust
// WebGPU resources managed via Arc
let texture: Arc<Texture> = device.create_texture(&desc);
let view: Arc<TextureView> = texture.create_view(&Default::default());

// Auto-cleanup when last Arc dropped
drop(view); // Texture still alive if others hold Arc
drop(texture); // Now fully cleaned up
```

**Files:** `renderer/renderer.rs`, `renderer/texture.rs`

### Component Registry Pattern
Unified component definition with metadata for scene serialization and editor inspection:

```rust
// Define components with automatic derives and defaults
define_component! {
    pub struct Health {
        pub value: f32 = 100.0,
        pub max: f32 = 100.0,
    }
}

// ComponentMeta trait auto-implemented - provides runtime type info
assert_eq!(Health::type_name(), "Health");
assert_eq!(Health::field_names(), &["value", "max"]);

// Global registry for type lookup by name (built-ins registered at startup)
let registry = global_registry();
assert!(registry.is_registered("Transform2D"));
assert!(registry.is_registered("Sprite"));
```

**Built-in Components with ComponentMeta:**
- `Transform2D` - position, rotation, scale
- `Sprite` - texture_handle, offset, rotation, scale, color, depth, tex_region
- `Camera` - position, rotation, zoom, viewport_size, is_main_camera, near, far
- `SpriteAnimation` - fps, frames, playing, loop_animation, current_frame, time_accumulator

**Files:** `ecs/src/component_registry.rs`, `ecs/src/sprite_components.rs`

## Current Known Limitations (Updated January 2026)

**Technical Debt Tracking:**
- ~~SRP violations in GameRunner~~ ✅ FIXED: Managers extracted, game.rs refactored
- ~~Bind groups created per frame~~ ✅ FIXED: Camera bind group cached, texture bind groups cached per handle
- Glyph texture cache includes color in key (memory waste)
- ~~First-frame UI placeholder flicker~~ ✅ FIXED: Font rendering bug fixed
- ~~40+ allocations per frame in behavior system~~ ✅ FIXED: Behaviors accessed by reference
- Component registration still requires separate ComponentMeta impl (macro only handles struct definition)

**All tracked in:** `PROJECT_ROADMAP.md` Technical Debt section

---

## Quick Reference

### File Structure
```
/home/jedi/RustroverProjects/insiculous_2d/
├── AGENTS.md                    # Main project documentation
├── PROJECT_ROADMAP.md          # Updated with verified test counts
├── README.md                   # Project overview
├── TEST_IMPROVEMENTS.md        # Test enhancement documentation
├── training.md                 # This file - AI pair programming guide
├── crates/
│   ├── engine_core/           # Core engine (game.rs, managers, lifecycle)
│   ├── renderer/              # WGPU rendering (sprite.rs, texture.rs)
│   ├── ecs/                   # Entity Component System
│   ├── input/                 # Input handling with event queuing
│   ├── ui/                    # Immediate-mode UI framework
│   ├── physics/               # Rapier2d physics integration
│   ├── audio/                 # Audio playback via rodio
│   └── common/                # Shared types and utilities
└── examples/                  # Working demonstrations
    └── hello_world.rs        # Full-featured physics platformer demo
```

### Key Commands
```bash
# Run all tests
cargo test --workspace

# Run specific crate tests
cargo test -p engine_core
cargo test -p renderer
cargo test -p ecs

# Run demo
cargo run --example hello_world

# Check for TODOs (should be 0)
grep -r "TODO:" crates/ --include="*.rs" | wc -l

# Run with output
cargo test -- --nocapture
```

### Test Status Summary
```
Total: 356 tests
├─ Passed: 338 (100% of executable)
├─ Ignored: 18 (5% - require GPU/window)
└─ Failed: 0 (0%)

Success Rate: 100%
Quality: Excellent ✅
```

For more game programming patterns, refer to the [Game Programming Patterns](https://gameprogrammingpatterns.com/) book.

## Example Docstrings

### Crate-Level Documentation
```rust
//! Core functionality for the insiculous_2d game engine.
//! 
//! This crate provides the game loop, timing, and world state management.
```

### Struct Documentation
```rust
/// The main game loop
pub struct GameLoop {
    config: GameLoopConfig,
    timer: Timer,
    running: bool,
}
```

### Method Documentation
```rust
/// Create a new game loop with the given configuration
pub fn new(config: GameLoopConfig) -> Self {
    Self {
        config,
        timer: Timer::new(),
        running: false,
    }
}
```

### Error Type Documentation
```rust
/// Errors that can occur in the engine core
#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error("Failed to initialize engine: {0}")]
    InitializationError(String),

    #[error("Game loop error: {0}")]
    GameLoopError(String),
}
```

## Prompt Recipes

### Adding a New System

```
I want to add a new particle system to the engine. It should:
1. Be a component that can be attached to entities
2. Support configurable emission rate, particle lifetime, and velocity
3. Integrate with the renderer to draw particles efficiently
4. Follow the existing patterns in the codebase

Please help me design and implement this system.
```

### Refactoring Existing Code

```
I need to refactor the input system to support input mapping configurations. Users should be able to:
1. Define custom key/button mappings for actions
2. Save and load these configurations
3. Query input by action name rather than specific keys/buttons

Please help me refactor the input_handler.rs file while maintaining backward compatibility.
```

### Generating Documentation

```
Please generate comprehensive documentation for the ECS world implementation. Include:
1. Overview of the world's responsibility
2. Explanation of how entities, components, and systems interact
3. Code examples showing common usage patterns
4. Performance considerations and best practices
```

### Writing Tests

```
I need to write tests for the renderer module. Please help me create:
1. Unit tests for individual functions
2. Integration tests for the rendering pipeline
3. Mock objects for testing without a GPU
4. Test cases that cover error handling and edge cases

The tests should follow the existing testing patterns in the codebase.
```

### Implementing a New Feature

```
I want to implement a scene graph system that:
1. Allows entities to have parent-child relationships
2. Propagates transformations down the hierarchy
3. Supports efficient querying of entities by their position in the graph
4. Integrates with the existing ECS architecture

Please help me design and implement this feature.
```
