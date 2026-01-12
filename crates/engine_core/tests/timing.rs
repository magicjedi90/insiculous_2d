use engine_core::prelude::*;
use std::thread::sleep;
use std::time::Duration;

#[test]
fn test_timer_creation() {
    // Test creating a new timer
    let timer = Timer::new();

    // TODO: Assert that the timer is properly initialized
    // Since the timer starts at the current time, we can't directly test the start_time
    // But we can check that the elapsed time is very small
    assert!(timer.elapsed() < Duration::from_millis(10));
    assert!(timer.delta_time() == Duration::from_secs(0));
}

#[test]
fn test_timer_reset() {
    // Test resetting the timer
    let mut timer = Timer::new();

    // Sleep a bit to ensure some time passes
    sleep(Duration::from_millis(10));

    // Update the timer to record the elapsed time
    timer.update();

    // There should be some elapsed time now
    assert!(timer.elapsed() > Duration::from_secs(0));

    // Reset the timer
    timer.reset();

    // TODO: Assert that the timer was reset
    assert!(timer.elapsed() < Duration::from_millis(10));
    assert!(timer.delta_time() == Duration::from_secs(0));
}

#[test]
fn test_timer_update() {
    // Test updating the timer
    let mut timer = Timer::new();

    // Sleep a bit to ensure some time passes
    sleep(Duration::from_millis(10));

    // Update the timer
    timer.update();

    // TODO: Assert that the timer was updated
    assert!(timer.delta_time() > Duration::from_secs(0));
    assert!(timer.elapsed() > Duration::from_secs(0));

    // Sleep a bit more
    sleep(Duration::from_millis(10));

    // Update the timer again
    let previous_elapsed = timer.elapsed();
    timer.update();

    // TODO: Assert that the timer was updated again
    assert!(timer.delta_time() > Duration::from_secs(0));
    assert!(timer.elapsed() > previous_elapsed);
}

#[test]
fn test_timer_delta_seconds() {
    // Test getting the delta time in seconds
    let mut timer = Timer::new();

    // Sleep a bit to ensure some time passes
    sleep(Duration::from_millis(100));

    // Update the timer
    timer.update();

    // TODO: Assert that the delta seconds is correct
    let delta_seconds = timer.delta_seconds();
    assert!(delta_seconds > 0.0);
    assert!(delta_seconds < 1.0); // Should be much less than 1 second

    // Check that delta_seconds matches delta_time
    let delta_time_seconds = timer.delta_time().as_secs_f32();
    assert_eq!(delta_seconds, delta_time_seconds);
}

#[test]
fn test_timer_elapsed_seconds() {
    // Test getting the elapsed time in seconds
    let mut timer = Timer::new();

    // Sleep a bit to ensure some time passes
    sleep(Duration::from_millis(100));

    // Update the timer
    timer.update();

    // TODO: Assert that the elapsed seconds is correct
    let elapsed_seconds = timer.elapsed_seconds();
    assert!(elapsed_seconds > 0.0);
    assert!(elapsed_seconds < 1.0); // Should be much less than 1 second

    // Check that elapsed_seconds matches elapsed
    let elapsed_time_seconds = timer.elapsed().as_secs_f32();
    assert_eq!(elapsed_seconds, elapsed_time_seconds);
}
