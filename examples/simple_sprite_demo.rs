//! Simple Sprite Demo - Minimal logging, maximum visibility
//! 
//! This is a stripped-down version to test if we can actually see the sprite

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
use glam::{Vec2, Vec4};

/// Simple demo app with minimal logging
struct SimpleSpriteDemo {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    sprite_pipeline: Option<SpritePipeline>,
    input_handler: Arc<Mutex<InputHandler>>,
    camera: Camera2D,
    sprite: Sprite,
    frame_count: u32,
}

impl SimpleSpriteDemo {
    fn new() -> Self {
        Self {
            window: None,
            renderer: None,
            sprite_pipeline: None,
            input_handler: Arc::new(Mutex::new(InputHandler::new())),
            camera: Camera2D::default(),
            sprite: Sprite::new(TextureHandle { id: 0 })
                .with_position(Vec2::new(0.0, 0.0))
                .with_color(Vec4::new(1.0, 0.0, 0.0, 1.0)) // RED for visibility
                .with_scale(Vec2::new(100.0, 100.0)), // BIGGER square
            frame_count: 0,
        }
    }
}

impl ApplicationHandler<()> for SimpleSpriteDemo {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        println!("🎮 Simple Sprite Demo - Starting...");
        
        // Create window
        let window_attributes = WindowAttributes::default()
            .with_title("Simple Sprite Demo - RED SQUARE")
            .with_inner_size(winit::dpi::LogicalSize::new(800, 600));
            
        let window = match event_loop.create_window(window_attributes) {
            Ok(window) => {
                println!("✓ Window created");
                Arc::new(window)
            }
            Err(e) => {
                println!("❌ Failed to create window: {}", e);
                event_loop.exit();
                return;
            }
        };
        
        self.window = Some(window.clone());
        
        // Initialize renderer
        match pollster::block_on(renderer::init(window)) {
            Ok(renderer) => {
                println!("✓ Renderer initialized");
                
                // Create sprite pipeline
                let sprite_pipeline = SpritePipeline::new(renderer.device(), 100);
                
                // Update camera viewport
                self.camera.viewport_size = Vec2::new(800.0, 600.0);
                // Set camera to center view on origin
                self.camera.position = Vec2::new(0.0, 0.0);
                
                self.renderer = Some(renderer);
                self.sprite_pipeline = Some(sprite_pipeline);
                
                println!("✓ Sprite pipeline created");
                println!("🎯 Looking for RED square at position (0, 0)");
                println!("🕹️  Use WASD or Arrow Keys to move it around!");
            }
            Err(e) => {
                println!("❌ Failed to initialize renderer: {}", e);
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
        // Forward events to input handler
        self.input_handler.lock().unwrap().handle_window_event(&event);
        
        match event {
            WindowEvent::CloseRequested => {
                println!("👋 Closing demo...");
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(size.width, size.height);
                }
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
                
                use winit::event::DeviceId;
                self.input_handler.lock().unwrap().handle_window_event(&WindowEvent::KeyboardInput { 
                    device_id: DeviceId::dummy(), 
                    event, 
                    is_synthetic: false 
                });
                
                // Handle escape key exit after forwarding the event
                if should_exit {
                    println!("👋 ESC pressed - exiting...");
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
        
        // Update sprite position based on input
        self.update_sprite_position();
        
        // Process input events
        {
            let mut input = self.input_handler.lock().unwrap();
            input.process_queued_events();
        }
        
        self.frame_count += 1;
        
        // Log position occasionally (every ~3 seconds)
        if self.frame_count % 180 == 0 {
            println!("📍 Sprite position: ({:.1}, {:.1})", self.sprite.position.x, self.sprite.position.y);
        }
    }
}

impl SimpleSpriteDemo {
    fn update_sprite_position(&mut self) {
        let input = self.input_handler.lock().unwrap();
        let mut new_x = self.sprite.position.x;
        let mut new_y = self.sprite.position.y;
        let speed = 200.0;
        let delta_time = 0.016; // ~60 FPS
        
        let moved = if input.is_action_active(&GameAction::MoveLeft) {
            new_x -= speed * delta_time;
            true
        } else if input.is_action_active(&GameAction::MoveRight) {
            new_x += speed * delta_time;
            true
        } else if input.is_action_active(&GameAction::MoveUp) {
            new_y += speed * delta_time;
            true
        } else if input.is_action_active(&GameAction::MoveDown) {
            new_y -= speed * delta_time;
            true
        } else {
            false
        };
        
        if moved {
            self.sprite = Sprite::new(TextureHandle { id: 0 })
                .with_position(Vec2::new(new_x, new_y))
                .with_color(self.sprite.color)
                .with_scale(self.sprite.scale);
        }
    }
    
    fn render_frame(&mut self) {
        // Convert sprite to batch
        let mut batcher = SpriteBatcher::new(10);
        batcher.add_sprite(&self.sprite);
        
        let batches: Vec<SpriteBatch> = batcher.batches().values().cloned().collect();
        let texture_resources = std::collections::HashMap::new();
        let batch_refs: Vec<&SpriteBatch> = batches.iter().collect();
        
        // Render
        if let (Some(renderer), Some(sprite_pipeline)) = (&mut self.renderer, &self.sprite_pipeline) {
            match renderer.render_with_sprites(sprite_pipeline, &self.camera, &texture_resources, &batch_refs) {
                Ok(_) => {
                    if self.frame_count == 1 {
                        println!("✓ First frame rendered! Looking for RED square...");
                    }
                }
                Err(e) => {
                    if self.frame_count < 10 {
                        println!("❌ Render error: {}", e);
                    }
                }
            }
        }
    }
}

/// Main function
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up minimal logging
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Warn) // Only warnings and errors
        .init();

    println!("🎮 Simple Sprite Demo - RED SQUARE TEST");
    println!("========================================");
    println!("🎯 You should see a RED square in the window");
    println!("🕹️  Use WASD or Arrow Keys to move it");
    println!("🚪 Press ESC to exit");
    println!("========================================");

    let event_loop = EventLoop::new()?;
    let mut app = SimpleSpriteDemo::new();
    
    event_loop.run_app(&mut app)?;
    
    println!("✅ Demo completed!");
    Ok(())
}