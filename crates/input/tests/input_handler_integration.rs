use input::prelude::*;
use winit::keyboard::KeyCode;
use winit::event::MouseButton;

#[test]
fn test_input_handler_with_mapping() {
    let mut input_handler = InputHandler::new();
    
    // Simulate pressing the W key (bound to MoveUp by default)
    input_handler.queue_event(InputEvent::KeyPressed(KeyCode::KeyW));
    input_handler.process_queued_events();
    
    // Test that the MoveUp action is active
    assert!(input_handler.is_action_active(&GameAction::MoveUp));
    assert!(input_handler.is_action_just_activated(&GameAction::MoveUp));
    assert!(!input_handler.is_action_just_deactivated(&GameAction::MoveUp));
}

#[test]
fn test_action_states_after_update() {
    let mut input_handler = InputHandler::new();
    
    // Simulate pressing the Space key (bound to Action1 by default)
    input_handler.queue_event(InputEvent::KeyPressed(KeyCode::Space));
    input_handler.process_queued_events();
    
    // Action should be active and just activated
    assert!(input_handler.is_action_active(&GameAction::Action1));
    assert!(input_handler.is_action_just_activated(&GameAction::Action1));
    
    // Update the input handler (this clears just pressed/released states)
    input_handler.update();
    
    // Action should still be active but no longer just activated
    assert!(input_handler.is_action_active(&GameAction::Action1));
    assert!(!input_handler.is_action_just_activated(&GameAction::Action1));
}

#[test]
fn test_action_deactivation() {
    let mut input_handler = InputHandler::new();
    
    // Simulate pressing and releasing the Space key
    input_handler.queue_event(InputEvent::KeyPressed(KeyCode::Space));
    input_handler.process_queued_events();
    input_handler.update(); // Clear just pressed state
    
    // Action should be active
    assert!(input_handler.is_action_active(&GameAction::Action1));
    
    // Now release the key
    input_handler.queue_event(InputEvent::KeyReleased(KeyCode::Space));
    input_handler.process_queued_events();
    
    // Action should be inactive and just deactivated
    assert!(!input_handler.is_action_active(&GameAction::Action1));
    assert!(!input_handler.is_action_just_activated(&GameAction::Action1));
    assert!(input_handler.is_action_just_deactivated(&GameAction::Action1));
}

#[test]
fn test_multiple_actions_with_same_binding() {
    let mut input_handler = InputHandler::new();
    
    // Create a custom mapping where one key controls multiple actions
    let mut mapping = InputMapping::new();
    let custom_key = InputSource::Keyboard(KeyCode::KeyQ);
    mapping.bind_input_to_multiple_actions(custom_key, vec![GameAction::Custom(1), GameAction::Custom(2)]);
    
    // Replace the input handler's mapping
    *input_handler.input_mapping_mut() = mapping;
    
    // Simulate pressing the Q key
    input_handler.queue_event(InputEvent::KeyPressed(KeyCode::KeyQ));
    input_handler.process_queued_events();
    
    // Both custom actions should be active
    assert!(input_handler.is_action_active(&GameAction::Custom(1)));
    assert!(input_handler.is_action_active(&GameAction::Custom(2)));
}

#[test]
fn test_mouse_action_integration() {
    let mut input_handler = InputHandler::new();
    
    // Simulate left mouse button press (bound to Action1 by default)
    input_handler.queue_event(InputEvent::MouseButtonPressed(MouseButton::Left));
    input_handler.process_queued_events();
    
    // Action1 should be active
    assert!(input_handler.is_action_active(&GameAction::Action1));
    assert!(input_handler.is_action_just_activated(&GameAction::Action1));
}

#[test]
fn test_gamepad_action_integration() {
    let mut input_handler = InputHandler::new();
    
    // Register a gamepad first
    input_handler.gamepads_mut().register_gamepad(0);
    
    // Simulate gamepad A button press (bound to Action1 by default)
    input_handler.queue_event(InputEvent::GamepadButtonPressed(0, GamepadButton::A));
    input_handler.process_queued_events();
    
    // Action1 should be active
    assert!(input_handler.is_action_active(&GameAction::Action1));
    assert!(input_handler.is_action_just_activated(&GameAction::Action1));
}

#[test]
fn test_action_with_no_bindings() {
    let mut input_handler = InputHandler::new();
    
    // Create a custom action with no bindings
    let custom_action = GameAction::Custom(999);
    
    // Simulate some input
    input_handler.queue_event(InputEvent::KeyPressed(KeyCode::KeyA));
    input_handler.process_queued_events();
    
    // The custom action should not be active
    assert!(!input_handler.is_action_active(&custom_action));
    assert!(!input_handler.is_action_just_activated(&custom_action));
    assert!(!input_handler.is_action_just_deactivated(&custom_action));
}

#[test]
fn test_custom_input_mapping() {
    let mut input_handler = InputHandler::new();
    
    // Create a custom mapping
    let mut mapping = InputMapping::new();
    mapping.clear_bindings(); // Clear default bindings
    
    // Bind custom keys to actions
    mapping.bind_input(InputSource::Keyboard(KeyCode::KeyI), GameAction::MoveUp);
    mapping.bind_input(InputSource::Keyboard(KeyCode::KeyK), GameAction::MoveDown);
    mapping.bind_input(InputSource::Keyboard(KeyCode::KeyJ), GameAction::MoveLeft);
    mapping.bind_input(InputSource::Keyboard(KeyCode::KeyL), GameAction::MoveRight);
    
    // Replace the input handler's mapping
    *input_handler.input_mapping_mut() = mapping;
    
    // Test the custom bindings
    input_handler.queue_event(InputEvent::KeyPressed(KeyCode::KeyI));
    input_handler.process_queued_events();
    assert!(input_handler.is_action_active(&GameAction::MoveUp));
    
    // Release the I key to clear the state
    input_handler.queue_event(InputEvent::KeyReleased(KeyCode::KeyI));
    input_handler.process_queued_events();
    input_handler.update(); // Clear just pressed/released states
    
    input_handler.queue_event(InputEvent::KeyPressed(KeyCode::KeyK));
    input_handler.process_queued_events();
    assert!(input_handler.is_action_active(&GameAction::MoveDown));
    
    // Release the K key to clear the state
    input_handler.queue_event(InputEvent::KeyReleased(KeyCode::KeyK));
    input_handler.process_queued_events();
    input_handler.update(); // Clear just pressed/released states
    
    // Default WASD keys should not work anymore
    input_handler.queue_event(InputEvent::KeyPressed(KeyCode::KeyW));
    input_handler.process_queued_events();
    assert!(!input_handler.is_action_active(&GameAction::MoveUp));
}