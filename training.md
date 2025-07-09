# Insiculous 2D - AI Pair Programming Guide

## Context
Insiculous 2D is a lightweight, modular game engine designed for creating 2D games with Rust. It aims to provide a simple yet powerful API that allows developers to focus on game logic rather than boilerplate code. The engine prioritizes performance, cross-platform compatibility, and a clean, intuitive interface. The architecture follows a component-based design with clear separation of concerns between systems.

## Directory Map
- **crates/engine_core/** - Core functionality including game loop, timing, and scene state management
  - `game_loop.rs` - Main game loop implementation with configurable FPS and timestep
  - `timing.rs` - Utilities for time measurement and frame timing
  - `scene.rs` - Game scene state management
  - `application.rs` - Launcher and dependency coordinator

- **crates/renderer/** - WGPU-based rendering system
  - `renderer.rs` - Core rendering functionality using WGPU
  - `window.rs` - Window creation and management using winit
  - `error.rs` - Error handling specific to rendering

- **crates/ecs/** - Entity Component System for game object management
  - `component.rs` - Component trait and storage implementation
  - `entity.rs` - Entity management and identification
  - `system.rs` - System trait for game logic processing
  - `world.rs` - ECS world that ties entities, components, and systems together

- **crates/input/** - Input handling abstraction
  - `input_handler.rs` - Unified input handling facade
  - `keyboard.rs` - Keyboard state tracking
  - `mouse.rs` - Mouse state and event handling
  - `gamepad.rs` - Gamepad detection and state management

- **examples/** - Example projects demonstrating engine usage
  - `hello_world.rs` - Minimal example showing basic engine setup

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

### Component Pattern
Used in the ECS system to compose game objects from reusable components. See `crates/ecs/component.rs` for implementation.

### Update Method Pattern
Used in the game loop to update game state at regular intervals. See `crates/engine_core/game_loop.rs`.

### Service Locator Pattern
Used to provide access to engine subsystems like rendering and input. See how the `Renderer` provides access to the `Device` and `Queue`.

### Facade Pattern
Used in `InputHandler` to provide a unified interface to different input subsystems (keyboard, mouse, gamepad).

### Observer Pattern
Implemented through event systems for handling input and game events.

### Scene Encapsulation Pattern
Used to encapsulate an ECS World within each Scene, providing isolation between game scenes. This pattern enables:
- Isolated hot-reload of individual scenes
- Easy save-game implementation through scene serialization
- Multi-scene streaming similar to Bevy sub-worlds and Godot packed scenes

The architecture follows "Application → Scene(s) { World + Scene Graph }" where each Scene contains its own World and Scene Graph.

### Resource Acquisition Is Initialization (RAII)
Used throughout the codebase for resource management, particularly in the renderer.

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
