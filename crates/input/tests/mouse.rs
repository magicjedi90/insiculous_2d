use input::prelude::*;

#[test]
fn test_mouse_state_creation() {
    // Test creating a new mouse state
    let mouse = MouseState::new();

    // TODO: Assert that the mouse state is properly initialized
    // Initially the position should be (0, 0) and no buttons should be pressed
    assert_eq!(mouse.position().x, 0.0);
    assert_eq!(mouse.position().y, 0.0);
    assert!(!mouse.is_button_pressed(MouseButton::Left));
    assert!(!mouse.is_button_just_pressed(MouseButton::Left));
    assert!(!mouse.is_button_just_released(MouseButton::Left));
    assert_eq!(mouse.wheel_delta(), 0.0);
}

#[test]
fn test_mouse_position() {
    // Test updating the mouse position
    let mut mouse = MouseState::new();

    // Update the position
    mouse.update_position(10.0, 20.0);

    // TODO: Assert that the position was updated
    assert_eq!(mouse.position().x, 10.0);
    assert_eq!(mouse.position().y, 20.0);

    // The previous position should still be (0, 0)
    assert_eq!(mouse.previous_position().x, 0.0);
    assert_eq!(mouse.previous_position().y, 0.0);

    // The movement delta should be (10, 20)
    let (dx, dy) = mouse.movement_delta();
    assert_eq!(dx, 10.0);
    assert_eq!(dy, 20.0);

    // Update the position again
    mouse.update_position(15.0, 25.0);

    // TODO: Assert that the position was updated
    assert_eq!(mouse.position().x, 15.0);
    assert_eq!(mouse.position().y, 25.0);

    // The previous position should now be (10, 20)
    assert_eq!(mouse.previous_position().x, 10.0);
    assert_eq!(mouse.previous_position().y, 20.0);

    // The movement delta should be (5, 5)
    let (dx, dy) = mouse.movement_delta();
    assert_eq!(dx, 5.0);
    assert_eq!(dy, 5.0);
}

#[test]
fn test_mouse_button_press_and_release() {
    // Test mouse button press and release
    let mut mouse = MouseState::new();

    // Press a button
    mouse.handle_button_press(MouseButton::Left);

    // TODO: Assert that the button is pressed and just pressed
    assert!(mouse.is_button_pressed(MouseButton::Left));
    assert!(mouse.is_button_just_pressed(MouseButton::Left));
    assert!(!mouse.is_button_just_released(MouseButton::Left));

    // Update to clear the "just pressed" state
    mouse.update();

    // TODO: Assert that the button is still pressed but not just pressed
    assert!(mouse.is_button_pressed(MouseButton::Left));
    assert!(!mouse.is_button_just_pressed(MouseButton::Left));
    assert!(!mouse.is_button_just_released(MouseButton::Left));

    // Release the button
    mouse.handle_button_release(MouseButton::Left);

    // TODO: Assert that the button is not pressed but just released
    assert!(!mouse.is_button_pressed(MouseButton::Left));
    assert!(!mouse.is_button_just_pressed(MouseButton::Left));
    assert!(mouse.is_button_just_released(MouseButton::Left));

    // Update to clear the "just released" state
    mouse.update();

    // TODO: Assert that the button is not pressed and not just released
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

    // TODO: Assert that the wheel delta was updated
    assert_eq!(mouse.wheel_delta(), 1.0);

    // Update to clear the wheel delta
    mouse.update();

    // TODO: Assert that the wheel delta was cleared
    assert_eq!(mouse.wheel_delta(), 0.0);

    // Update the wheel delta again
    mouse.update_wheel_delta(-2.0);

    // TODO: Assert that the wheel delta was updated
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

    // TODO: Assert that all buttons are pressed
    assert!(mouse.is_button_pressed(MouseButton::Left));
    assert!(mouse.is_button_pressed(MouseButton::Right));
    assert!(mouse.is_button_pressed(MouseButton::Middle));

    // Release one button
    mouse.handle_button_release(MouseButton::Right);

    // TODO: Assert that the released button is not pressed but the others are
    assert!(mouse.is_button_pressed(MouseButton::Left));
    assert!(!mouse.is_button_pressed(MouseButton::Right));
    assert!(mouse.is_button_pressed(MouseButton::Middle));
    assert!(mouse.is_button_just_released(MouseButton::Right));
}
