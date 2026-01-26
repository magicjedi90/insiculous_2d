use input::prelude::*;

#[test]
fn test_input_handler_creation() {
    // Test creating a new input handler
    let _input_handler = InputHandler::new();

    // Assert that the input handler is properly initialized
    // Since the input handler just contains default-initialized states,
    // we can verify it was created successfully by checking its components
    let input_handler = InputHandler::new();
    
    // Verify keyboard starts with no keys pressed
    assert!(!input_handler.keyboard().is_key_pressed(KeyCode::KeyA));
    
    // Verify mouse starts at origin
    assert_eq!(input_handler.mouse().position().x, 0.0);
    assert_eq!(input_handler.mouse().position().y, 0.0);
    
    // Verify no gamepads are connected initially
    assert!(input_handler.gamepads().get_gamepad(0).is_none());
}

#[test]
fn test_keyboard_access() {
    // Test accessing the keyboard state
    let mut input_handler = InputHandler::new();

    // Get immutable reference to keyboard state
    let _keyboard = input_handler.keyboard();

    // Assert that we can access keyboard state
    // We can verify the keyboard state is accessible and starts with no keys pressed
    let keyboard = input_handler.keyboard();
    assert!(!keyboard.is_key_pressed(KeyCode::KeyA));
    assert!(!keyboard.is_key_just_pressed(KeyCode::KeyA));

    // Get mutable reference to keyboard state
    let keyboard_mut = input_handler.keyboard_mut();

    // Simulate a key press
    keyboard_mut.handle_key_press(KeyCode::KeyA);

    // Assert that the key press was registered
    assert!(keyboard_mut.is_key_pressed(KeyCode::KeyA));
    assert!(keyboard_mut.is_key_just_pressed(KeyCode::KeyA));
}

#[test]
fn test_mouse_access() {
    // Test accessing the mouse state
    let mut input_handler = InputHandler::new();

    // Get immutable reference to mouse state
    let mouse = input_handler.mouse();

    // Assert that we can access mouse state
    // Initial position should be (0, 0)
    assert_eq!(mouse.position().x, 0.0);
    assert_eq!(mouse.position().y, 0.0);
    assert_eq!(mouse.wheel_delta(), 0.0);
    assert!(!mouse.is_button_pressed(MouseButton::Left));

    // Get mutable reference to mouse state
    let mouse_mut = input_handler.mouse_mut();

    // Update mouse position
    mouse_mut.update_position(10.0, 20.0);

    // Assert that the position was updated
    assert_eq!(mouse_mut.position().x, 10.0);
    assert_eq!(mouse_mut.position().y, 20.0);
    assert_eq!(mouse_mut.previous_position().x, 0.0);
    assert_eq!(mouse_mut.previous_position().y, 0.0);
}

#[test]
fn test_gamepad_access() {
    // Test accessing the gamepad manager
    let mut input_handler = InputHandler::new();

    // Get immutable reference to gamepad manager
    let _gamepads = input_handler.gamepads();

    // Assert that we can access gamepad manager
    // Initially there should be no gamepads
    assert!(input_handler.gamepads().get_gamepad(0).is_none());
    assert!(input_handler.gamepads().get_gamepad(1).is_none());

    // Get mutable reference to gamepad manager
    let gamepads_mut = input_handler.gamepads_mut();

    // Register a gamepad
    gamepads_mut.register_gamepad(0);

    // Assert that the gamepad was registered
    assert!(gamepads_mut.get_gamepad(0).is_some());
    
    // Verify the gamepad has default state
    let gamepad = gamepads_mut.get_gamepad(0).unwrap();
    assert!(!gamepad.is_button_pressed(GamepadButton::A));
    assert_eq!(gamepad.axis_value(GamepadAxis::LeftStickX), 0.0);
}

#[test]
fn test_input_handler_update() {
    // Test updating the input handler
    let mut input_handler = InputHandler::new();

    // Simulate some input events
    input_handler.keyboard_mut().handle_key_press(KeyCode::KeyA);
    input_handler
        .mouse_mut()
        .handle_button_press(MouseButton::Left);

    // Verify the inputs were registered
    assert!(input_handler.keyboard().is_key_just_pressed(KeyCode::KeyA));
    assert!(input_handler
        .mouse()
        .is_button_just_pressed(MouseButton::Left));

    // Update the input handler
    input_handler.update();

    // Assert that the "just pressed" states were cleared
    assert!(!input_handler.keyboard().is_key_just_pressed(KeyCode::KeyA));
    assert!(!input_handler
        .mouse()
        .is_button_just_pressed(MouseButton::Left));
    
    // Also verify that the regular pressed states are maintained
    assert!(input_handler.keyboard().is_key_pressed(KeyCode::KeyA));
    assert!(input_handler.mouse().is_button_pressed(MouseButton::Left));

    // But the keys should still be considered pressed
    assert!(input_handler.keyboard().is_key_pressed(KeyCode::KeyA));
    assert!(input_handler.mouse().is_button_pressed(MouseButton::Left));
}
