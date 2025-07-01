//! A simple example that opens a window, clears the screen, and logs a message.

use engine_core::World;

/// Main function
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger
    env_logger::init();

    // Log startup message
    log::info!("Engine booted");

    // Initialize engine core
    engine_core::init().map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

    // Create a world
    let mut world = World::new("Main World");
    world.initialize();

    // Initialize renderer
    let mut renderer = renderer::init().await.map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

    // Initialize ECS
    let _ecs_world = ecs::init().map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

    // Initialize input
    let _input = input::init().map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

    // Set clear color to a nice blue
    renderer.set_clear_color(0.1, 0.2, 0.3, 1.0);

    // Render a frame
    if let Err(e) = renderer.render() {
        log::error!("Render error: {}", e);
    }

    // The renderer already created an event loop and window internally,
    // so we don't need to create another one or run an event loop here.
    // In a real application, you would use the renderer's window to handle events.
    log::info!("Example completed successfully");

    Ok(())
}
