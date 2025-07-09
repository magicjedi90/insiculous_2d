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

## Roadmap
Here are our planned milestones for upcoming development:

- [ ] Input mapping system for configurable controls
- [ ] Sprite batching for improved rendering performance
- [ ] Physics system integration
- [ ] Audio system implementation
- [ ] Asset management pipeline
- [ ] Scene management system
- [ ] UI framework
- [ ] Particle system
- [ ] Serialization/deserialization for game state
- [ ] Editor tools for level design
