//! Hello World - Demonstrates the simplified Game API with Asset Management
//!
//! This example shows how easy it is to create a game with the Insiculous 2D engine.
//! All the window, event loop, and rendering boilerplate is handled internally.
//!
//! Features demonstrated:
//! - Simple Game API (Game trait, GameConfig, run_game)
//! - ECS for entity/component management
//! - Asset Manager for loading/creating textures
//! - Input handling with keyboard
//!
//! Controls: WASD to move player, ESC to exit

use engine_core::prelude::*;

/// Our game state - just the data we need
struct HelloWorld {
    player: Option<EntityId>,
    target: Option<EntityId>,
    obstacles: Vec<EntityId>,
    wood_platform: Option<EntityId>,
    player_texture: Option<TextureHandle>,
    target_texture: Option<TextureHandle>,
    wood_texture: Option<TextureHandle>,
}

impl HelloWorld {
    fn new() -> Self {
        Self {
            player: None,
            target: None,
            obstacles: Vec::new(),
            wood_platform: None,
            player_texture: None,
            target_texture: None,
            wood_texture: None,
        }
    }
}

impl Game for HelloWorld {
    /// Called once at startup - create our entities and load assets
    fn init(&mut self, ctx: &mut GameContext) {
        // Set asset base path to examples directory (where wood_texture.png is)
        ctx.assets.set_base_path("examples");

        // Load wood texture from file
        match ctx.assets.load_texture("wood_texture.png") {
            Ok(handle) => {
                self.wood_texture = Some(handle);
                println!("Loaded wood_texture.png successfully!");
            }
            Err(e) => {
                println!("Failed to load wood_texture.png: {}", e);
            }
        }

        // Create textures using the asset manager
        // These are solid color textures for demonstration
        self.player_texture = ctx.assets
            .create_solid_color(32, 32, [50, 100, 255, 255])
            .ok();

        self.target_texture = ctx.assets
            .create_solid_color(32, 32, [255, 80, 80, 255])
            .ok();

        // Create a checkerboard texture for obstacles
        let obstacle_texture = ctx.assets
            .create_checkerboard(32, 32, [100, 100, 100, 255], [150, 150, 150, 255], 8)
            .ok();

        println!("Loaded {} textures", ctx.assets.texture_count());

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

        // Create wood platform entity at the bottom using the loaded texture
        if let Some(wood_tex) = self.wood_texture {
            let platform = ctx.world.create_entity();
            ctx.world.add_component(&platform, Transform2D::new(Vec2::new(0.0, -200.0))).ok();
            // Use the wood texture handle and scale it to be a platform
            ctx.world.add_component(&platform,
                Sprite::new(wood_tex.id)
                    .with_color(Vec4::ONE) // No tint, show original texture colors
                    .with_scale(Vec2::new(3.0, 0.5)) // Wide platform
            ).ok();
            self.wood_platform = Some(platform);
            println!("Created wood platform with texture handle {}", wood_tex.id);
        }

        // Create some obstacle entities using checkerboard texture
        for i in 0..3 {
            let obstacle = ctx.world.create_entity();
            let y = (i as f32 - 1.0) * 100.0;
            ctx.world.add_component(&obstacle, Transform2D::new(Vec2::new(0.0, y))).ok();

            // Use the obstacle texture handle if available, otherwise use white texture with color
            let sprite = if let Some(_tex) = obstacle_texture {
                Sprite::new(0).with_color(Vec4::new(0.5, 0.5, 0.5, 1.0))
            } else {
                Sprite::new(0).with_color(Vec4::new(0.5, 0.5, 0.5, 1.0))
            };
            ctx.world.add_component(&obstacle, sprite).ok();
            self.obstacles.push(obstacle);
        }

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
