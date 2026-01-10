//! Hello World - Demonstrates the simplified Game API with Physics and Scene Graph
//!
//! This example shows how easy it is to create a game with the Insiculous 2D engine.
//! All the window, event loop, and rendering boilerplate is handled internally.
//!
//! Features demonstrated:
//! - Simple Game API (Game trait, GameConfig, run_game)
//! - RON scene file loading for entity/component definition
//! - ECS for entity/component management
//! - Asset Manager for loading/creating textures
//! - Input handling with keyboard
//! - 2D Physics with rapier2d integration
//! - **Scene Graph Hierarchy** - parent-child entity relationships with transform propagation
//!
//! Controls: WASD to move player, SPACE to jump, R to reset, ESC to exit
//!
//! Scene file: examples/assets/scenes/hello_world.scene.ron

use engine_core::prelude::*;
use ecs::hierarchy_system::TransformHierarchySystem;
use std::path::Path;

/// Our game state - simplified with BehaviorRunner handling player movement
struct HelloWorld {
    physics: Option<PhysicsSystem>,
    /// Behavior runner - processes all entity behaviors (player movement, AI, etc.)
    behaviors: BehaviorRunner,
    /// Scene instance with named entity lookups
    scene_instance: Option<SceneInstance>,
    /// Transform hierarchy system for parent-child relationships
    transform_hierarchy: TransformHierarchySystem,
}

impl HelloWorld {
    fn new() -> Self {
        Self {
            physics: None,
            behaviors: BehaviorRunner::new(),
            scene_instance: None,
            transform_hierarchy: TransformHierarchySystem::new(),
        }
    }

    fn reset_player(&mut self, ctx: &mut GameContext) {
        // Get player entity from scene instance
        let player = self.scene_instance.as_ref()
            .and_then(|scene| scene.get_entity("player"));

        if let Some(player) = player {
            // Reset player position
            if let Some(transform) = ctx.world.get_mut::<Transform2D>(player) {
                transform.position = Vec2::new(-200.0, 100.0);
            }
            // Reset velocity
            if let Some(body) = ctx.world.get_mut::<RigidBody>(player) {
                body.velocity = Vec2::ZERO;
            }
            // Update physics world
            if let Some(physics) = &mut self.physics {
                physics.physics_world_mut().set_body_transform(player, Vec2::new(-200.0, 100.0), 0.0);
                physics.physics_world_mut().set_body_velocity(player, Vec2::ZERO, 0.0);
            }
        }
    }
}

impl Game for HelloWorld {
    /// Called once at startup - load scene from file
    fn init(&mut self, ctx: &mut GameContext) {
        // Set asset base path to examples directory
        ctx.assets.set_base_path("examples");

        // Try to load the scene from RON file
        let scene_path = Path::new("examples/assets/scenes/hello_world.scene.ron");

        match SceneLoader::load_and_instantiate(scene_path, &mut ctx.world, ctx.assets) {
            Ok(instance) => {
                println!("Loaded scene '{}' with {} entities", instance.name, instance.entity_count);

                // Set named entities on behavior runner (for FollowEntity and other reference-based behaviors)
                self.behaviors.set_named_entities(instance.named_entities.clone());

                // Create physics system from scene settings
                let physics_config = if let Some(settings) = &instance.physics {
                    PhysicsConfig::new(Vec2::new(settings.gravity.0, settings.gravity.1))
                        .with_scale(settings.pixels_per_meter)
                } else {
                    PhysicsConfig::platformer()
                };

                self.physics = Some(PhysicsSystem::with_config(physics_config));
                self.scene_instance = Some(instance);
            }
            Err(e) => {
                println!("Failed to load scene: {}", e);
                println!("Creating entities programmatically as fallback...");

                // Fallback: create entities manually with Behavior component
                use ecs::behavior::Behavior;
                let player = ctx.world.create_entity();
                ctx.world.add_component(&player, Transform2D::new(Vec2::new(-200.0, 100.0))).ok();
                ctx.world.add_component(&player, Sprite::new(0).with_color(Vec4::new(0.2, 0.4, 1.0, 1.0))).ok();
                ctx.world.add_component(&player, RigidBody::player_platformer()).ok();
                ctx.world.add_component(&player, Collider::player_box()).ok();
                // Add behavior for player-controlled platformer movement
                ctx.world.add_component(&player, Behavior::PlayerPlatformer {
                    move_speed: 120.0,
                    jump_impulse: 420.0,
                    jump_cooldown: 0.3,
                    tag: "player".to_string(),
                }).ok();

                // Create ground
                let ground = ctx.world.create_entity();
                ctx.world.add_component(&ground,
                    Transform2D::new(Vec2::new(0.0, -250.0))
                        .with_scale(Vec2::new(10.0, 0.5))
                ).ok();
                ctx.world.add_component(&ground,
                    Sprite::new(0).with_color(Vec4::new(0.3, 0.3, 0.3, 1.0))
                ).ok();
                ctx.world.add_component(&ground, RigidBody::new_static()).ok();
                ctx.world.add_component(&ground, Collider::platform(800.0, 40.0)).ok();

                self.physics = Some(PhysicsSystem::with_config(PhysicsConfig::platformer()));
            }
        }

        // Initialize the physics system
        if let Some(physics) = &mut self.physics {
            use ecs::System;
            physics.initialize(&mut ctx.world).ok();
        }

        // Initialize the transform hierarchy system
        use ecs::System;
        self.transform_hierarchy.initialize(&mut ctx.world).ok();

        // Count entities with hierarchy relationships
        let root_count = ctx.world.get_root_entities().len();
        let total_count = ctx.world.entity_count();
        let child_count = total_count - root_count;

        println!("Game initialized with {} entities ({} root, {} children)",
                 total_count, root_count, child_count);
        println!("Controls: WASD to move, SPACE to jump, R to reset, ESC to exit");
        println!("Physics enabled - push the wood boxes around!");
        if child_count > 0 {
            println!("Scene Graph: {} child entities will follow their parents!", child_count);
        }
    }

    /// Called every frame - update game logic
    fn update(&mut self, ctx: &mut GameContext) {
        // Process all entity behaviors (player movement, AI, etc.)
        // This single call replaces 40+ lines of hardcoded movement logic!
        self.behaviors.update(
            &mut ctx.world,
            ctx.input,
            ctx.delta_time,
            self.physics.as_mut(),
        );

        // Reset player position (manual action, not a behavior)
        if ctx.input.is_key_pressed(KeyCode::KeyR) {
            self.reset_player(ctx);
        }

        // Step physics simulation
        if let Some(physics) = &mut self.physics {
            use ecs::System;
            physics.update(&mut ctx.world, ctx.delta_time);
        }

        // Update transform hierarchy - propagates transforms from parents to children
        // This must run after physics so child entities follow their parents
        {
            use ecs::System;
            self.transform_hierarchy.update(&mut ctx.world, ctx.delta_time);
        }
    }

    // render() uses the default implementation which extracts sprites from ECS
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create game configuration
    let game_config = GameConfig::new("Hello World - Insiculous 2D Physics Demo")
        .with_size(800, 600)
        .with_clear_color(0.1, 0.1, 0.15, 1.0);

    // Create and run the game
    let game = HelloWorld::new();
    run_game(game, game_config)
}
