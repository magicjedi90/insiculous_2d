//! A simple example that opens a window, clears the screen, and logs a message.

use engine_core::{Scene, GameLoop, GameLoopConfig, EngineApplication};
use renderer::prelude::*;

/// Main function
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger
    env_logger::init();

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

    // Run the event loop with the application to create the window
    // This will keep the window open and handle events like window closing
    log::info!("Starting event loop");
    renderer::run_with_app(&mut engine_app)
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

    // Get the window from the application
    let window = engine_app.window().ok_or("Window not created")?.clone();

    // Initialize renderer with the window
    let mut renderer = init(window).await.map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

    // Set clear color to a nice blue
    renderer.set_clear_color(0.1, 0.2, 0.3, 1.0);

    // Render initial frame
    if let Err(e) = renderer.render() {
        log::error!("Render error: {}", e);
    }

    // Store the renderer in the application
    engine_app.renderer = Some(renderer);

    log::info!("Example completed successfully");

    Ok(())
}
