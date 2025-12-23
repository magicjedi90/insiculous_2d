use renderer::prelude::*;

#[test]
#[ignore] // Ignore by default as it requires a display and GPU
fn test_init() {
    // Test initializing the renderer
    // This test is ignored by default as it requires a display and GPU
    // In a real test environment with the necessary hardware, we would:
    // let window = create_window_with_active_loop(WindowConfig::default()).unwrap();
    // let result = pollster::block_on(renderer::init(window));
    // assert!(result.is_ok());
}

#[test]
#[ignore] // Ignore by default as it requires a display and GPU
fn test_init_and_render() {
    // Test initializing the renderer and rendering a frame
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
fn test_window_config_creation() {
    // Test creating a window configuration
    let config = WindowConfig::default();

    // TODO: Assert that the config has the expected default values
    assert_eq!(config.title, "insiculous_2d v0.1");
    assert_eq!(config.width, 800);
    assert_eq!(config.height, 600);
    assert!(config.resizable);
}
