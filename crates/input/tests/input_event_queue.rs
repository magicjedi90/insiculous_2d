use input::prelude::*;
use winit::event::MouseButton;
use winit::keyboard::KeyCode;

#[test]
fn test_input_event_queue_creation() {
    let _input_handler = InputHandler::new();
    // Should not panic
}

#[test]
fn test_input_event_queuing() {
    let mut input_handler = InputHandler::new();
    
    // Queue some events
    input_handler.queue_event(InputEvent::KeyPressed(KeyCode::KeyA));
    input_handler.queue_event(InputEvent::KeyReleased(KeyCode::KeyB));
    input_handler.queue_event(InputEvent::MouseButtonPressed(MouseButton::Left));
    
    // Events should be queued but not processed yet
    assert!(!input_handler.keyboard().is_key_pressed(KeyCode::KeyA));
    assert!(!input_handler.keyboard().is_key_pressed(KeyCode::KeyB));
    assert!(!input_handler.mouse().is_button_pressed(MouseButton::Left));
}

#[test]
fn test_input_event_processing() {
    let mut input_handler = InputHandler::new();
    
    // Queue some events
    input_handler.queue_event(InputEvent::KeyPressed(KeyCode::KeyA));
    input_handler.queue_event(InputEvent::KeyReleased(KeyCode::KeyB));
    input_handler.queue_event(InputEvent::MouseButtonPressed(MouseButton::Left));
    input_handler.queue_event(InputEvent::MouseMoved(100.0, 200.0));
    
    // Process the queued events
    input_handler.process_queued_events();
    
    // Verify events were processed
    assert!(input_handler.keyboard().is_key_pressed(KeyCode::KeyA));
    assert!(input_handler.keyboard().is_key_just_pressed(KeyCode::KeyA));
    assert!(!input_handler.keyboard().is_key_pressed(KeyCode::KeyB));
    assert!(input_handler.mouse().is_button_pressed(MouseButton::Left));
    assert!(input_handler.mouse().is_button_just_pressed(MouseButton::Left));
    assert_eq!(input_handler.mouse().position().x, 100.0);
    assert_eq!(input_handler.mouse().position().y, 200.0);
}

#[test]
fn test_update_clears_just_states() {
    let mut input_handler = InputHandler::new();
    
    // Queue and process some events
    input_handler.queue_event(InputEvent::KeyPressed(KeyCode::KeyA));
    input_handler.queue_event(InputEvent::MouseButtonPressed(MouseButton::Left));
    input_handler.process_queued_events();
    
    // Verify just pressed states are set
    assert!(input_handler.keyboard().is_key_just_pressed(KeyCode::KeyA));
    assert!(input_handler.mouse().is_button_just_pressed(MouseButton::Left));
    
    // Update (this should clear just pressed/released states)
    input_handler.update();
    
    // Verify just pressed states are cleared
    assert!(!input_handler.keyboard().is_key_just_pressed(KeyCode::KeyA));
    assert!(!input_handler.mouse().is_button_just_pressed(MouseButton::Left));
    
    // But the keys should still be considered pressed
    assert!(input_handler.keyboard().is_key_pressed(KeyCode::KeyA));
    assert!(input_handler.mouse().is_button_pressed(MouseButton::Left));
}

#[test]
fn test_multiple_events_processing_order() {
    let mut input_handler = InputHandler::new();
    
    // Queue events in a specific order
    input_handler.queue_event(InputEvent::KeyPressed(KeyCode::KeyA));
    input_handler.queue_event(InputEvent::KeyPressed(KeyCode::KeyB));
    input_handler.queue_event(InputEvent::KeyReleased(KeyCode::KeyA));
    
    // Process events
    input_handler.process_queued_events();
    
    // Verify final state
    assert!(!input_handler.keyboard().is_key_pressed(KeyCode::KeyA)); // Released
    assert!(input_handler.keyboard().is_key_pressed(KeyCode::KeyB));   // Still pressed
    assert!(input_handler.keyboard().is_key_just_released(KeyCode::KeyA));
    assert!(input_handler.keyboard().is_key_just_pressed(KeyCode::KeyB));
}

#[test]
fn test_window_event_handling() {
    let input_handler = InputHandler::new();
    
    // Test that window events are queued (we can't easily create them in tests)
    // but we can test that the handle_window_event method exists and doesn't panic
    
    // Just verify the input handler is in a clean state
    assert!(!input_handler.keyboard().is_key_pressed(KeyCode::KeyA));
    
    // The actual window event handling is tested indirectly through the event queue
    // since handle_window_event calls queue_event internally
}

#[test]
fn test_gamepad_event_queuing() {
    let mut input_handler = InputHandler::new();
    
    // Register a gamepad first
    input_handler.gamepads_mut().register_gamepad(0);
    
    // Queue gamepad events
    input_handler.queue_event(InputEvent::GamepadButtonPressed(0, GamepadButton::A));
    input_handler.queue_event(InputEvent::GamepadButtonReleased(0, GamepadButton::B));
    input_handler.queue_event(InputEvent::GamepadAxisUpdated(0, GamepadAxis::LeftStickX, 0.5));
    
    // Events should be queued but not processed yet
    let gamepad = input_handler.gamepads().get_gamepad(0).unwrap();
    assert!(!gamepad.is_button_pressed(GamepadButton::A));
    assert!(!gamepad.is_button_pressed(GamepadButton::B));
    assert_eq!(gamepad.axis_value(GamepadAxis::LeftStickX), 0.0);
    
    // Process the queued events
    input_handler.process_queued_events();
    
    // Verify events were processed
    let gamepad = input_handler.gamepads().get_gamepad(0).unwrap();
    assert!(gamepad.is_button_pressed(GamepadButton::A));
    assert!(gamepad.is_button_just_pressed(GamepadButton::A));
    assert!(!gamepad.is_button_pressed(GamepadButton::B));
    assert_eq!(gamepad.axis_value(GamepadAxis::LeftStickX), 0.5);
}