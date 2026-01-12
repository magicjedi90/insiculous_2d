//! Integration test for scene running in the application.

use engine_core::{Scene, EngineApplication};

#[test]
fn test_scene_runs() {
    // Create a new scene
    let scene = Scene::new("Test Scene");

    // Create application with the scene
    let mut app = EngineApplication::with_scene(scene);

    // Run a single frame
    let _ = app.frame(0.016);

    // If we got here without panicking, the test passes
}