use engine_core::prelude::*;

#[test]
fn test_init() {
    // Test initializing the engine core
    let result = init();
    
    // Verify initialization was successful  
    assert!(result.is_ok(), "Engine initialization should succeed");
    
    // Verify no error was returned
    match result {
        Ok(()) => {}, // Success - no value returned
        Err(e) => panic!("Engine initialization failed: {e}"),
    }
}

#[test]
fn test_engine_error() {
    // Test creating an engine error
    let error = EngineError::InitializationError("Test error".to_string());

    // Verify the error is correctly created with the right message
    match error {
        EngineError::InitializationError(ref msg) => {
            assert_eq!(msg, "Test error", "Error message should match");
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
