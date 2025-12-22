//! A minimal example showing how to create a scene and run it in the application.

use engine_core::{Scene, EngineApplication};

// Simple Transform component
#[derive(Default)]
struct Transform {
    // Position, rotation, scale, etc. would go here
}

// Simple Sprite component
#[derive(Default)]
struct Sprite {
    // Texture, color, size, etc. would go here
}

/// Main function
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger
    env_logger::init();

    // Create a new scene
    let mut scene = Scene::new("Example Scene");

    // Create an entity and add components
    let entity_id = scene.world.create_entity();
    scene.world.add_component(&entity_id, Transform::default()).unwrap();
    scene.world.add_component(&entity_id, Sprite::default()).unwrap();

    // Create application with the scene
    let mut app = EngineApplication::with_scene(scene);

    // Start the game loop
    if let Err(e) = app.start_game_loop() {
        log::error!("Failed to start game loop: {}", e);
        return Err(format!("Failed to start game loop: {}", e).into());
    }

    // Run the event loop with the application
    // This will create the window, initialize the renderer, and handle events
    log::info!("Starting event loop");
    renderer::run_with_app(&mut app)
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

    log::info!("Example completed successfully");

    Ok(())
}