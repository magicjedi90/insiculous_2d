use renderer::prelude::*;

#[test]
fn test_window_config_default() {
    // Test the default window configuration
    let config = WindowConfig::default();

    // TODO: Assert that the default values are as expected
    assert_eq!(config.title, "insiculous_2d v0.1");
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
