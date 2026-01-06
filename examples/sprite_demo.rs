//! Sprite Demo - Demonstrates the sprite rendering API
//!
//! This example shows 7 animated colored sprites with smooth motion.
//! Controls: WASD to move camera, ESC to exit

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

/// Sprite rendering demo (currently broken - sprites are invisible)
struct SpriteDemo {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    sprite_pipeline: Option<SpritePipeline>,
    input_handler: Arc<std::sync::Mutex<InputHandler>>,
    camera: Camera2D,
    time: f32,
    sprites: Vec<Sprite>,
    frame_count: u32,
    animation_speed: f32,
}

impl SpriteDemo {
    fn new() -> Self {
        Self {
            window: None,
            renderer: None,
            sprite_pipeline: None,
            input_handler: Arc::new(std::sync::Mutex::new(InputHandler::new())),
            camera: Camera2D::default(),
            time: 0.0,
            sprites: Vec::new(),
            frame_count: 0,
            animation_speed: 1.0,
        }
    }
    
    /// Create sprites for the demo
    fn create_sprites(&mut self) {
        self.sprites.clear();
        
        // Use white texture (handle 0) for colored sprites
        let white_texture = TextureHandle { id: 0 };
        
        // 🌟 **RADIANT CENTER STAR** - Pure white, rotating
        self.sprites.push(Sprite::new(white_texture)
            .with_position(Vec2::new(0.0, 0.0))
            .with_color(Vec4::new(1.0, 1.0, 1.0, 1.0))  // Pure white
            .with_scale(Vec2::new(120.0, 120.0))
            .with_depth(0.0));
        
        // 🔴 **VIBRANT RED ORB** - Top position, pulsing
        self.sprites.push(Sprite::new(white_texture)
            .with_position(Vec2::new(0.0, 200.0))
            .with_color(Vec4::new(1.0, 0.1, 0.1, 1.0))  // Bright red
            .with_scale(Vec2::new(80.0, 80.0))
            .with_depth(1.0));
        
        // 💙 **ELECTRIC BLUE GEM** - Bottom position, scaling
        self.sprites.push(Sprite::new(white_texture)
            .with_position(Vec2::new(0.0, -200.0))
            .with_color(Vec4::new(0.1, 0.3, 1.0, 1.0))  // Electric blue
            .with_scale(Vec2::new(70.0, 70.0))
            .with_depth(2.0));
        
        // 💚 **LIME GREEN CRYSTAL** - Left position, rotating
        self.sprites.push(Sprite::new(white_texture)
            .with_position(Vec2::new(-250.0, 0.0))
            .with_color(Vec4::new(0.3, 1.0, 0.1, 1.0))  // Lime green
            .with_scale(Vec2::new(60.0, 60.0))
            .with_depth(3.0));
        
        // 💜 **ROYAL PURPLE JEWEL** - Right position, orbiting
        self.sprites.push(Sprite::new(white_texture)
            .with_position(Vec2::new(250.0, 0.0))
            .with_color(Vec4::new(0.8, 0.1, 1.0, 1.0))  // Royal purple
            .with_scale(Vec2::new(65.0, 65.0))
            .with_depth(4.0));
        
        // 🧡 **SUNSET ORANGE** - Top-left, gentle float
        self.sprites.push(Sprite::new(white_texture)
            .with_position(Vec2::new(-180.0, 150.0))
            .with_color(Vec4::new(1.0, 0.5, 0.0, 1.0))  // Sunset orange
            .with_scale(Vec2::new(50.0, 50.0))
            .with_depth(5.0));
        
        // 💛 **GOLDEN YELLOW** - Top-right, sparkling
        self.sprites.push(Sprite::new(white_texture)
            .with_position(Vec2::new(180.0, 150.0))
            .with_color(Vec4::new(1.0, 0.9, 0.0, 1.0))  // Golden yellow
            .with_scale(Vec2::new(55.0, 55.0))
            .with_depth(6.0));
    }
    
    /// ✨ **BEAUTIFUL ANIMATION** - Smooth, elegant movement patterns
    fn update_animations(&mut self) {
        let time = self.time;
        let speed = self.animation_speed;
        
        // Center star - gentle rotation
        if let Some(sprite) = self.sprites.get_mut(0) {
            sprite.rotation = (time * 0.5 * speed).sin() * 0.2;
        }
        
        // Red orb - pulsing scale
        if let Some(sprite) = self.sprites.get_mut(1) {
            let pulse = (time * 2.0 * speed).sin() * 0.2 + 1.0;
            sprite.scale = Vec2::new(80.0 * pulse, 80.0 * pulse);
        }
        
        // Blue gem - vertical float
        if let Some(sprite) = self.sprites.get_mut(2) {
            let float = (time * 1.5 * speed).sin() * 30.0;
            sprite.position.y = -200.0 + float;
        }
        
        // Green crystal - rotation with orbit
        if let Some(sprite) = self.sprites.get_mut(3) {
            sprite.rotation = time * speed;
            let orbit = (time * 0.8 * speed).sin() * 20.0;
            sprite.position.x = -250.0 + orbit;
        }
        
        // Purple jewel - circular orbit
        if let Some(sprite) = self.sprites.get_mut(4) {
            let orbit_radius = 50.0;
            let orbit_speed = time * speed;
            sprite.position.x = 250.0 + orbit_radius * orbit_speed.cos();
            sprite.position.y = orbit_radius * orbit_speed.sin();
        }
        
        // Orange float - gentle drift
        if let Some(sprite) = self.sprites.get_mut(5) {
            let drift = (time * 0.6 * speed).sin() * 15.0;
            sprite.position.y = 150.0 + drift;
            sprite.position.x = -180.0 + drift * 0.5;
        }
        
        // Yellow sparkle - scaling shimmer
        if let Some(sprite) = self.sprites.get_mut(6) {
            let shimmer = (time * 3.0 * speed).sin() * 0.15 + 1.0;
            sprite.scale = Vec2::new(55.0 * shimmer, 55.0 * shimmer);
        }
    }
    
    /// Set up the background color
    fn setup_background(&mut self) {
        if let Some(renderer) = &mut self.renderer {
            // Deep space gradient: Dark blue to black
            renderer.set_clear_color(0.05, 0.08, 0.15, 1.0); // Deep space blue
        }
    }
}

impl ApplicationHandler<()> for SpriteDemo {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        println!("Sprite Demo");
        println!("===========");
        println!("7 animated colored sprites with smooth motion");
        println!("Controls: WASD to move camera, ESC to exit");
        println!("===========");

        let window_attributes = WindowAttributes::default()
            .with_title("Sprite Demo - Insiculous 2D")
            .with_inner_size(winit::dpi::LogicalSize::new(1024, 768));
            
        let window = match event_loop.create_window(window_attributes) {
            Ok(window) => {
                println!("✅ Window created successfully");
                Arc::new(window)
            }
            Err(e) => {
                println!("❌ Failed to create window: {}", e);
                event_loop.exit();
                return;
            }
        };
        
        self.window = Some(window.clone());
        
        match pollster::block_on(renderer::init(window)) {
            Ok(renderer) => {
                println!("Renderer: {}", renderer.adapter_info());

                let sprite_pipeline = SpritePipeline::new(renderer.device(), 1000);

                self.camera.viewport_size = Vec2::new(1024.0, 768.0);
                self.camera.position = Vec2::new(0.0, 0.0);
                self.camera.zoom = 1.0;

                self.renderer = Some(renderer);
                self.sprite_pipeline = Some(sprite_pipeline);

                self.create_sprites();
                self.setup_background();

                println!("Created {} animated sprites", self.sprites.len());
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
        self.input_handler.lock().unwrap().handle_window_event(&event);
        
        match event {
            WindowEvent::CloseRequested => {
                println!("🚪 Closing perfect demo...");
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
                // Check for escape key
                let should_exit = if let PhysicalKey::Code(KeyCode::Escape) = event.physical_key {
                    event.state == ElementState::Pressed
                } else {
                    false
                };
                
                // Handle input for camera movement
                use winit::event::DeviceId;
                self.input_handler.lock().unwrap().handle_window_event(&WindowEvent::KeyboardInput { 
                    device_id: DeviceId::dummy(), 
                    event, 
                    is_synthetic: false 
                });
                
                if should_exit {
                    println!("🚪 ESC pressed - exiting demo...");
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
        
        // Update time and animations
        self.time += 0.016; // 60 FPS
        self.update_animations();
        
        // Handle camera controls
        self.update_camera();
        
        // Handle animation speed controls
        self.update_animation_speed();
        
        {
            let mut input = self.input_handler.lock().unwrap();
            input.process_queued_events();
        }
        
        self.frame_count += 1;
        
        // Status update every 5 seconds
        if self.frame_count % 300 == 0 {
            println!("Frame {} - {} sprites rendering", self.frame_count / 60, self.sprites.len());
        }
    }
}

impl SpriteDemo {
    /// 📷 Update camera based on input
    fn update_camera(&mut self) {
        let input = self.input_handler.lock().unwrap();
        let speed = 200.0;
        let delta_time = 0.016;
        
        let mut moved = false;
        let mut new_pos = self.camera.position;
        
        if input.is_action_active(&GameAction::MoveLeft) {
            new_pos.x -= speed * delta_time;
            moved = true;
        }
        if input.is_action_active(&GameAction::MoveRight) {
            new_pos.x += speed * delta_time;
            moved = true;
        }
        if input.is_action_active(&GameAction::MoveUp) {
            new_pos.y += speed * delta_time;
            moved = true;
        }
        if input.is_action_active(&GameAction::MoveDown) {
            new_pos.y -= speed * delta_time;
            moved = true;
        }
        
        if moved {
            self.camera.position = new_pos;
        }
    }
    
    /// ⚡ Update animation speed
    fn update_animation_speed(&mut self) {
        let input = self.input_handler.lock().unwrap();
        
        // Use Action3 and Action4 for speed control since ZoomIn/ZoomOut don't exist
        if input.is_action_active(&GameAction::Action3) {
            self.animation_speed = (self.animation_speed + 0.1).min(3.0);
        }
        if input.is_action_active(&GameAction::Action4) {
            self.animation_speed = (self.animation_speed - 0.1).max(0.1);
        }
    }
    
    /// Render the scene
    fn render_frame(&mut self) {
        let mut batcher = SpriteBatcher::new(1000);
        for sprite in &self.sprites {
            batcher.add_sprite(sprite);
        }

        let batches: Vec<SpriteBatch> = batcher.batches().values().cloned().collect();
        let texture_resources = std::collections::HashMap::new();
        let batch_refs: Vec<&SpriteBatch> = batches.iter().collect();

        if let (Some(renderer), Some(sprite_pipeline)) = (&mut self.renderer, &mut self.sprite_pipeline) {
            match renderer.render_with_sprites(sprite_pipeline, &self.camera, &texture_resources, &batch_refs) {
                Ok(_) => {
                    if self.frame_count == 1 {
                        println!("Rendering started - you should see 7 colored sprites!");
                    }
                }
                Err(e) => {
                    if self.frame_count < 3 {
                        println!("Render error: {}", e);
                    }
                }
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Warn)
        .init();

    let event_loop = EventLoop::new()?;
    let mut app = SpriteDemo::new();

    event_loop.run_app(&mut app)?;

    println!("Sprite demo finished.");

    Ok(())
}