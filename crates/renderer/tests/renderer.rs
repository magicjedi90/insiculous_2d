use renderer::prelude::*;

#[ignore] // Ignore by default as it requires a display and GPU
async fn test_renderer_creation() {
    // Test creating a new renderer
    let result = renderer::init().await;

    // TODO: Assert that the renderer was created successfully
    // This test is ignored by default as it requires a display and GPU
    // In a real test environment with the necessary hardware, we would:
    // assert!(result.is_ok());
    // let renderer = result.unwrap();
    // assert that renderer properties are as expected
}

#[ignore] // Ignore by default as it requires a display and GPU
async fn test_renderer_clear_color() {
    // Test setting the clear color
    let result = renderer::init().await;

    // TODO: Assert that we can set the clear color
    // This test is ignored by default as it requires a display and GPU
    // In a real test environment with the necessary hardware, we would:
    // assert!(result.is_ok());
    // let mut renderer = result.unwrap();
    // renderer.set_clear_color(1.0, 0.0, 0.0, 1.0);
    // assert that the clear color was set correctly
}

#[ignore] // Ignore by default as it requires a display and GPU
async fn test_renderer_render() {
    // Test rendering a frame
    let result = renderer::init().await;

    // TODO: Assert that we can render a frame
    // This test is ignored by default as it requires a display and GPU
    // In a real test environment with the necessary hardware, we would:
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
    assert_eq!(config.title, "insiculous_2d");
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
