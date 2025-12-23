//! A minimal example showing how to create a scene and run it in the application.

use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::{WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowAttributes, WindowId},
};
use renderer::prelude::*;

// Simple Transform component
#[derive(Default)]
struct Transform {
    x: f32,
    y: f32,
    rotation: f32,
}

// Simple Sprite component
#[derive(Default)]
struct Sprite {
    texture_id: u32,
    color: [f32; 4],
}

/// Scene demo application that shows basic ECS usage
struct SceneIntroApp {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    time: f32,
}

impl SceneIntroApp {
    fn new() -> Self {
        Self {
            window: None,
            renderer: None,
            time: 0.0,
        }
    }
}

impl ApplicationHandler<()> for SceneIntroApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        log::info!("=== Scene Introduction Demo ===");
        log::info!("This demo shows basic scene and ECS functionality.");
        log::info!("Close the window to exit.");
        log::info!("================================");
        
        // Create window with proper attributes
        let window_attributes = WindowAttributes::default()
            .with_title("Insiculous 2D - Scene Intro")
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
                self.renderer = Some(renderer);
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
            }
            WindowEvent::RedrawRequested => {
                self.render_frame();
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // Update time for animation
        self.time += 0.016; // Assume 60 FPS
        
        // Simulate scene updates
        self.update_scene();
        
        // Request redraw for continuous animation
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

impl SceneIntroApp {
    fn update_scene(&mut self) {
        // Simulate some basic scene logic
        // In a real implementation, this would update entities, systems, etc.
        
        static mut FRAME_COUNT: u32 = 0;
        unsafe {
            FRAME_COUNT += 1;
            if FRAME_COUNT % 120 == 0 { // Every 2 seconds at 60 FPS
                log::info!("Scene update - Frame {}", FRAME_COUNT);
                log::info!("  - Simulating entity updates");
                log::info!("  - Processing component data");
                log::info!("  - Running systems");
            }
        }
    }

    fn render_frame(&mut self) {
        // Set a nice scene-like color (sky blue)
        let scene_color = wgpu::Color {
            r: 0.529, // 135/255
            g: 0.808, // 206/255  
            b: 0.922, // 235/255
            a: 1.0,
        };
        
        if let Some(renderer) = &mut self.renderer {
            renderer.set_clear_color(scene_color.r, scene_color.g, scene_color.b, scene_color.a);
            
            match renderer.render() {
                Ok(_) => {
                    // Log frame info periodically
                    static mut FRAME_COUNT: u32 = 0;
                    unsafe {
                        FRAME_COUNT += 1;
                        if FRAME_COUNT % 60 == 0 {
                            log::info!("Scene frame {} rendered", FRAME_COUNT / 60);
                        }
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
    env_logger::init();

    log::info!("=== Scene Introduction Demo ===");
    log::info!("This demo shows basic scene functionality.");
    log::info!("It demonstrates:");
    log::info!("  - Window creation and management");
    log::info!("  - Renderer initialization");
    log::info!("  - Basic scene update loop");
    log::info!("  - Simple rendering");
    log::info!("Close the window to exit.");
    log::info!("================================");

    // Create event loop
    let event_loop = EventLoop::new()?;
    
    // Create application
    let mut app = SceneIntroApp::new();
    
    log::info!("Starting scene introduction demo...");
    
    // Run the event loop
    event_loop.run_app(&mut app)?;
    
    log::info!("Scene introduction demo completed successfully!");
    
    Ok(())
}