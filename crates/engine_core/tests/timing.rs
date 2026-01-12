use engine_core::prelude::*;
use std::thread::sleep;
use std::time::Duration;

#[test]
fn test_timer_creation() {
    // Test creating a new timer
    let timer = Timer::new();

    // Verify timer is initialized with zero delta time and small elapsed time
    assert_eq!(timer.delta_time(), Duration::from_secs(0), 
        "New timer should have zero delta time");
    assert!(timer.elapsed() < Duration::from_millis(10), 
        "New timer should have very small elapsed time (less than 10ms)");
    
    // Verify convenience methods return consistent values
    assert_eq!(timer.delta_seconds(), 0.0, "Delta seconds should be 0.0");
    assert!(timer.elapsed_seconds() < 0.01, "Elapsed seconds should be less than 0.01");
}

#[test]
fn test_timer_reset() {
    // Test resetting the timer
    let mut timer = Timer::new();

    // Sleep a bit to ensure some time passes
    sleep(Duration::from_millis(10));

    // Update the timer to record the elapsed time
    timer.update();
    let elapsed_before_reset = timer.elapsed();
    assert!(elapsed_before_reset > Duration::from_millis(5), 
        "Timer should have recorded elapsed time before reset");

    // Reset the timer
    timer.reset();

    // Verify timer was properly reset
    assert!(timer.elapsed() < Duration::from_millis(10), 
        "Timer should be reset to near-zero elapsed time");
    assert_eq!(timer.delta_time(), Duration::from_secs(0), 
        "Timer should have zero delta time after reset");
    assert!(timer.elapsed() < elapsed_before_reset, 
        "Elapsed time after reset should be less than before");
}

#[test]
fn test_timer_update() {
    // Test updating the timer records delta time correctly
    let mut timer = Timer::new();

    // Sleep a bit to ensure some time passes
    sleep(Duration::from_millis(10));

    // Update the timer
    timer.update();
    
    let first_delta = timer.delta_time();
    let first_elapsed = timer.elapsed();
    
    // Verify update recorded elapsed time
    assert!(first_delta > Duration::from_secs(0), 
        "First update should record non-zero delta time");
    assert!(first_elapsed > Duration::from_secs(0), 
        "First update should record non-zero elapsed time");

    // Sleep a bit more
    sleep(Duration::from_millis(15));

    // Update the timer again
    timer.update();
    let second_delta = timer.delta_time();
    let second_elapsed = timer.elapsed();

    // Verify second update recorded new delta
    assert!(second_delta > Duration::from_secs(0), 
        "Second update should record non-zero delta time");
    assert!(second_elapsed > first_elapsed, 
        "Elapsed time should increase after second update");
    assert!(second_delta > first_delta, 
        "Second delta should be larger due to longer sleep");
}

#[test]
fn test_timer_delta_seconds() {
    // Test getting the delta time in seconds
    let mut timer = Timer::new();

    // Sleep a bit to ensure some time passes
    sleep(Duration::from_millis(100));

    // Update the timer
    timer.update();

    // Verify delta_seconds is reasonable (between 0 and 1 second)
    let delta_seconds = timer.delta_seconds();
    assert!(delta_seconds > 0.0, "Delta seconds should be positive");
    assert!(delta_seconds < 1.0, "Delta seconds should be less than 1 second");
    assert!(delta_seconds > 0.09, "Delta seconds should be at least 90ms");

    // Verify consistency between delta_seconds and delta_time
    let delta_time_seconds = timer.delta_time().as_secs_f32();
    assert!((delta_seconds - delta_time_seconds).abs() < 0.0001, 
        "delta_seconds() should match delta_time().as_secs_f32() within epsilon");
    
    // Verify adding delta_seconds creates cumulative elapsed time
    let elapsed_before = timer.elapsed_seconds();
    sleep(Duration::from_millis(50));
    timer.update();
    let delta2 = timer.delta_seconds();
    let elapsed_after = timer.elapsed_seconds();
    
    assert!((elapsed_after - (elapsed_before + delta2)).abs() < 0.0001, 
        "Elapsed time should accumulate delta seconds within epsilon");
}

#[test]
fn test_timer_elapsed_seconds() {
    // Test getting the elapsed time in seconds
    let mut timer = Timer::new();

    // Sleep a bit to ensure some time passes
    sleep(Duration::from_millis(100));

    // Update the timer
    timer.update();

    // Verify elapsed_seconds is reasonable
    let elapsed_seconds = timer.elapsed_seconds();
    assert!(elapsed_seconds > 0.0, "Elapsed seconds should be positive");
    assert!(elapsed_seconds < 1.0, "Elapsed seconds should be less than 1 second");
    assert!(elapsed_seconds > 0.09, "Elapsed seconds should be at least 90ms");

    // Verify consistency between elapsed_seconds and elapsed
    let elapsed_time_seconds = timer.elapsed().as_secs_f32();
    assert_eq!(elapsed_seconds, elapsed_time_seconds,
        "elapsed_seconds() should match elapsed().as_secs_f32()");
    
    // Verify elapsed_seconds increases with updates
    let before_update = timer.elapsed_seconds();
    sleep(Duration::from_millis(50));
    timer.update();
    let after_update = timer.elapsed_seconds();
    
    assert!(after_update > before_update, 
        "Elapsed seconds should increase after update");
}
