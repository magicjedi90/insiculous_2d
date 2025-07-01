use input::prelude::*;

#[test]
fn test_gamepad_state_creation() {
    // Test creating a new gamepad state
    let gamepad = GamepadState::new();

    // TODO: Assert that the gamepad state is properly initialized
    // Initially no buttons should be pressed and no axes should have values
    assert!(!gamepad.is_button_pressed(GamepadButton::A));
    assert!(!gamepad.is_button_just_pressed(GamepadButton::A));
    assert!(!gamepad.is_button_just_released(GamepadButton::A));
    assert_eq!(gamepad.axis_value(GamepadAxis::LeftStickX), 0.0);
}

#[test]
fn test_gamepad_button_press_and_release() {
    // Test gamepad button press and release
    let mut gamepad = GamepadState::new();

    // Press a button
    gamepad.handle_button_press(GamepadButton::A);

    // TODO: Assert that the button is pressed and just pressed
    assert!(gamepad.is_button_pressed(GamepadButton::A));
    assert!(gamepad.is_button_just_pressed(GamepadButton::A));
    assert!(!gamepad.is_button_just_released(GamepadButton::A));

    // Update to clear the "just pressed" state
    gamepad.update();

    // TODO: Assert that the button is still pressed but not just pressed
    assert!(gamepad.is_button_pressed(GamepadButton::A));
    assert!(!gamepad.is_button_just_pressed(GamepadButton::A));
    assert!(!gamepad.is_button_just_released(GamepadButton::A));

    // Release the button
    gamepad.handle_button_release(GamepadButton::A);

    // TODO: Assert that the button is not pressed but just released
    assert!(!gamepad.is_button_pressed(GamepadButton::A));
    assert!(!gamepad.is_button_just_pressed(GamepadButton::A));
    assert!(gamepad.is_button_just_released(GamepadButton::A));

    // Update to clear the "just released" state
    gamepad.update();

    // TODO: Assert that the button is not pressed and not just released
    assert!(!gamepad.is_button_pressed(GamepadButton::A));
    assert!(!gamepad.is_button_just_pressed(GamepadButton::A));
    assert!(!gamepad.is_button_just_released(GamepadButton::A));
}

#[test]
fn test_gamepad_axis() {
    // Test gamepad axis values
    let mut gamepad = GamepadState::new();

    // Update an axis value
    gamepad.update_axis(GamepadAxis::LeftStickX, 0.5);

    // TODO: Assert that the axis value was updated
    assert_eq!(gamepad.axis_value(GamepadAxis::LeftStickX), 0.5);

    // Other axes should still be 0
    assert_eq!(gamepad.axis_value(GamepadAxis::LeftStickY), 0.0);
    assert_eq!(gamepad.axis_value(GamepadAxis::RightStickX), 0.0);

    // Update another axis value
    gamepad.update_axis(GamepadAxis::LeftStickY, -0.75);

    // TODO: Assert that the axis value was updated
    assert_eq!(gamepad.axis_value(GamepadAxis::LeftStickY), -0.75);

    // The first axis should still have its value
    assert_eq!(gamepad.axis_value(GamepadAxis::LeftStickX), 0.5);
}

#[test]
fn test_gamepad_manager_creation() {
    // Test creating a new gamepad manager
    let manager = GamepadManager::new();

    // TODO: Assert that the manager is properly initialized
    // Initially there should be no gamepads
    assert!(manager.get_gamepad(0).is_none());
}

#[test]
fn test_gamepad_registration() {
    // Test registering and unregistering gamepads
    let mut manager = GamepadManager::new();

    // Register a gamepad
    manager.register_gamepad(0);

    // TODO: Assert that the gamepad was registered
    assert!(manager.get_gamepad(0).is_some());

    // Register another gamepad
    manager.register_gamepad(1);

    // TODO: Assert that both gamepads are registered
    assert!(manager.get_gamepad(0).is_some());
    assert!(manager.get_gamepad(1).is_some());

    // Unregister a gamepad
    manager.unregister_gamepad(0);

    // TODO: Assert that the gamepad was unregistered
    assert!(manager.get_gamepad(0).is_none());
    assert!(manager.get_gamepad(1).is_some());
}

#[test]
fn test_gamepad_manager_update() {
    // Test updating the gamepad manager
    let mut manager = GamepadManager::new();

    // Register a gamepad
    manager.register_gamepad(0);

    // Get a mutable reference to the gamepad
    let gamepad = manager.get_gamepad_mut(0).unwrap();

    // Press a button
    gamepad.handle_button_press(GamepadButton::A);

    // TODO: Assert that the button is pressed and just pressed
    assert!(gamepad.is_button_pressed(GamepadButton::A));
    assert!(gamepad.is_button_just_pressed(GamepadButton::A));

    // Update the manager
    manager.update();

    // Get a reference to the gamepad again
    let gamepad = manager.get_gamepad(0).unwrap();

    // TODO: Assert that the "just pressed" state was cleared
    assert!(gamepad.is_button_pressed(GamepadButton::A));
    assert!(!gamepad.is_button_just_pressed(GamepadButton::A));
}
