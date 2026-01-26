use engine_core::prelude::*;

#[test]
fn test_game_loop_creation() {
    // Test creating a new game loop with default config
    let config = GameLoopConfig::default();
    let game_loop = GameLoop::new(config);

    // Verify the game loop is properly initialized but not running
    assert!(!game_loop.is_running(), "New game loop should not be running");
    
    // Verify default FPS is 60
    let expected_duration = std::time::Duration::from_secs_f64(1.0 / 60.0);
    assert_eq!(game_loop.target_frame_duration(), expected_duration, 
        "Default target frame duration should be 60 FPS");
}

#[test]
fn test_game_loop_custom_config() {
    // Test creating a new game loop with custom config
    let config = GameLoopConfig {
        target_fps: 30,
        fixed_timestep: false,
    };
    let game_loop = GameLoop::new(config);

    // Verify custom FPS is applied
    let expected_duration = std::time::Duration::from_secs_f64(1.0 / 30.0);
    assert_eq!(game_loop.target_frame_duration(), expected_duration,
        "Custom target frame duration should match 30 FPS");
    
    // Verify the game loop is not running initially
    assert!(!game_loop.is_running(), "New game loop should not be running");
}

#[test]
fn test_game_loop_start_stop() {
    // Test starting and stopping the game loop
    let config = GameLoopConfig::default();
    let mut game_loop = GameLoop::new(config);

    // Initially the game loop should not be running
    assert!(!game_loop.is_running(), "Game loop should start in stopped state");

    // Start the game loop
    let result = game_loop.start();

    // Verify the game loop started successfully
    assert!(result.is_ok(), "Starting game loop should succeed");
    assert!(game_loop.is_running(), "Game loop should be running after start()");

    // Stop the game loop
    game_loop.stop();

    // Verify the game loop is stopped
    assert!(!game_loop.is_running(), "Game loop should be stopped after stop()");
}

#[test]
fn test_game_loop_target_frame_duration() {
    // Test the target frame duration calculation for various FPS values
    let test_cases = vec![
        (30, 1.0 / 30.0),
        (60, 1.0 / 60.0),
        (120, 1.0 / 120.0),
    ];
    
    for (fps, expected_secs) in test_cases {
        let config = GameLoopConfig {
            target_fps: fps,
            fixed_timestep: true,
        };
        let game_loop = GameLoop::new(config);
        
        let expected_duration = std::time::Duration::from_secs_f64(expected_secs);
        assert_eq!(game_loop.target_frame_duration(), expected_duration,
            "Target frame duration should be correct for {} FPS", fps);
    }
}
