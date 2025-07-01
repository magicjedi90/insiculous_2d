use renderer::prelude::*;

#[ignore] // Ignore by default as it requires a display and GPU
async fn test_init() {
    // Test initializing the renderer
    let result = renderer::init().await;

    // TODO: Assert that initialization was successful
    // This test is ignored by default as it requires a display and GPU
    // In a real test environment with the necessary hardware, we would:
    // assert!(result.is_ok());
    // let renderer = result.unwrap();
    // assert that renderer is properly initialized
}

#[ignore] // Ignore by default as it requires a display and GPU
async fn test_init_and_render() {
    // Test initializing the renderer and rendering a frame
    let result = renderer::init().await;

    // TODO: Assert that initialization and rendering were successful
    // This test is ignored by default as it requires a display and GPU
    // In a real test environment with the necessary hardware, we would:
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
    assert_eq!(config.title, "insiculous_2d");
    assert_eq!(config.width, 800);
    assert_eq!(config.height, 600);
    assert!(config.resizable);
}
