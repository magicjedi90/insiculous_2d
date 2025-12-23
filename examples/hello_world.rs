//! A simple example that opens a window, clears the screen, and logs a message.

use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::{WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowAttributes, WindowId},
};
use renderer::prelude::*;

/// Simple application that demonstrates basic window and rendering setup
struct HelloWorldApp {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    time: f32,
}

impl HelloWorldApp {
    fn new() -> Self {
        Self {
            window: None,
            renderer: None,
            time: 0.0,
        }
    }
}

impl ApplicationHandler<()> for HelloWorldApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        log::info!("Application resumed - creating window");
        
        // Create window with proper attributes
        let window_attributes = WindowAttributes::default()
            .with_title("Insiculous 2D - Hello World")
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
        // Request redraw for continuous animation
        if let Some(window) = &self.window {
            window.request_redraw();
        }
        
        // Update time for animation
        self.time += 0.016; // Assume 60 FPS for simple animation
    }
}

impl HelloWorldApp {
    fn render_frame(&mut self) {
        // Set animated clear color
        let r = (self.time * 0.5).sin() * 0.5 + 0.5;
        let g = (self.time * 0.7).sin() * 0.5 + 0.5;
        let b = (self.time * 0.3).sin() * 0.5 + 0.5;
        
        // For hello_world, we use the renderer's built-in render method
        // which just clears the screen with the current clear color
        if let Some(renderer) = &mut self.renderer {
            renderer.set_clear_color(r as f64, g as f64, b as f64, 1.0);
            
            match renderer.render() {
                Ok(_) => {
                    // Log frame info periodically
                    static mut FRAME_COUNT: u32 = 0;
                    unsafe {
                        FRAME_COUNT += 1;
                        if FRAME_COUNT % 60 == 0 {
                            log::info!("Frame {} rendered - Color: ({:.2}, {:.2}, {:.2})", FRAME_COUNT / 60, r, g, b);
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
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    log::info!("=== Insiculous 2D - Hello World Demo ===");
    log::info!("This demo opens a window and displays an animated colored background.");
    log::info!("Close the window to exit.");
    log::info!("=========================================");

    // Create event loop
    let event_loop = EventLoop::new()?;
    
    // Create application
    let mut app = HelloWorldApp::new();
    
    log::info!("Starting event loop...");
    
    // Run the event loop
    event_loop.run_app(&mut app)?;
    
    log::info!("Hello World demo completed successfully!");
    
    Ok(())
}