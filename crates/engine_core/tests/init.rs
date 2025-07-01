use engine_core::prelude::*;

#[test]
fn test_init() {
    // Test initializing the engine core
    let result = engine_core::init();

    // TODO: Assert that initialization was successful
    assert!(result.is_ok());
}

#[test]
fn test_engine_error() {
    // Test creating an engine error
    let error = EngineError::InitializationError("Test error".to_string());

    // TODO: Assert that the error is correctly created
    match error {
        EngineError::InitializationError(ref msg) => {
            assert_eq!(msg, "Test error");
        }
        _ => panic!("Wrong error type"),
    }

    // Test the Display implementation
    let error_string = format!("{error}");
    assert!(error_string.contains("Failed to initialize engine: Test error"));

    // Test another error type
    let error = EngineError::GameLoopError("Game loop test error".to_string());
    match error {
        EngineError::GameLoopError(ref msg) => {
            assert_eq!(msg, "Game loop test error");
        }
        _ => panic!("Wrong error type"),
    }

    // Test the Display implementation
    let error_string = format!("{error}");
    assert!(error_string.contains("Game loop error: Game loop test error"));
}
