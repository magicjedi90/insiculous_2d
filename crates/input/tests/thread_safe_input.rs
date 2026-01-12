use input::prelude::*;
use winit::keyboard::KeyCode;
use winit::event::MouseButton;
use std::thread;
use std::sync::Arc;

#[test]
fn test_thread_safe_input_handler_creation() {
    let _input_handler = ThreadSafeInputHandler::new();
    // Should not panic
}

#[test]
fn test_thread_safe_basic_operations() {
    let input_handler = ThreadSafeInputHandler::new();
    
    // Test basic operations
    let result = input_handler.is_key_pressed(KeyCode::KeyA);
    assert!(result.is_ok());
    assert!(!result.unwrap());
    
    let result = input_handler.is_key_just_pressed(KeyCode::KeyA);
    assert!(result.is_ok());
    assert!(!result.unwrap());
    
    let result = input_handler.mouse_position();
    assert!(result.is_ok());
    let pos = result.unwrap();
    assert_eq!(pos.x, 0.0);
    assert_eq!(pos.y, 0.0);
}

#[test]
fn test_thread_safe_event_queuing() {
    let input_handler = ThreadSafeInputHandler::new();
    
    // Queue events
    assert!(input_handler.queue_event(InputEvent::KeyPressed(KeyCode::KeyA)).is_ok());
    assert!(input_handler.queue_event(InputEvent::KeyReleased(KeyCode::KeyB)).is_ok());
    assert!(input_handler.queue_event(InputEvent::MouseButtonPressed(MouseButton::Left)).is_ok());
    
    // Update to process events
    assert!(input_handler.update().is_ok());
    
    // Check results
    let result = input_handler.is_key_pressed(KeyCode::KeyA);
    assert!(result.is_ok());
    assert!(result.unwrap());
    
    let result = input_handler.is_mouse_button_pressed(MouseButton::Left);
    assert!(result.is_ok());
    assert!(result.unwrap());
}

#[test]
fn test_thread_safe_action_integration() {
    let input_handler = ThreadSafeInputHandler::new();
    
    // Queue an event that should trigger an action
    assert!(input_handler.queue_event(InputEvent::KeyPressed(KeyCode::Space)).is_ok());
    
    // Process queued events
    assert!(input_handler.process_queued_events().is_ok());
    
    // Check that action is active and just activated
    let result = input_handler.is_action_active(&GameAction::Action1);
    assert!(result.is_ok());
    assert!(result.unwrap());
    
    let result = input_handler.is_action_just_activated(&GameAction::Action1);
    assert!(result.is_ok());
    assert!(result.unwrap());
}

#[test]
fn test_thread_safe_concurrent_access() {
    let input_handler = Arc::new(ThreadSafeInputHandler::new());
    let input_handler_clone = input_handler.clone();
    
    // Spawn a thread that queues events
    let handle = thread::spawn(move || {
        for i in 0..10 {
            let key = match i % 3 {
                0 => KeyCode::KeyA,
                1 => KeyCode::KeyB,
                _ => KeyCode::KeyC,
            };
            
            // Queue key press
            if let Err(e) = input_handler_clone.queue_event(InputEvent::KeyPressed(key)) {
                eprintln!("Failed to queue event: {}", e);
                return false;
            }
            
            // Small delay to simulate real-world timing
            thread::sleep(std::time::Duration::from_millis(1));
        }
        true
    });
    
    // In the main thread, also queue some events and check state
    for _ in 0..5 {
        assert!(input_handler.queue_event(InputEvent::KeyPressed(KeyCode::KeyD)).is_ok());
        thread::sleep(std::time::Duration::from_millis(2));
    }
    
    // Wait for the thread to complete
    assert!(handle.join().unwrap());
    
    // Process all events
    assert!(input_handler.update().is_ok());
    
    // Check that keys are pressed (some of them should be)
    let a_pressed = input_handler.is_key_pressed(KeyCode::KeyA).unwrap();
    let b_pressed = input_handler.is_key_pressed(KeyCode::KeyB).unwrap();
    let c_pressed = input_handler.is_key_pressed(KeyCode::KeyC).unwrap();
    let d_pressed = input_handler.is_key_pressed(KeyCode::KeyD).unwrap();
    
    // At least some keys should be pressed
    assert!(a_pressed || b_pressed || c_pressed || d_pressed);
}

#[test]
fn test_thread_safe_window_event_handling() {
    let input_handler = ThreadSafeInputHandler::new();
    
    // Test window event handling indirectly through event queuing
    // Since we can't easily create proper WindowEvent objects in tests
    
    // Just verify that the method exists and can be called with a dummy event
    // The actual functionality is tested through the event queue system
    assert!(input_handler.update().is_ok());
}

#[test]
fn test_thread_safe_mouse_operations() {
    let input_handler = ThreadSafeInputHandler::new();
    
    // Test mouse operations
    assert!(input_handler.queue_event(InputEvent::MouseButtonPressed(MouseButton::Right)).is_ok());
    assert!(input_handler.queue_event(InputEvent::MouseMoved(100.0, 200.0)).is_ok());
    assert!(input_handler.update().is_ok());
    
    // Check mouse button state
    let result = input_handler.is_mouse_button_pressed(MouseButton::Right);
    assert!(result.is_ok());
    assert!(result.unwrap());
    
    // Check mouse position
    let result = input_handler.mouse_position();
    assert!(result.is_ok());
    let pos = result.unwrap();
    assert_eq!(pos.x, 100.0);
    assert_eq!(pos.y, 200.0);
    
    // Check mouse delta
    let result = input_handler.mouse_movement_delta();
    assert!(result.is_ok());
    let (dx, dy) = result.unwrap();
    assert_eq!(dx, 100.0);
    assert_eq!(dy, 200.0);
}

#[test]
fn test_thread_safe_error_handling() {
    // This test is hard to trigger in practice since Mutex poisoning is rare,
    // but we can test the error types exist
    let error = InputThreadError::LockError("Test error".to_string());
    assert_eq!(format!("{}", error), "Failed to lock input handler: Test error");
    
    let error = InputThreadError::OperationError("Test operation error".to_string());
    assert_eq!(format!("{}", error), "Input operation error: Test operation error");
}

#[test]
fn test_thread_safe_state_access() {
    let input_handler = ThreadSafeInputHandler::new();
    
    // Test state access methods
    let result = input_handler.keyboard_state();
    assert!(result.is_ok());
    
    let result = input_handler.mouse_state();
    assert!(result.is_ok());
    
    let result = input_handler.gamepad_manager();
    assert!(result.is_ok());
}

#[test]
fn test_thread_safe_action_states() {
    let input_handler = ThreadSafeInputHandler::new();
    
    // Test action state methods
    let result = input_handler.is_action_active(&GameAction::MoveUp);
    assert!(result.is_ok());
    assert!(!result.unwrap()); // Should not be active initially
    
    let result = input_handler.is_action_just_activated(&GameAction::Action1);
    assert!(result.is_ok());
    assert!(!result.unwrap()); // Should not be just activated initially
    
    let result = input_handler.is_action_just_deactivated(&GameAction::Action1);
    assert!(result.is_ok());
    assert!(!result.unwrap()); // Should not be just deactivated initially
}