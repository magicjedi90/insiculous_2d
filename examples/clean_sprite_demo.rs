//! Clean Sprite Demo - No emojis, just working sprites
//! 
//! This demonstrates visible colored rectangles on screen

use std::sync::Arc;
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

/// Demo that shows visible colored rectangles
struct CleanSpriteDemo {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    sprite_pipeline: Option<SpritePipeline>,
    input_handler: Arc<std::sync::Mutex<InputHandler>>,
    camera: Camera2D,
    sprites: Vec<Sprite>,
    frame_count: u32,
}

impl CleanSpriteDemo {
    fn new() -> Self {
        // Create multiple colored squares for better visibility
        let mut sprites = Vec::new();
        
        // Red square (center)
        sprites.push(Sprite::new(TextureHandle { id: 0 })
            .with_position(Vec2::new(0.0, 0.0))
            .with_color(Vec4::new(1.0, 0.0, 0.0, 1.0))
            .with_scale(Vec2::new(80.0, 80.0))
            .with_depth(0.0));
            
        // Blue square (left)
        sprites.push(Sprite::new(TextureHandle { id: 0 })
            .with_position(Vec2::new(-150.0, 0.0))
            .with_color(Vec4::new(0.0, 0.0, 1.0, 1.0))
            .with_scale(Vec2::new(60.0, 60.0))
            .with_depth(1.0));
            
        // Green square (right)
        sprites.push(Sprite::new(TextureHandle { id: 0 })
            .with_position(Vec2::new(150.0, 0.0))
            .with_color(Vec4::new(0.0, 1.0, 0.0, 1.0))
            .with_scale(Vec2::new(60.0, 60.0))
            .with_depth(2.0));
            
        // Yellow square (top)
        sprites.push(Sprite::new(TextureHandle { id: 0 })
            .with_position(Vec2::new(0.0, 100.0))
            .with_color(Vec4::new(1.0, 1.0, 0.0, 1.0))
            .with_scale(Vec2::new(50.0, 50.0))
            .with_depth(3.0));
            
        // Purple square (bottom)
        sprites.push(Sprite::new(TextureHandle { id: 0 })
            .with_position(Vec2::new(0.0, -100.0))
            .with_color(Vec4::new(1.0, 0.0, 1.0, 1.0))
            .with_scale(Vec2::new(50.0, 50.0))
            .with_depth(4.0));
        
        Self {
            window: None,
            renderer: None,
            sprite_pipeline: None,
            input_handler: Arc::new(std::sync::Mutex::new(InputHandler::new())),
            camera: Camera2D::default(),
            sprites,
            frame_count: 0,
        }
    }
}

impl ApplicationHandler<()> for CleanSpriteDemo {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        println!("Clean Sprite Demo - Multiple Colored Squares");
        println!("==============================================");
        println!("You should see RED, BLUE, GREEN, YELLOW, PURPLE squares");
        println!("Use WASD or Arrow Keys to move the RED square");
        println!("Press ESC to exit");
        println!("==============================================");
        
        let window_attributes = WindowAttributes::default()
            .with_title("Clean Sprite Demo - COLORED SQUARES")
            .with_inner_size(winit::dpi::LogicalSize::new(800, 600));
            
        let window = match event_loop.create_window(window_attributes) {
            Ok(window) => {
                println!("Window created");
                Arc::new(window)
            }
            Err(e) => {
                println!("Failed to create window: {}", e);
                event_loop.exit();
                return;
            }
        };
        
        self.window = Some(window.clone());
        
        match pollster::block_on(renderer::init(window)) {
            Ok(renderer) => {
                println!("Renderer initialized");
                
                let sprite_pipeline = SpritePipeline::new(renderer.device(), 100);
                self.camera.viewport_size = Vec2::new(800.0, 600.0);
                self.camera.position = Vec2::new(0.0, 0.0); // Center camera
                
                self.renderer = Some(renderer);
                self.sprite_pipeline = Some(sprite_pipeline);
                
                println!("Created {} colored squares", self.sprites.len());
                println!("Looking for colored squares...");
            }
            Err(e) => {
                println!("Failed to initialize renderer: {}", e);
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
        self.input_handler.lock().unwrap().handle_window_event(&event);
        
        match event {
            WindowEvent::CloseRequested => {
                println!("Closing demo...");
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
                
                if should_exit {
                    println!("ESC pressed - exiting...");
                    event_loop.exit();
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
        
        self.update_red_square();
        
        {
            let mut input = self.input_handler.lock().unwrap();
            input.process_queued_events();
        }
        
        self.frame_count += 1;
        
        // Log position every ~3 seconds
        if self.frame_count % 180 == 0 {
            println!("RED square position: ({:.1}, {:.1})", 
                self.sprites[0].position.x, self.sprites[0].position.y);
        }
    }
}

impl CleanSpriteDemo {
    fn update_red_square(&mut self) {
        let input = self.input_handler.lock().unwrap();
        let mut new_x = self.sprites[0].position.x;
        let mut new_y = self.sprites[0].position.y;
        let speed = 150.0;
        let delta_time = 0.016;
        
        let mut moved = false;
        
        if input.is_action_active(&GameAction::MoveLeft) {
            new_x -= speed * delta_time;
            moved = true;
        }
        if input.is_action_active(&GameAction::MoveRight) {
            new_x += speed * delta_time;
            moved = true;
        }
        if input.is_action_active(&GameAction::MoveUp) {
            new_y += speed * delta_time;
            moved = true;
        }
        if input.is_action_active(&GameAction::MoveDown) {
            new_y -= speed * delta_time;
            moved = true;
        }
        
        if moved {
            self.sprites[0] = Sprite::new(TextureHandle { id: 0 })
                .with_position(Vec2::new(new_x, new_y))
                .with_color(self.sprites[0].color)
                .with_scale(self.sprites[0].scale)
                .with_depth(self.sprites[0].depth);
        }
    }
    
    fn render_frame(&mut self) {
        let mut batcher = SpriteBatcher::new(100);
        for sprite in &self.sprites {
            batcher.add_sprite(sprite);
        }
        
        let batches: Vec<SpriteBatch> = batcher.batches().values().cloned().collect();
        let texture_resources = std::collections::HashMap::new();
        let batch_refs: Vec<&SpriteBatch> = batches.iter().collect();
        
        if let (Some(renderer), Some(sprite_pipeline)) = (&mut self.renderer, &self.sprite_pipeline) {
            match renderer.render_with_sprites(sprite_pipeline, &self.camera, &texture_resources, &batch_refs) {
                Ok(_) => {
                    if self.frame_count == 1 {
                        println!("First frame rendered! Look for colored squares...");
                        println!("RED square is controllable with WASD/Arrow keys");
                    }
                }
                Err(e) => {
                    if self.frame_count < 5 {
                        println!("Render error: {}", e);
                    }
                }
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Minimal logging - only warnings and errors
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Warn)
        .init();

    println!("Clean Sprite Demo - COLORED SQUARES");
    println!("=========================================");
    println!("Look for: RED (movable), BLUE, GREEN, YELLOW, PURPLE squares");
    println!("Controls: WASD/Arrows move RED square");
    println!("Exit: ESC key");
    println!("=========================================");

    let event_loop = EventLoop::new()?;
    let mut app = CleanSpriteDemo::new();
    
    event_loop.run_app(&mut app)?;
    
    println!("Demo completed!");
    Ok(())
}