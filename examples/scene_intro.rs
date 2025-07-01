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
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    // Simple game loop (in a real application, you'd use the event loop)
    let dt = 0.016; // 60 FPS
    for _ in 0..10 {  // Run for 10 frames for this example
        app.frame(dt);
    }

    log::info!("Example completed successfully");

    Ok(())
}
