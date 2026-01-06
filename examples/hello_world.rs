//! Hello World - Demonstrates the simplified Game API
//!
//! This example shows how easy it is to create a game with the Insiculous 2D engine.
//! All the window, event loop, and rendering boilerplate is handled internally.
//!
//! Controls: WASD to move player, ESC to exit

use engine_core::prelude::*;

/// Our game state - just the data we need
struct HelloWorld {
    player: Option<EntityId>,
    target: Option<EntityId>,
}

impl HelloWorld {
    fn new() -> Self {
        Self {
            player: None,
            target: None,
        }
    }
}

impl Game for HelloWorld {
    /// Called once at startup - create our entities
    fn init(&mut self, ctx: &mut GameContext) {
        // Create player entity (blue, movable)
        let player = ctx.world.create_entity();
        ctx.world.add_component(&player, Transform2D::new(Vec2::new(-100.0, 0.0))).ok();
        ctx.world.add_component(&player, Sprite::new(0).with_color(Vec4::new(0.2, 0.4, 1.0, 1.0))).ok();
        self.player = Some(player);

        // Create target entity (red, stationary)
        let target = ctx.world.create_entity();
        ctx.world.add_component(&target, Transform2D::new(Vec2::new(100.0, 0.0))).ok();
        ctx.world.add_component(&target, Sprite::new(0).with_color(Vec4::new(1.0, 0.2, 0.2, 1.0))).ok();
        self.target = Some(target);

        println!("Game initialized with {} entities", ctx.world.entity_count());
        println!("Controls: WASD to move blue sprite, ESC to exit");
    }

    /// Called every frame - update game logic
    fn update(&mut self, ctx: &mut GameContext) {
        let speed = 200.0 * ctx.delta_time; // Pixels per second

        // Move player based on input
        if let Some(player) = self.player {
            if let Some(transform) = ctx.world.get_mut::<Transform2D>(player) {
                if ctx.input.is_key_pressed(KeyCode::KeyA) {
                    transform.position.x -= speed;
                }
                if ctx.input.is_key_pressed(KeyCode::KeyD) {
                    transform.position.x += speed;
                }
                if ctx.input.is_key_pressed(KeyCode::KeyW) {
                    transform.position.y += speed;
                }
                if ctx.input.is_key_pressed(KeyCode::KeyS) {
                    transform.position.y -= speed;
                }
            }
        }
    }

    // render() uses the default implementation which extracts sprites from ECS
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure the game
    let config = GameConfig::new("Hello World - Insiculous 2D")
        .with_size(800, 600)
        .with_clear_color(0.1, 0.1, 0.15, 1.0);

    // Run the game - that's it!
    run_game(HelloWorld::new(), config)
}
