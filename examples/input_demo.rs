//! A demonstration of the improved input system with event queuing and input mapping.

use std::sync::{Arc, Mutex};
use winit::{
    application::ApplicationHandler,
    event::{WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowAttributes, WindowId},
};
use input::prelude::*;
use renderer::prelude::*;

/// A simple system that responds to input actions
struct InputDemoSystem {
    player_x: f32,
    player_y: f32,
    player_color: [f32; 3],
    last_action: String,
}

impl InputDemoSystem {
    fn new() -> Self {
        Self {
            player_x: 0.0,
            player_y: 0.0,
            player_color: [0.0, 1.0, 0.0], // Green by default
            last_action: "None".to_string(),
        }
    }

    fn update(&mut self, input: &InputHandler, dt: f32) {
        let speed = 200.0; // pixels per second
        let mut moved = false;

        // Handle movement actions
        if input.is_action_active(&GameAction::MoveUp) {
            self.player_y += speed * dt;
            moved = true;
            self.last_action = "Moving Up".to_string();
        }
        if input.is_action_active(&GameAction::MoveDown) {
            self.player_y -= speed * dt;
            moved = true;
            self.last_action = "Moving Down".to_string();
        }
        if input.is_action_active(&GameAction::MoveLeft) {
            self.player_x -= speed * dt;
            moved = true;
            self.last_action = "Moving Left".to_string();
        }
        if input.is_action_active(&GameAction::MoveRight) {
            self.player_x += speed * dt;
            moved = true;
            self.last_action = "Moving Right".to_string();
        }

        // Handle action buttons with color changes
        if input.is_action_just_activated(&GameAction::Action1) {
            self.player_color = [1.0, 0.0, 0.0]; // Red
            self.last_action = "Action 1 (Red)".to_string();
            log::info!("Action 1 activated! Color changed to red.");
        }
        if input.is_action_just_activated(&GameAction::Action2) {
            self.player_color = [0.0, 0.0, 1.0]; // Blue
            self.last_action = "Action 2 (Blue)".to_string();
            log::info!("Action 2 activated! Color changed to blue.");
        }
        if input.is_action_just_activated(&GameAction::Action3) {
            self.player_color = [1.0, 1.0, 0.0]; // Yellow
            self.last_action = "Action 3 (Yellow)".to_string();
            log::info!("Action 3 activated! Color changed to yellow.");
        }
        if input.is_action_just_activated(&GameAction::Action4) {
            self.player_color = [1.0, 0.0, 1.0]; // Magenta
            self.last_action = "Action 4 (Magenta)".to_string();
            log::info!("Action 4 activated! Color changed to magenta.");
        }

        // Handle menu action
        if input.is_action_just_activated(&GameAction::Menu) {
            self.last_action = "Menu toggled".to_string();
            log::info!("Menu action activated!");
        }

        // If no movement actions are active, reset to green
        if !moved && !self.last_action.starts_with("Action") && !self.last_action.starts_with("Menu") {
            self.player_color = [0.0, 1.0, 0.0]; // Green
        }

        // Log current action state periodically
        static mut FRAME_COUNT: u32 = 0;
        unsafe {
            FRAME_COUNT += 1;
            if FRAME_COUNT % 60 == 0 { // Log every 60 frames (about once per second at 60 FPS)
                log::debug!("Player position: ({:.1}, {:.1}), Last action: {}", 
                           self.player_x, self.player_y, self.last_action);
            }
        }
    }

    fn get_player_info(&self) -> (f32, f32, [f32; 3]) {
        (self.player_x, self.player_y, self.player_color)
    }
}

/// Input demo application that integrates with the engine
struct InputDemoApp {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    input_handler: InputHandler,
    demo_system: Arc<Mutex<InputDemoSystem>>,
    time: f32,
}

impl InputDemoApp {
    fn new() -> Self {
        Self {
            window: None,
            renderer: None,
            input_handler: InputHandler::new(),
            demo_system: Arc::new(Mutex::new(InputDemoSystem::new())),
            time: 0.0,
        }
    }
}

impl ApplicationHandler<()> for InputDemoApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        log::info!("=== Input System Demo ===");
        log::info!("Controls:");
        log::info!("  Movement: WASD or Arrow Keys");
        log::info!("  Action 1: Space or Left Mouse Button");
        log::info!("  Action 2: Enter or Right Mouse Button");
        log::info!("  Action 3: Left Shift");
        log::info!("  Action 4: Left Control");
        log::info!("  Menu: Escape");
        log::info!("========================");
        
        // Create window with proper attributes
        let window_attributes = WindowAttributes::default()
            .with_title("Insiculous 2D - Input Demo")
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
        // Forward events to input handler
        self.input_handler.handle_window_event(&event);
        
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
        // Update input system
        self.input_handler.update();
        
        // Update demo system
        let dt = 0.016; // Assume 60 FPS
        self.time += dt;
        
        {
            let mut system = self.demo_system.lock().unwrap();
            system.update(&self.input_handler, dt);
            
            let (x, y, _color) = system.get_player_info();
            
            // Log interesting input states
            if self.input_handler.is_key_just_pressed(KeyCode::KeyF) {
                log::info!("Special key F pressed! Player position: ({:.1}, {:.1})", x, y);
            }
            
            if self.input_handler.mouse_wheel_delta() != 0.0 {
                log::info!("Mouse wheel scrolled: {:.1}", self.input_handler.mouse_wheel_delta());
            }
            
            let (mouse_dx, mouse_dy) = self.input_handler.mouse_movement_delta();
            if mouse_dx.abs() > 1.0 || mouse_dy.abs() > 1.0 {
                log::trace!("Mouse moved: ({:.1}, {:.1})", mouse_dx, mouse_dy);
            }
        }
        
        // Request redraw for continuous updates
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

impl InputDemoApp {
    fn render_frame(&mut self) {
        // Get current frame
        if let (Some(renderer), Some(_window)) = (&mut self.renderer, &self.window) {
            match renderer.render() {
                Ok(_) => {
                    // Log frame info periodically
                    static mut FRAME_COUNT: u32 = 0;
                    unsafe {
                        FRAME_COUNT += 1;
                        if FRAME_COUNT % 60 == 0 {
                            let system = self.demo_system.lock().unwrap();
                            let (x, y, _) = system.get_player_info();
                            log::info!("Frame {} - Player position: ({:.1}, {:.1})", FRAME_COUNT / 60, x, y);
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

    log::info!("Starting input demo...");

    // Create event loop
    let event_loop = EventLoop::new()?;
    
    // Create application
    let mut app = InputDemoApp::new();
    
    log::info!("Starting input demo event loop...");
    
    // Run the event loop
    event_loop.run_app(&mut app)?;
    
    log::info!("Input demo completed successfully!");
    
    Ok(())
}