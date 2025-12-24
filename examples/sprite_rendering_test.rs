//! Simple test to validate that sprite rendering is working correctly
//! 
//! This example creates a window and renders some colored rectangles to verify
//! that the sprite rendering pipeline is functioning properly.

use winit::{
    application::ApplicationHandler,
    event::{WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowAttributes, WindowId},
};
use renderer::prelude::*;
use glam::{Vec2, Vec4};
use std::sync::Arc;

/// Test application that demonstrates sprite rendering
struct SpriteTestApp {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    sprite_pipeline: Option<SpritePipeline>,
    camera: Camera2D,
    time: f32,
    frame_count: u32,
    sprites: Vec<Sprite>,
}

impl SpriteTestApp {
    fn new() -> Self {
        Self {
            window: None,
            renderer: None,
            sprite_pipeline: None,
            camera: Camera2D::default(),
            time: 0.0,
            frame_count: 0,
            sprites: Vec::new(),
        }
    }
    
    fn create_test_sprites(&self) -> Vec<Sprite> {
        log::info!("Creating test sprites...");
        
        vec![
            // Red sprite (top-left)
            Sprite::new(TextureHandle { id: 0 })
                .with_position(Vec2::new(-200.0, 100.0))
                .with_color(Vec4::new(1.0, 0.0, 0.0, 1.0))
                .with_scale(Vec2::new(100.0, 100.0)),
            
            // Green sprite (top-right)  
            Sprite::new(TextureHandle { id: 0 })
                .with_position(Vec2::new(200.0, 100.0))
                .with_color(Vec4::new(0.0, 1.0, 0.0, 1.0))
                .with_scale(Vec2::new(100.0, 100.0)),
            
            // Blue sprite (bottom-left)
            Sprite::new(TextureHandle { id: 0 })
                .with_position(Vec2::new(-200.0, -100.0))
                .with_color(Vec4::new(0.0, 0.0, 1.0, 1.0))
                .with_scale(Vec2::new(100.0, 100.0)),
            
            // Yellow sprite (bottom-right)
            Sprite::new(TextureHandle { id: 0 })
                .with_position(Vec2::new(200.0, -100.0))
                .with_color(Vec4::new(1.0, 1.0, 0.0, 1.0))
                .with_scale(Vec2::new(100.0, 100.0)),
            
            // White sprite in the center
            Sprite::new(TextureHandle { id: 0 })
                .with_position(Vec2::new(0.0, 0.0))
                .with_color(Vec4::new(1.0, 1.0, 1.0, 1.0))
                .with_scale(Vec2::new(150.0, 150.0)),
        ]
    }
}

impl ApplicationHandler<()> for SpriteTestApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        log::info!("Application resumed - creating window");
        
        // Create window
        let window_attributes = WindowAttributes::default()
            .with_title("Insiculous 2D - Sprite Rendering Test")
            .with_inner_size(winit::dpi::LogicalSize::new(800, 600));
            
        let window = match event_loop.create_window(window_attributes) {
            Ok(window) => {
                log::info!("Window created successfully");
                Arc::new(window)
            }
            Err(e) => {
                log::error!("Failed to create window: {}", e);
                event_loop.exit();
                return;
            }
        };
        
        self.window = Some(window.clone());
        
        // Initialize renderer
        match pollster::block_on(renderer::init(window)) {
            Ok(renderer) => {
                log::info!("Renderer initialized successfully");
                
                // Create sprite pipeline
                let sprite_pipeline = SpritePipeline::new(renderer.device(), 1000);
                
                // Update camera viewport
                self.camera.viewport_size = Vec2::new(800.0, 600.0);
                
                // Create test sprites
                self.sprites = self.create_test_sprites();
                
                self.renderer = Some(renderer);
                self.sprite_pipeline = Some(sprite_pipeline);
                
                log::info!("Sprite rendering setup complete!");
            }
            Err(e) => {
                log::error!("Failed to initialize renderer: {}", e);
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
                log::info!("Window close requested - shutting down");
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                log::info!("Window resized to {}x{}", size.width, size.height);
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(size.width, size.height);
                }
                // Update camera viewport
                self.camera.viewport_size = Vec2::new(size.width as f32, size.height as f32);
            }
            WindowEvent::RedrawRequested => {
                self.render_frame();
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
        self.time += 0.016; // Assume 60 FPS for simple animation
    }
}

impl SpriteTestApp {
    fn render_frame(&mut self) {
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
        
        // Render
        if let (Some(renderer), Some(sprite_pipeline)) = (&mut self.renderer, &self.sprite_pipeline) {
            // Update camera (slight rotation for visual interest)
            self.camera.rotation = (self.time * 0.5).sin() * 0.1;
            
            // Render with sprites
            match renderer.render_with_sprites(sprite_pipeline, &self.camera, &texture_resources, &batch_refs) {
                Ok(_) => {
                    self.frame_count += 1;
                    if self.frame_count % 60 == 0 {
                        log::info!("Rendered frame {} - {} sprites in {} batches", 
                            self.frame_count / 60, batch_refs.len() * 5, batch_refs.len());
                    }
                }
                Err(e) => {
                    log::error!("Failed to render frame: {}", e);
                }
            }
        }
    }
}

/// Main function
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    log::info!("=== Insiculous 2D - Sprite Rendering Test ===");
    log::info!("This demo tests the sprite rendering pipeline.");
    log::info!("You should see colored rectangles on screen.");
    log::info!("Close the window to exit.");
    log::info!("==============================================");

    // Create event loop
    let event_loop = EventLoop::new()?;
    
    // Create application
    let mut app = SpriteTestApp::new();
    
    log::info!("Starting event loop...");
    
    // Run the event loop
    event_loop.run_app(&mut app)?;
    
    log::info!("Sprite rendering test completed successfully!");
    
    Ok(())
}