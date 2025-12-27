//! Playable Demo - A simple game where you control a little black square
//! 
//! This example consolidates the best features from all our demos:
//! - Sprite rendering with a controllable black square
//! - Input handling with WASD/Arrow key movement
//! - ECS with optimized archetype storage
//! - Scene management with proper lifecycle
//! - Resource management foundation

use std::sync::{Arc, Mutex};
use winit::{
    application::ApplicationHandler,
    event::{WindowEvent, ElementState},
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowAttributes, WindowId},
    keyboard::{PhysicalKey, KeyCode},
};
use renderer::prelude::*;
use input::prelude::*;
use ecs::prelude::*;

use glam::{Vec2, Vec4};

/// Player component for our controllable square
#[derive(Debug, Clone)]
struct Player {
    speed: f32,
}

/// Position component for entities
#[derive(Debug, Clone)]
struct Position {
    x: f32,
    y: f32,
}

/// Velocity component for movement
#[derive(Debug, Clone)]
struct Velocity {
    dx: f32,
    dy: f32,
}

/// Sprite component linking to renderer
#[derive(Debug, Clone)]
struct SpriteComponent {
    size: Vec2,
    color: Vec4,
}

/// Movement system that updates positions based on velocity
struct MovementSystem;

impl System for MovementSystem {
    fn update(&mut self, _world: &mut World, _delta_time: f32) {
        // For now, we'll use a simplified approach
        // In a full implementation, we'd use the archetype query system
        // Removed verbose logging - only log if there are significant changes
    }

    fn name(&self) -> &str {
        "MovementSystem"
    }
}

/// Player control system that handles input
struct PlayerControlSystem {
    input_handler: Arc<Mutex<InputHandler>>,
}

impl PlayerControlSystem {
    fn new(input_handler: Arc<Mutex<InputHandler>>) -> Self {
        Self { input_handler }
    }
}

impl System for PlayerControlSystem {
    fn update(&mut self, _world: &mut World, _delta_time: f32) {
        let input = self.input_handler.lock().unwrap();
        
        // Handle player movement - removed verbose logging
        // Only log when movement actually happens (will be handled in update_player)
    }

    fn name(&self) -> &str {
        "PlayerControlSystem"
    }
}

/// Main application state
struct PlayableDemoApp {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    sprite_pipeline: Option<SpritePipeline>,
    input_handler: Arc<Mutex<InputHandler>>,
    world: Option<World>,
    camera: Camera2D,
    player_entity: Option<EntityId>,
    sprites: Vec<Sprite>,
    time: f32,
}

impl PlayableDemoApp {
    fn new() -> Self {
        // Create input handler (already has default bindings)
        let input_handler = InputHandler::new();
        
        Self {
            window: None,
            renderer: None,
            sprite_pipeline: None,
            input_handler: Arc::new(Mutex::new(input_handler)),
            world: None,
            camera: Camera2D::default(),
            player_entity: None,
            sprites: Vec::new(),
            time: 0.0,
        }
    }
    
    fn create_player_sprite(&self) -> Sprite {
        // Create a little WHITE square sprite (more visible than black)
        Sprite::new(TextureHandle { id: 0 }) // Use texture ID 0 for now
            .with_position(Vec2::new(0.0, 0.0)) // Start at center
            .with_color(Vec4::new(1.0, 1.0, 1.0, 1.0)) // WHITE color for visibility
            .with_scale(Vec2::new(50.0, 50.0)) // Little square
            .with_depth(1.0) // Ensure it's on top
    }
    
    fn initialize_ecs(&mut self) {
        println!("Initializing ECS with optimized archetype storage...");
        
        // Create optimized ECS world
        let mut world = World::new_optimized();
        world.initialize().expect("Failed to initialize world");
        world.start().expect("Failed to start world");
        
        // Create player entity
        let player_entity = world.create_entity();
        
        // Add player components
        world.add_component(&player_entity, Player { speed: 200.0 }).unwrap();
        world.add_component(&player_entity, Position { x: 0.0, y: 0.0 }).unwrap();
        world.add_component(&player_entity, Velocity { dx: 0.0, dy: 0.0 }).unwrap();
        world.add_component(&player_entity, SpriteComponent {
            size: Vec2::new(50.0, 50.0),
            color: Vec4::new(1.0, 1.0, 1.0, 1.0), // WHITE for visibility
        }).unwrap();
        
        // Add systems
        world.add_system(MovementSystem);
        world.add_system(PlayerControlSystem::new(self.input_handler.clone()));
        
        self.player_entity = Some(player_entity);
        self.world = Some(world);
        
        println!("✓ ECS initialized with {} entities", self.world.as_ref().unwrap().entity_count());
        println!("🎮 Player entity created with Position, Velocity, Player, and SpriteComponent");
    }
    
    fn update_player(&mut self, delta_time: f32) {
        // For now, we'll just simulate player movement
        // In a full implementation, we'd update the sprite based on ECS position
        if !self.sprites.is_empty() {
            let input = self.input_handler.lock().unwrap();
            let mut new_x = self.sprites[0].position.x;
            let mut new_y = self.sprites[0].position.y;
            
            if input.is_action_active(&GameAction::MoveLeft) {
                new_x -= 200.0 * delta_time; // speed * delta_time
            }
            if input.is_action_active(&GameAction::MoveRight) {
                new_x += 200.0 * delta_time;
            }
            if input.is_action_active(&GameAction::MoveUp) {
                new_y += 200.0 * delta_time;
            }
            if input.is_action_active(&GameAction::MoveDown) {
                new_y -= 200.0 * delta_time;
            }
            
            self.sprites[0] = Sprite::new(TextureHandle { id: 0 })
                .with_position(Vec2::new(new_x, new_y))
                .with_color(self.sprites[0].color)
                .with_scale(self.sprites[0].scale);
        }
        
        // Update ECS world
        if let Some(world) = &mut self.world {
            world.update(delta_time).expect("Failed to update world");
        }
        
        // Process input events
        {
            let mut input = self.input_handler.lock().unwrap();
            input.update();
        }
    }
}

impl ApplicationHandler<()> for PlayableDemoApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        println!("Playable Demo - Creating window and initializing systems...");
        
        // Create window
        let window_attributes = WindowAttributes::default()
            .with_title("Insiculous 2D - Playable Demo (Control the Black Square!)")
            .with_inner_size(winit::dpi::LogicalSize::new(800, 600));
            
        let window = match event_loop.create_window(window_attributes) {
            Ok(window) => {
                println!("✓ Window created successfully");
                Arc::new(window)
            }
            Err(e) => {
                println!("✗ Failed to create window: {}", e);
                event_loop.exit();
                return;
            }
        };
        
        self.window = Some(window.clone());
        
        // Initialize renderer
        match pollster::block_on(renderer::init(window)) {
            Ok(renderer) => {
                println!("✓ Renderer initialized successfully");
                
                // Create sprite pipeline
                let sprite_pipeline = SpritePipeline::new(renderer.device(), 1000);
                
                // Update camera viewport
                self.camera.viewport_size = Vec2::new(800.0, 600.0);
                
                // Create player sprite
                self.sprites.push(self.create_player_sprite());
                
                self.renderer = Some(renderer);
                self.sprite_pipeline = Some(sprite_pipeline);
                
                // Initialize ECS
                self.initialize_ecs();
                
                println!("✓ All systems initialized!");
                println!("🎮 Controls: Use WASD or Arrow Keys to move the black square!");
            }
            Err(e) => {
                println!("✗ Failed to initialize renderer: {}", e);
                event_loop.exit();
                return;
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                println!("Window close requested - shutting down gracefully");
                if let Some(world) = &mut self.world {
                    world.shutdown().expect("Failed to shutdown world");
                }
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                println!("Window resized to {}x{}", size.width, size.height);
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(size.width, size.height);
                }
                // Update camera viewport
                self.camera.viewport_size = Vec2::new(size.width as f32, size.height as f32);
            }
            WindowEvent::RedrawRequested => {
                self.render_frame();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                // Check for escape key first (before moving the event)
                let should_exit = if let PhysicalKey::Code(KeyCode::Escape) = event.physical_key {
                    event.state == ElementState::Pressed
                } else {
                    false
                };
                
                // Forward keyboard events to input handler - create a new event with proper device_id
                use winit::event::DeviceId;
                self.input_handler.lock().unwrap().handle_window_event(&WindowEvent::KeyboardInput { 
                    device_id: DeviceId::dummy(), // Use dummy device ID for now
                    event, 
                    is_synthetic: false 
                });
                
                // Handle escape key exit after forwarding the event
                if should_exit {
                    println!("Escape pressed - exiting...");
                    event_loop.exit();
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // Request redraw for continuous rendering
        if let Some(window) = &self.window {
            window.request_redraw();
        }
        
        // Update time for animation
        self.time += 0.016; // Assume 60 FPS
        
        // Update player movement (reduced frequency logging)
        self.update_player(0.016);
        
        // Process input events
        {
            let mut input = self.input_handler.lock().unwrap();
            input.process_queued_events();
        }
        
        // Log position occasionally (every ~2 seconds)
        if (self.time * 0.5) as i32 != ((self.time - 0.016) * 0.5) as i32 {
            if !self.sprites.is_empty() {
                let sprite = &self.sprites[0];
                println!("📍 Player position: ({:.1}, {:.1})", sprite.position.x, sprite.position.y);
            }
        }
    }
}

impl PlayableDemoApp {
    fn render_frame(&mut self) {
        // Debug: Log sprite info every few frames
        if self.time > 0.0 && (self.time * 5.0) as i32 != ((self.time - 0.016) * 5.0) as i32 {
            if !self.sprites.is_empty() {
                let sprite = &self.sprites[0];
                println!("🎮 Player sprite: pos=({:.1}, {:.1}), color=({:.1}, {:.1}, {:.1}, {:.1}), scale=({:.1}, {:.1})", 
                    sprite.position.x, sprite.position.y,
                    sprite.color.x, sprite.color.y, sprite.color.z, sprite.color.w,
                    sprite.scale.x, sprite.scale.y);
            }
        }
        
        // Convert sprites to batches
        let mut batcher = SpriteBatcher::new(1000);
        for sprite in &self.sprites {
            batcher.add_sprite(sprite);
        }
        
        // Create batches for rendering
        let mut batches: Vec<SpriteBatch> = batcher.batches().values().cloned().collect();
        let texture_resources = std::collections::HashMap::new(); // Empty for now
        
        // Sort batches by depth for proper alpha blending
        for batch in &mut batches {
            batch.sort_by_depth();
        }
        
        let batch_refs: Vec<&SpriteBatch> = batches.iter().collect();
        
        // Debug: Log batch info
        if !batch_refs.is_empty() {
            println!("🎨 Rendering {} batches with {} total sprites", batch_refs.len(), self.sprites.len());
        }
        
        // Render
        if let (Some(renderer), Some(sprite_pipeline)) = (&mut self.renderer, &self.sprite_pipeline) {
            // Update camera with slight animation
            self.camera.rotation = (self.time * 0.2).sin() * 0.05; // Gentle sway
            
            // Render with sprites
            match renderer.render_with_sprites(sprite_pipeline, &self.camera, &texture_resources, &batch_refs) {
                Ok(_) => {
                    // Frame rendered successfully
                    if self.time < 0.1 {
                        println!("✓ First frame rendered successfully!");
                    }
                }
                Err(e) => {
                    println!("❌ Failed to render frame: {}", e);
                }
            }
        } else {
            println!("❌ Renderer or sprite pipeline not available");
        }
    }
}

/// Main function
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    println!("🎮 Insiculous 2D - Playable Demo");
    println!("==================================");
    println!("🕹️  Controls:");
    println!("  - WASD or Arrow Keys: Move the black square");
    println!("  - ESC: Exit the game");
    println!("");
    println!("🎯 Objective: Move the little black square around!");
    println!("==================================");

    // Create event loop
    let event_loop = EventLoop::new()?;
    
    // Create application
    let mut app = PlayableDemoApp::new();
    
    println!("Starting playable demo...");
    
    // Run the event loop
    event_loop.run_app(&mut app)?;
    
    println!("Playable demo completed successfully!");
    
    Ok(())
}