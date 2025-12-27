//! Debug Sprite Demo - Comprehensive debugging for sprite visibility
//! 
//! This demo includes extensive debugging to identify why sprites aren't visible

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

/// Comprehensive debug demo for sprite visibility
struct DebugSpriteDemo {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    sprite_pipeline: Option<SpritePipeline>,
    input_handler: Arc<std::sync::Mutex<InputHandler>>,
    camera: Camera2D,
    sprites: Vec<Sprite>,
    frame_count: u32,
    debug_info: DebugInfo,
}

#[derive(Default)]
struct DebugInfo {
    window_visible: bool,
    surface_acquired: bool,
    sprites_created: bool,
    batches_created: bool,
    render_calls: u32,
    render_errors: u32,
    texture_issues: u32,
    batch_info: String,
}

impl DebugSpriteDemo {
    fn new() -> Self {
        // Create HIGHLY VISIBLE sprites with contrasting colors
        let mut sprites = Vec::new();
        
        // BRIGHT RED on DARK BLUE background - maximum contrast
        sprites.push(Sprite::new(TextureHandle { id: 0 })
            .with_position(Vec2::new(0.0, 0.0))
            .with_color(Vec4::new(1.0, 0.0, 0.0, 1.0)) // BRIGHT RED
            .with_scale(Vec2::new(100.0, 100.0))
            .with_depth(0.0));
            
        // BRIGHT GREEN - also high contrast
        sprites.push(Sprite::new(TextureHandle { id: 0 })
            .with_position(Vec2::new(-150.0, 0.0))
            .with_color(Vec4::new(0.0, 1.0, 0.0, 1.0)) // BRIGHT GREEN
            .with_scale(Vec2::new(80.0, 80.0))
            .with_depth(1.0));
            
        // BRIGHT YELLOW - maximum visibility
        sprites.push(Sprite::new(TextureHandle { id: 0 })
            .with_position(Vec2::new(150.0, 0.0))
            .with_color(Vec4::new(1.0, 1.0, 0.0, 1.0)) // BRIGHT YELLOW
            .with_scale(Vec2::new(60.0, 60.0))
            .with_depth(2.0));
        
        Self {
            window: None,
            renderer: None,
            sprite_pipeline: None,
            input_handler: Arc::new(std::sync::Mutex::new(InputHandler::new())),
            camera: Camera2D::default(),
            sprites,
            frame_count: 0,
            debug_info: DebugInfo::default(),
        }
    }
    
    fn log_debug_info(&self) {
        println!("=== DEBUG INFO ===");
        println!("Window visible: {}", self.debug_info.window_visible);
        println!("Surface acquired: {}", self.debug_info.surface_acquired);
        println!("Sprites created: {}", self.debug_info.sprites_created);
        println!("Batches created: {}", self.debug_info.batches_created);
        println!("Render calls: {}", self.debug_info.render_calls);
        println!("Render errors: {}", self.debug_info.render_errors);
        println!("Texture issues: {}", self.debug_info.texture_issues);
        println!("Batch info: {}", self.debug_info.batch_info);
        
        if !self.sprites.is_empty() {
            println!("Sprite count: {}", self.sprites.len());
            for (i, sprite) in self.sprites.iter().enumerate() {
                println!("  Sprite {}: pos=({:.1}, {:.1}), color=({:.1}, {:.1}, {:.1}, {:.1}), scale=({:.1}, {:.1}), depth={:.1}", 
                    i, sprite.position.x, sprite.position.y,
                    sprite.color.x, sprite.color.y, sprite.color.z, sprite.color.w,
                    sprite.scale.x, sprite.scale.y, sprite.depth);
            }
        }
        
        println!("Camera: pos=({:.1}, {:.1}), viewport=({:.1}, {:.1}), rotation={:.2}", 
            self.camera.position.x, self.camera.position.y,
            self.camera.viewport_size.x, self.camera.viewport_size.y,
            self.camera.rotation);
        println!("==================");
    }
}

impl ApplicationHandler<()> for DebugSpriteDemo {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        println!("DEBUG: Starting sprite visibility investigation...");
        self.log_debug_info();
        
        // Create window with maximum visibility settings
        let window_attributes = WindowAttributes::default()
            .with_title("DEBUG: Sprite Visibility Test")
            .with_inner_size(winit::dpi::LogicalSize::new(800, 600))
            .with_visible(true);
            
        let window = match event_loop.create_window(window_attributes) {
            Ok(window) => {
                println!("DEBUG: Window created successfully");
                self.debug_info.window_visible = window.is_visible().unwrap_or(false);
                println!("DEBUG: Window visible: {}", self.debug_info.window_visible);
                Arc::new(window)
            }
            Err(e) => {
                println!("ERROR: Failed to create window: {}", e);
                event_loop.exit();
                return;
            }
        };
        
        self.window = Some(window.clone());
        
        // Initialize renderer with detailed error checking
        match pollster::block_on(renderer::init(window)) {
            Ok(renderer) => {
                println!("DEBUG: Renderer initialized successfully");
                self.debug_info.surface_acquired = true;
                
                // Create sprite pipeline with detailed logging
                println!("DEBUG: Creating sprite pipeline...");
                let sprite_pipeline = SpritePipeline::new(renderer.device(), 100);
                println!("DEBUG: Sprite pipeline created successfully");
                
                // Configure camera for maximum visibility
                self.camera.viewport_size = Vec2::new(800.0, 600.0);
                self.camera.position = Vec2::new(0.0, 0.0); // Center of world
                self.camera.rotation = 0.0; // No rotation for simplicity
                
                // Create sprites
                self.debug_info.sprites_created = true;
                println!("DEBUG: Created {} sprites", self.sprites.len());
                
                self.renderer = Some(renderer);
                self.sprite_pipeline = Some(sprite_pipeline);
                
                println!("DEBUG: All systems initialized successfully");
                self.log_debug_info();
            }
            Err(e) => {
                println!("ERROR: Failed to initialize renderer: {}", e);
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
                println!("DEBUG: Window close requested");
                self.log_debug_info();
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                println!("DEBUG: Window resized to {}x{}", size.width, size.height);
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(size.width, size.height);
                }
                self.camera.viewport_size = Vec2::new(size.width as f32, size.height as f32);
                println!("DEBUG: Camera viewport updated to {:.1}x{:.1}", 
                    self.camera.viewport_size.x, self.camera.viewport_size.y);
            }
            WindowEvent::RedrawRequested => {
                println!("DEBUG: Redraw requested (frame {})", self.frame_count + 1);
                self.render_frame();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                use winit::event::DeviceId;
                self.input_handler.lock().unwrap().handle_window_event(&WindowEvent::KeyboardInput { 
                    device_id: DeviceId::dummy(), 
                    event: event.clone(), 
                    is_synthetic: false 
                });
                
                if let PhysicalKey::Code(KeyCode::Escape) = event.physical_key {
                    if event.state == ElementState::Pressed {
                        println!("DEBUG: ESC pressed - showing final debug info");
                        self.log_debug_info();
                        event_loop.exit();
                    }
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
        self.update_red_square();
        
        // Process input events
        {
            let mut input = self.input_handler.lock().unwrap();
            input.process_queued_events();
        }
        
        self.frame_count += 1;
        
        // Log detailed info every few frames
        if self.frame_count % 60 == 0 { // Every ~1 second at 60fps
            println!("DEBUG: Frame {} - Still rendering...", self.frame_count);
            if self.frame_count % 300 == 0 { // Every ~5 seconds
                self.log_debug_info();
            }
        }
    }
}

impl DebugSpriteDemo {
    fn update_red_square(&mut self) {
        let input = self.input_handler.lock().unwrap();
        let mut new_x = self.sprites[0].position.x;
        let mut new_y = self.sprites[0].position.y;
        let speed = 100.0;
        let delta_time = 0.016;
        
        let mut moved = false;
        
        if input.is_action_active(&GameAction::MoveLeft) {
            new_x -= speed * delta_time;
            moved = true;
            println!("DEBUG: Moving LEFT");
        }
        if input.is_action_active(&GameAction::MoveRight) {
            new_x += speed * delta_time;
            moved = true;
            println!("DEBUG: Moving RIGHT");
        }
        if input.is_action_active(&GameAction::MoveUp) {
            new_y += speed * delta_time;
            moved = true;
            println!("DEBUG: Moving UP");
        }
        if input.is_action_active(&GameAction::MoveDown) {
            new_y -= speed * delta_time;
            moved = true;
            println!("DEBUG: Moving DOWN");
        }
        
        if moved {
            println!("DEBUG: Red square moved to ({:.1}, {:.1})", new_x, new_y);
            self.sprites[0] = Sprite::new(TextureHandle { id: 0 })
                .with_position(Vec2::new(new_x, new_y))
                .with_color(self.sprites[0].color)
                .with_scale(self.sprites[0].scale)
                .with_depth(self.sprites[0].depth);
        }
    }
    
    fn render_frame(&mut self) {
        println!("DEBUG: Starting render_frame() call {}", self.debug_info.render_calls + 1);
        self.debug_info.render_calls += 1;
        
        // Convert sprites to batches with detailed logging
        let mut batcher = SpriteBatcher::new(100);
        for sprite in &self.sprites {
            println!("DEBUG: Adding sprite to batcher: pos=({:.1}, {:.1}), color=({:.1}, {:.1}, {:.1}, {:.1})", 
                sprite.position.x, sprite.position.y,
                sprite.color.x, sprite.color.y, sprite.color.z, sprite.color.w);
            batcher.add_sprite(sprite);
        }
        
        let batches: Vec<SpriteBatch> = batcher.batches().values().cloned().collect();
        self.debug_info.batches_created = !batches.is_empty();
        self.debug_info.batch_info = format!("{} batches with {} total sprites", batches.len(), self.sprites.len());
        
        println!("DEBUG: Created {} batches from {} sprites", batches.len(), self.sprites.len());
        
        let texture_resources = std::collections::HashMap::new(); // Empty for colored sprites
        let batch_refs: Vec<&SpriteBatch> = batches.iter().collect();
        
        println!("DEBUG: Starting actual rendering with {} batches", batch_refs.len());
        
        if let (Some(renderer), Some(sprite_pipeline)) = (&mut self.renderer, &self.sprite_pipeline) {
            match renderer.render_with_sprites(sprite_pipeline, &self.camera, &texture_resources, &batch_refs) {
                Ok(_) => {
                    if self.frame_count == 1 {
                        println!("DEBUG: FIRST FRAME RENDERED SUCCESSFULLY!");
                        println!("DEBUG: If you don't see sprites, check:");
                        println!("DEBUG: 1. Is the window actually visible on screen?");
                        println!("DEBUG: 2. Are the sprites being drawn behind something?");
                        println!("DEBUG: 3. Is the clear color obscuring the sprites?");
                        println!("DEBUG: 4. Are the sprites outside the viewport?");
                        println!("DEBUG: 5. Is there a graphics driver issue?");
                    }
                    println!("DEBUG: Frame {} rendered successfully", self.frame_count + 1);
                }
                Err(e) => {
                    self.debug_info.render_errors += 1;
                    println!("DEBUG: RENDER ERROR: {}", e);
                }
            }
        } else {
            println!("DEBUG: Cannot render - renderer or sprite_pipeline not available");
            if self.renderer.is_none() {
                println!("DEBUG: Renderer is None!");
            }
            if self.sprite_pipeline.is_none() {
                println!("DEBUG: SpritePipeline is None!");
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Detailed logging for debugging
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    println!("DEBUG SPRITE DEMO - VISIBILITY INVESTIGATION");
    println!("==============================================");
    println!("This demo will help identify why sprites aren't visible");
    println!("Look for colored squares: RED, GREEN, YELLOW");
    println!("Use WASD/Arrow keys to move the RED square");
    println!("Press ESC to exit and see final debug info");
    println!("==============================================");

    let event_loop = EventLoop::new()?;
    let mut app = DebugSpriteDemo::new();
    
    println!("DEBUG: Starting event loop...");
    
    event_loop.run_app(&mut app)?;
    
    println!("DEBUG: Demo completed!");
    Ok(())
}