use renderer::prelude::*;

#[test]
#[ignore] // Ignore by default as it requires a display and GPU
fn test_renderer_creation() {
    // Test creating a new renderer
    // This test is ignored by default as it requires a display and GPU
    // In a real test environment with the necessary hardware, we would:
    // let window = create_window_with_active_loop(WindowConfig::default()).unwrap();
    // let result = pollster::block_on(renderer::init(window));
    // assert!(result.is_ok());
}

#[test]
#[ignore] // Ignore by default as it requires a display and GPU
fn test_renderer_clear_color() {
    // Test setting the clear color
    // This test is ignored by default as it requires a display and GPU
    // In a real test environment with the necessary hardware, we would:
    // let window = create_window_with_active_loop(WindowConfig::default()).unwrap();
    // let result = pollster::block_on(renderer::init(window));
    // assert!(result.is_ok());
    // let mut renderer = result.unwrap();
    // renderer.set_clear_color(1.0, 0.0, 0.0, 1.0);
}

#[test]
#[ignore] // Ignore by default as it requires a display and GPU
fn test_renderer_render() {
    // Test rendering a frame
    // This test is ignored by default as it requires a display and GPU
    // In a real test environment with the necessary hardware, we would:
    // let window = create_window_with_active_loop(WindowConfig::default()).unwrap();
    // let result = pollster::block_on(renderer::init(window));
    // assert!(result.is_ok());
    // let renderer = result.unwrap();
    // let render_result = renderer.render();
    // assert!(render_result.is_ok());
}

#[test]
fn test_window_config() {
    // Test creating a window configuration
    let config = WindowConfig::default();

    // TODO: Assert that the config has the expected default values
    assert_eq!(config.title, "insiculous_2d v0.1");
    assert_eq!(config.width, 800);
    assert_eq!(config.height, 600);
    assert!(config.resizable);

    // Test creating a custom window configuration
    let custom_config = WindowConfig {
        title: "Test Window".to_string(),
        width: 1024,
        height: 768,
        resizable: false,
    };

    // TODO: Assert that the custom config has the expected values
    assert_eq!(custom_config.title, "Test Window");
    assert_eq!(custom_config.width, 1024);
    assert_eq!(custom_config.height, 768);
    assert!(!custom_config.resizable);
}
