use engine_core::prelude::*;

#[test]
fn test_game_loop_creation() {
    // Test creating a new game loop with default config
    let config = GameLoopConfig::default();
    let game_loop = GameLoop::new(config);

    // TODO: Assert that the game loop is properly initialized
    assert!(!game_loop.is_running());
}

#[test]
fn test_game_loop_custom_config() {
    // Test creating a new game loop with custom config
    let config = GameLoopConfig {
        target_fps: 30,
        fixed_timestep: false,
    };
    let game_loop = GameLoop::new(config);

    // TODO: Assert that the game loop has the correct configuration
    // We can check the target frame duration which is derived from target_fps
    let expected_duration = std::time::Duration::from_secs_f64(1.0 / 30.0);
    assert_eq!(game_loop.target_frame_duration(), expected_duration);
}

#[test]
fn test_game_loop_start_stop() {
    // Test starting and stopping the game loop
    let config = GameLoopConfig::default();
    let mut game_loop = GameLoop::new(config);

    // Initially the game loop should not be running
    assert!(!game_loop.is_running());

    // Start the game loop
    let result = game_loop.start();

    // TODO: Assert that the game loop started successfully
    assert!(result.is_ok());
    assert!(game_loop.is_running());

    // Stop the game loop
    game_loop.stop();

    // TODO: Assert that the game loop is stopped
    assert!(!game_loop.is_running());
}

#[test]
fn test_game_loop_target_frame_duration() {
    // Test the target frame duration calculation
    let config = GameLoopConfig {
        target_fps: 60,
        fixed_timestep: true,
    };
    let game_loop = GameLoop::new(config);

    // TODO: Assert that the target frame duration is correct
    let expected_duration = std::time::Duration::from_secs_f64(1.0 / 60.0);
    assert_eq!(game_loop.target_frame_duration(), expected_duration);
}
