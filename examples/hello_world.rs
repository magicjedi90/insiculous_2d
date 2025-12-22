//! A simple example that opens a window, clears the screen, and logs a message.

use engine_core::{Scene, GameLoop, GameLoopConfig, EngineApplication};
use renderer::prelude::*;

/// Main function
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger
    // env_logger::init();
    env_logger::Builder::from_default_env().filter_level(log::LevelFilter::Debug).init();
    // Log startup message
    log::info!("Engine booted");

    // Initialize engine core
    engine_core::init().map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

    // Create a scene
    let mut world = Scene::new("Main Scene");
    world.initialize();

    // Initialize ECS
    let _ecs_world = ecs::init().map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

    // Initialize input
    let _input = input::init().map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

    // Create game loop
    let game_loop = GameLoop::new(GameLoopConfig::default());

    // Create engine application with custom window config
    let window_config = WindowConfig::default();
    let mut engine_app = EngineApplication::new(world, game_loop)
        .with_window_config(window_config);

    // Start the game loop before running the event loop
    // This ensures the game loop is running alongside the event loop
    if let Err(e) = engine_app.start_game_loop() {
        log::error!("Failed to start game loop: {}", e);
        return Err(format!("Failed to start game loop: {}", e).into());
    }

    // Run the event loop with the application
    // This will create the window, initialize the renderer, and handle events
    log::info!("Starting event loop");
    run_with_app(&mut engine_app)
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

    log::info!("Example completed successfully");

    Ok(())
}