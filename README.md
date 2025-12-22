# Insiculous 2D

## Project Vision
Insiculous 2D is a lightweight, modular game engine designed for creating 2D games with Rust. It aims to provide a simple yet powerful API that allows developers to focus on game logic rather than boilerplate code. The engine prioritizes performance, cross-platform compatibility, and a clean, intuitive interface.

## Architecture Overview
The engine is built with a modular architecture, consisting of several core components:

```
insiculous_2d/
├── crates/
│   ├── engine_core/    # Core functionality and game loop
│   ├── renderer/       # WGPU-based rendering system
│   ├── ecs/            # Entity Component System
│   └── input/          # Input handling abstraction
└── examples/           # Example projects
```

Data flow:
1. Input events are captured by the input system
2. The Application manages one or more Scenes
3. Each Scene encapsulates its own ECS World and Scene Graph
4. Game state is updated in the Scene's ECS World
5. Rendering is handled by the WGPU renderer
6. The engine core coordinates all systems

## Quick Start

### Prerequisites
- Rust (latest stable version)
- Cargo
- Git

### Installation
```bash
# Clone the repository
git clone https://github.com/yourusername/insiculous_2d.git
cd insiculous_2d

# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup update stable

# Run the hello world example
cargo run --example hello_world
```

## Contributing
We welcome contributions to Insiculous 2D! Here are some guidelines to follow:

### Coding Standards
- **Single Responsibility Principle (SRP)**: Each module, class, and function should have a single responsibility.
- **Don't Repeat Yourself (DRY)**: Avoid code duplication by abstracting common functionality.
- **Descriptive Names**: Use clear, descriptive names for variables, functions, and modules.
- **Documentation**: Document public APIs with comments that explain purpose and usage.

### Commit Message Style
Follow the conventional commits specification:
```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

Types: feat, fix, docs, style, refactor, test, chore
Example: `feat(renderer): add sprite batching system`

### Pull Request Checklist
- [ ] Code follows the project's coding standards
- [ ] Tests have been added or updated for changes
- [ ] Documentation has been updated
- [ ] Commit messages follow the convention
- [ ] Changes have been tested on multiple platforms (if applicable)

## Development Progress

### ✅ Phase 1: Stabilization - COMPLETED (100%)
The foundation is now solid with comprehensive error handling, memory safety, and proper lifecycle management.

**Completed Steps:**
- ✅ **Step 1: Memory Safety & Lifetime Issues** - Fixed critical safety issues, removed unsafe lifetime requirements
- ✅ **Step 2: Input System Integration** - Implemented event queuing, input mapping, and thread-safe input handling  
- ✅ **Step 3: Core System Initialization** - Added lifecycle management, entity generation tracking, and panic recovery

**Key Achievements:**
- **66 Tests Passing**: 100% test success rate across all core systems
- **Memory Safety**: Eliminated race conditions and undefined behavior
- **Thread Safety**: All core systems support concurrent access safely
- **Error Recovery**: Engine can gracefully handle and recover from errors
- **Production Ready**: Comprehensive test coverage and proper error handling

### 🚧 Phase 2: Core Features - IN PROGRESS
Building essential game engine functionality for 2D game development.

**Current Step:**
- 🔄 **Step 4: Sprite Rendering System** - Implementing sprite batching, texture loading, and rendering optimizations

**Upcoming Steps:**
- [ ] Step 5: ECS Optimization - Archetype-based component storage and performance improvements
- [ ] Step 6: Resource Management - Asset loading, caching, and management systems
- [ ] Step 7: 2D Physics Integration - Physics engine integration for realistic gameplay

### 📋 Phase 3: Usability - PLANNED
Making the engine productive for game developers with tools and frameworks.

**Planned Steps:**
- [ ] Step 8: Scene Graph System - Hierarchical scene management and spatial queries
- [ ] Step 9: Audio System - 2D audio playback and spatial audio support
- [ ] Step 10: UI Framework - Immediate mode UI system for game interfaces

### 🚀 Phase 4: Polish - PLANNED  
Advanced features and optimization for production-ready games.

**Planned Steps:**
- [ ] Step 11: Advanced Rendering - 2D lighting, post-processing, and visual effects
- [ ] Step 12: Editor Tools - Visual editor for level design and game development
- [ ] Step 13: Platform Support - Mobile platforms, web export, and console support

## Roadmap
Here are our detailed technical milestones for upcoming development:

- [x] Input mapping system for configurable controls ✅ COMPLETED
- [x] Event queue system for frame-synchronized input ✅ COMPLETED  
- [x] Thread-safe input handling with concurrent access ✅ COMPLETED
- [x] Entity generation tracking for memory safety ✅ COMPLETED
- [x] System lifecycle management with panic recovery ✅ COMPLETED
- [x] Scene lifecycle with proper state management ✅ COMPLETED
- [ ] Sprite batching for improved rendering performance
- [ ] Texture loading and management system
- [ ] Dynamic buffer management for rendering
- [ ] Camera matrix calculations and transformations
- [ ] Physics system integration (rapier2d or similar)
- [ ] Audio system implementation with spatial audio
- [ ] Asset management pipeline with hot reloading
- [ ] Scene graph system with hierarchical transforms
- [ ] UI framework for game interfaces
- [ ] Particle system for visual effects
- [ ] Serialization/deserialization for game state
- [ ] Editor tools for level design and debugging
