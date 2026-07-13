use input::prelude::*;

#[test]
fn test_mouse_state_creation() {
    // Test creating a new mouse state
    let mouse = MouseState::new();

    // Assert that the mouse state is properly initialized
    // Initially the position should be (0, 0) and no buttons should be pressed
    assert_eq!(mouse.position().x, 0.0);
    assert_eq!(mouse.position().y, 0.0);
    assert!(!mouse.is_button_pressed(MouseButton::Left));
    assert!(!mouse.is_button_just_pressed(MouseButton::Left));
    assert!(!mouse.is_button_just_released(MouseButton::Left));
    assert_eq!(mouse.wheel_delta(), 0.0);
    assert_eq!(mouse.movement_delta(), (0.0, 0.0));
}

#[test]
fn test_first_position_update_records_position_without_delta() {
    let mut mouse = MouseState::new();

    // The first update after startup only establishes the position; its delta
    // against the default (0, 0) would be a spurious warp, so it is suppressed.
    mouse.update_position(10.0, 20.0);
    assert_eq!(mouse.position().x, 10.0);
    assert_eq!(mouse.position().y, 20.0);
    assert_eq!(mouse.movement_delta(), (0.0, 0.0));

    // A subsequent move within the same frame reports a real delta.
    mouse.update_position(13.0, 25.0);
    assert_eq!(mouse.position().x, 13.0);
    assert_eq!(mouse.position().y, 25.0);
    assert_eq!(mouse.movement_delta(), (3.0, 5.0));
}

#[test]
fn test_movement_delta_accumulates_within_frame() {
    let mut mouse = MouseState::new();

    // Establish the starting position (first update is delta-suppressed), then
    // begin a clean frame.
    mouse.update_position(10.0, 0.0);
    mouse.clear_frame_state();

    // Multiple move events in one frame (e.g. high polling rate mouse)
    mouse.update_position(15.0, 5.0);
    mouse.update_position(12.0, 8.0);

    // Delta should be the full frame movement, not just the last segment:
    // (15,5)-(10,0) + (12,8)-(15,5) = (5,5) + (-3,3) = (2,8)
    assert_eq!(mouse.position().x, 12.0);
    assert_eq!(mouse.position().y, 8.0);
    assert_eq!(mouse.movement_delta(), (2.0, 8.0));
}

#[test]
fn test_movement_delta_resets_each_frame() {
    let mut mouse = MouseState::new();

    // Establish the starting position (first update is delta-suppressed) and
    // start a clean frame.
    mouse.update_position(0.0, 0.0);
    mouse.clear_frame_state();

    // Move during frame 1
    mouse.update_position(10.0, 20.0);
    assert_eq!(mouse.movement_delta(), (10.0, 20.0));

    // End frame 1 — delta must reset even though the mouse stays still
    mouse.clear_frame_state();
    assert_eq!(mouse.movement_delta(), (0.0, 0.0));

    // Frame 2 with no movement still reports zero delta
    mouse.clear_frame_state();
    assert_eq!(mouse.movement_delta(), (0.0, 0.0));

    // Frame 3: a new move is measured relative to the current position
    mouse.update_position(13.0, 24.0);
    assert_eq!(mouse.movement_delta(), (3.0, 4.0));
}

#[test]
fn test_mouse_button_press_and_release() {
    // Test mouse button press and release
    let mut mouse = MouseState::new();

    // Press a button
    mouse.handle_button_press(MouseButton::Left);

    // Assert that the button is pressed and just pressed
    assert!(mouse.is_button_pressed(MouseButton::Left));
    assert!(mouse.is_button_just_pressed(MouseButton::Left));
    assert!(!mouse.is_button_just_released(MouseButton::Left));

    // Clear per-frame state to reset the "just pressed" flag
    mouse.clear_frame_state();

    // Assert that the button is still pressed but not just pressed
    assert!(mouse.is_button_pressed(MouseButton::Left));
    assert!(!mouse.is_button_just_pressed(MouseButton::Left));
    assert!(!mouse.is_button_just_released(MouseButton::Left));

    // Release the button
    mouse.handle_button_release(MouseButton::Left);

    // Assert that the button is not pressed but just released
    assert!(!mouse.is_button_pressed(MouseButton::Left));
    assert!(!mouse.is_button_just_pressed(MouseButton::Left));
    assert!(mouse.is_button_just_released(MouseButton::Left));

    // Clear per-frame state to reset the "just released" flag
    mouse.clear_frame_state();

    // Assert that the button is not pressed and not just released
    assert!(!mouse.is_button_pressed(MouseButton::Left));
    assert!(!mouse.is_button_just_pressed(MouseButton::Left));
    assert!(!mouse.is_button_just_released(MouseButton::Left));
}

#[test]
fn test_mouse_wheel() {
    // Test mouse wheel delta
    let mut mouse = MouseState::new();

    // Update the wheel delta
    mouse.update_wheel_delta(1.0);

    // Assert that the wheel delta was updated
    assert_eq!(mouse.wheel_delta(), 1.0);

    // Multiple scroll events in one frame accumulate
    mouse.update_wheel_delta(0.5);
    assert_eq!(mouse.wheel_delta(), 1.5);

    // End of frame clears the wheel delta
    mouse.clear_frame_state();
    assert_eq!(mouse.wheel_delta(), 0.0);

    // Update the wheel delta again
    mouse.update_wheel_delta(-2.0);

    // Assert that the wheel delta was updated
    assert_eq!(mouse.wheel_delta(), -2.0);
}

#[test]
fn test_multiple_mouse_buttons() {
    // Test handling multiple mouse buttons
    let mut mouse = MouseState::new();

    // Press multiple buttons
    mouse.handle_button_press(MouseButton::Left);
    mouse.handle_button_press(MouseButton::Right);
    mouse.handle_button_press(MouseButton::Middle);

    // Assert that all buttons are pressed
    assert!(mouse.is_button_pressed(MouseButton::Left));
    assert!(mouse.is_button_pressed(MouseButton::Right));
    assert!(mouse.is_button_pressed(MouseButton::Middle));

    // Release one button
    mouse.handle_button_release(MouseButton::Right);

    // Assert that the released button is not pressed but the others are
    assert!(mouse.is_button_pressed(MouseButton::Left));
    assert!(!mouse.is_button_pressed(MouseButton::Right));
    assert!(mouse.is_button_pressed(MouseButton::Middle));
    assert!(mouse.is_button_just_released(MouseButton::Right));
}
