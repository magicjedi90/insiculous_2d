//! A demonstration of the improved input system with event queuing and input mapping.

use engine_core::{Scene, EngineApplication};
use input::prelude::*;

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

/// Main function
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    log::info!("=== Input System Demo ===");
    log::info!("Controls:");
    log::info!("  Movement: WASD or Arrow Keys");
    log::info!("  Action 1: Space or Left Mouse Button");
    log::info!("  Action 2: Enter or Right Mouse Button");
    log::info!("  Action 3: Left Shift");
    log::info!("  Action 4: Left Control");
    log::info!("  Menu: Escape");
    log::info!("========================");

    // Create a scene with our input demo system
    let mut scene = Scene::new("Input Demo Scene");
    
    // Create our demo system
    let input_demo_system = InputDemoSystem::new();
    
    // Add the system to the scene (we'll access it through the input handler)
    // For this demo, we'll just store it in a simple way
    let demo_system = std::sync::Arc::new(std::sync::Mutex::new(input_demo_system));
    let demo_system_clone = demo_system.clone();

    // Create application with the scene
    let mut app = EngineApplication::with_scene(scene);

    // Add a custom update function that demonstrates the input system
    let custom_update = move |input: &InputHandler, dt: f32| {
        let mut system = demo_system_clone.lock().unwrap();
        system.update(input, dt);
        
        let (x, y, color) = system.get_player_info();
        
        // Log interesting input states
        if input.is_key_just_pressed(KeyCode::KeyF) {
            log::info!("Special key F pressed! Player position: ({:.1}, {:.1})", x, y);
        }
        
        if input.mouse_wheel_delta() != 0.0 {
            log::info!("Mouse wheel scrolled: {:.1}", input.mouse_wheel_delta());
        }
        
        let (mouse_dx, mouse_dy) = input.mouse_movement_delta();
        if mouse_dx.abs() > 1.0 || mouse_dy.abs() > 1.0 {
            log::trace!("Mouse moved: ({:.1}, {:.1})", mouse_dx, mouse_dy);
        }
    };

    // Store the custom update function reference (this is a simplified approach)
    // In a real implementation, you'd integrate this properly with the ECS system
    log::info!("Input demo system initialized");

    // Start the game loop
    if let Err(e) = app.start_game_loop() {
        log::error!("Failed to start game loop: {}", e);
        return Err(format!("Failed to start game loop: {}", e).into());
    }

    // Run the event loop with the application
    log::info!("Starting input demo event loop");
    renderer::run_with_app(&mut app)
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

    log::info!("Input demo completed successfully");

    Ok(())
}