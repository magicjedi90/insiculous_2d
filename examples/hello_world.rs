//! Hello World - Demonstrates the simplified Game API with Physics
//!
//! This example shows how easy it is to create a game with the Insiculous 2D engine.
//! All the window, event loop, and rendering boilerplate is handled internally.
//!
//! Features demonstrated:
//! - Simple Game API (Game trait, GameConfig, run_game)
//! - ECS for entity/component management
//! - Asset Manager for loading/creating textures
//! - Input handling with keyboard
//! - 2D Physics with rapier2d integration
//!
//! Controls: WASD to move player, SPACE to jump, R to reset, ESC to exit

use engine_core::prelude::*;

/// Our game state - just the data we need
struct HelloWorld {
    player: Option<EntityId>,
    ground: Option<EntityId>,
    obstacles: Vec<EntityId>,
    physics: Option<PhysicsSystem>,
    player_texture: Option<TextureHandle>,
    ground_texture: Option<TextureHandle>,
    jump_cooldown: f32,
}

impl HelloWorld {
    fn new() -> Self {
        Self {
            player: None,
            ground: None,
            obstacles: Vec::new(),
            // Use the platformer preset for standard 2D platformer physics
            physics: Some(PhysicsSystem::with_config(PhysicsConfig::platformer())),
            player_texture: None,
            ground_texture: None,
            jump_cooldown: 0.0,
        }
    }

    fn reset_player(&mut self, ctx: &mut GameContext) {
        if let Some(player) = self.player {
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
    /// Called once at startup - create our entities and load assets
    fn init(&mut self, ctx: &mut GameContext) {
        // Set asset base path to examples directory
        ctx.assets.set_base_path("examples");

        // Create textures for entities
        self.player_texture = ctx.assets
            .create_solid_color(32, 32, [50, 100, 255, 255])
            .ok();

        self.ground_texture = ctx.assets
            .create_solid_color(32, 32, [80, 80, 80, 255])
            .ok();

        let _obstacle_texture = ctx.assets
            .create_checkerboard(32, 32, [100, 50, 50, 255], [150, 75, 75, 255], 8)
            .ok();

        println!("Loaded {} textures", ctx.assets.texture_count());

        // Create player entity (blue, dynamic physics body)
        // Default sprite scale (1.0) renders at 80x80 pixels, so collider should match
        let player = ctx.world.create_entity();
        ctx.world.add_component(&player, Transform2D::new(Vec2::new(-200.0, 100.0))).ok();
        ctx.world.add_component(&player, Sprite::new(0).with_color(Vec4::new(0.2, 0.4, 1.0, 1.0))).ok();
        // Use preset rigid body and collider for platformer player
        ctx.world.add_component(&player, RigidBody::player_platformer()).ok();
        ctx.world.add_component(&player, Collider::player_box()).ok();
        self.player = Some(player);

        // Create ground entity (gray, static physics body)
        let ground = ctx.world.create_entity();
        ctx.world.add_component(&ground,
            Transform2D::new(Vec2::new(0.0, -250.0))
                .with_scale(Vec2::new(10.0, 0.5))
        ).ok();
        ctx.world.add_component(&ground,
            Sprite::new(0)
                .with_color(Vec4::new(0.3, 0.3, 0.3, 1.0))
            // Don't set sprite scale - transform.scale already handles sizing
        ).ok();
        ctx.world.add_component(&ground, RigidBody::new_static()).ok();
        ctx.world.add_component(&ground, Collider::platform(800.0, 40.0)).ok();
        self.ground = Some(ground);

        // Create floating platform (thicker for better collision)
        let platform = ctx.world.create_entity();
        ctx.world.add_component(&platform,
            Transform2D::new(Vec2::new(100.0, -50.0))
                .with_scale(Vec2::new(3.0, 0.5))
        ).ok();
        ctx.world.add_component(&platform,
            Sprite::new(0)
                .with_color(Vec4::new(0.4, 0.4, 0.4, 1.0))
            // Don't set sprite scale - transform.scale already handles sizing
        ).ok();
        ctx.world.add_component(&platform, RigidBody::new_static()).ok();
        ctx.world.add_component(&platform, Collider::platform(240.0, 40.0)).ok();

        // Create some obstacle boxes that can be pushed around
        // Space them out horizontally so they don't stack on spawn
        for i in 0..3 {
            let obstacle = ctx.world.create_entity();
            let x = -100.0 + (i as f32) * 120.0;  // More spacing between boxes
            let y = 0.0;  // All on the ground level
            ctx.world.add_component(&obstacle, Transform2D::new(Vec2::new(x, y))).ok();
            ctx.world.add_component(&obstacle,
                Sprite::new(0)
                    .with_color(Vec4::new(0.6, 0.3, 0.3, 1.0))
            ).ok();
            // Use preset for pushable objects
            ctx.world.add_component(&obstacle, RigidBody::pushable()).ok();
            ctx.world.add_component(&obstacle, Collider::pushable_box(80.0, 80.0)).ok();
            self.obstacles.push(obstacle);
        }

        // Initialize the physics system
        if let Some(physics) = &mut self.physics {
            use ecs::System;
            physics.initialize(&mut ctx.world).ok();
        }

        println!("Game initialized with {} entities", ctx.world.entity_count());
        println!("Controls: WASD to move, SPACE to jump, R to reset, ESC to exit");
        println!("Physics enabled - push the red boxes around!");
    }

    /// Called every frame - update game logic
    fn update(&mut self, ctx: &mut GameContext) {
        // Update jump cooldown
        if self.jump_cooldown > 0.0 {
            self.jump_cooldown -= ctx.delta_time;
        }

        // Move player based on input (velocity-based for precise control)
        if let Some(player) = self.player {
            // Use the platformer movement preset for tested, good-feeling values
            let movement = MovementConfig::platformer();
            let move_speed = movement.move_speed;
            let jump_impulse = movement.jump_impulse;

            // Get current velocity
            let current_vel = if let Some(physics) = &self.physics {
                physics.physics_world().get_body_velocity(player)
                    .map(|(v, _)| v)
                    .unwrap_or(Vec2::ZERO)
            } else {
                Vec2::ZERO
            };

            // Horizontal movement - set target velocity directly
            let mut target_vel_x = 0.0;
            if ctx.input.is_key_pressed(KeyCode::KeyA) {
                target_vel_x = -move_speed;
            }
            if ctx.input.is_key_pressed(KeyCode::KeyD) {
                target_vel_x = move_speed;
            }

            // Apply velocity change (preserve vertical velocity from gravity/jumping)
            if let Some(physics) = &mut self.physics {
                let new_vel = Vec2::new(target_vel_x, current_vel.y);
                physics.physics_world_mut().set_body_velocity(player, new_vel, 0.0);
            }

            // Jump (apply impulse when space is pressed)
            if ctx.input.is_key_pressed(KeyCode::Space) && self.jump_cooldown <= 0.0 {
                if let Some(physics) = &mut self.physics {
                    physics.apply_impulse(player, Vec2::new(0.0, jump_impulse));
                    self.jump_cooldown = 0.3; // 300ms cooldown
                }
            }

            // Reset player position
            if ctx.input.is_key_pressed(KeyCode::KeyR) {
                self.reset_player(ctx);
            }
        }

        // Step physics simulation
        if let Some(physics) = &mut self.physics {
            use ecs::System;
            physics.update(&mut ctx.world, ctx.delta_time);
        }
    }

    // render() uses the default implementation which extracts sprites from ECS
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure the game
    let config = GameConfig::new("Hello World - Insiculous 2D Physics Demo")
        .with_size(800, 600)
        .with_clear_color(0.1, 0.1, 0.15, 1.0);

    // Run the game - that's it!
    run_game(HelloWorld::new(), config)
}
