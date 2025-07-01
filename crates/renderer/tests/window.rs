use renderer::prelude::*;
use winit::{
    event_loop::{EventLoop, ActiveEventLoop},
    application::ApplicationHandler
};
use engine_core::{World, GameLoop, GameLoopConfig, EngineApplication};

#[test]
fn test_window_config_default() {
    // Test the default window configuration
    let config = WindowConfig::default();

    // TODO: Assert that the default values are as expected
    assert_eq!(config.title, "insiculous_2d");
    assert_eq!(config.width, 800);
    assert_eq!(config.height, 600);
    assert!(config.resizable);
}

#[test]
fn test_window_config_custom() {
    // Test creating a custom window configuration
    let config = WindowConfig {
        title: "Test Window".to_string(),
        width: 1024,
        height: 768,
        resizable: false,
    };

    // TODO: Assert that the custom values are as expected
    assert_eq!(config.title, "Test Window");
    assert_eq!(config.width, 1024);
    assert_eq!(config.height, 768);
    assert!(!config.resizable);
}

#[test]
#[ignore] // Ignore by default as it requires a display
fn test_create_window() {
    // Test creating a window
    let event_loop = EventLoop::new().unwrap();
    let config = WindowConfig::default();

    // Create a world and game loop for testing
    let mut world = World::new("Test World");
    world.initialize();
    let game_loop = GameLoop::new(GameLoopConfig::default());
    let engine_app = EngineApplication::new(world, game_loop);
    let mut app = RendererApplication::new(config, engine_app);

    // Run the event loop to create the window
    event_loop.run_app(&mut app).unwrap();

    // TODO: Assert that the window was created successfully
    // This test is ignored by default as it requires a display
    // In a real test environment with a display, we would:
    assert!(app.window().is_some());
    // let window = app.window().unwrap();
    // assert that window properties match the configuration
}

#[test]
#[ignore] // Ignore by default as it requires a display
fn test_create_window_custom() {
    // Test creating a window with custom configuration
    let event_loop = EventLoop::new().unwrap();
    let config = WindowConfig {
        title: "Test Window".to_string(),
        width: 1024,
        height: 768,
        resizable: false,
    };

    // Create a world and game loop for testing
    let mut world = World::new("Test Custom World");
    world.initialize();
    let game_loop = GameLoop::new(GameLoopConfig::default());
    let engine_app = EngineApplication::new(world, game_loop);
    let mut app = RendererApplication::new(config, engine_app);

    // Run the event loop to create the window
    event_loop.run_app(&mut app).unwrap();

    // TODO: Assert that the window was created successfully with custom config
    // This test is ignored by default as it requires a display
    // In a real test environment with a display, we would:
    assert!(app.window().is_some());
    // let window = app.window().unwrap();
    // assert that window properties match the custom configuration
}
