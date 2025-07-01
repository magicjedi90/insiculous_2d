use renderer::prelude::*;
use winit::event_loop::EventLoop;

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
    let result = create_window(&config, &event_loop);

    // TODO: Assert that the window was created successfully
    // This test is ignored by default as it requires a display
    // In a real test environment with a display, we would:
    // assert!(result.is_ok());
    // let window = result.unwrap();
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
    let result = create_window(&config, &event_loop);

    // TODO: Assert that the window was created successfully with custom config
    // This test is ignored by default as it requires a display
    // In a real test environment with a display, we would:
    // assert!(result.is_ok());
    // let window = result.unwrap();
    // assert that window properties match the custom configuration
}
