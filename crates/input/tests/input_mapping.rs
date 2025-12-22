use input::prelude::*;
use winit::keyboard::KeyCode;
use winit::event::MouseButton;

#[test]
fn test_input_mapping_creation() {
    let mut mapping = InputMapping::new();
    
    // Should have default bindings
    assert!(mapping.has_binding(&GameAction::MoveUp));
    assert!(mapping.has_binding(&GameAction::Action1));
    assert!(mapping.has_binding(&GameAction::Menu));
}

#[test]
fn test_default_movement_bindings() {
    let mut mapping = InputMapping::new();
    
    // Test W key for MoveUp
    let w_key = InputSource::Keyboard(KeyCode::KeyW);
    assert_eq!(mapping.get_action(&w_key), Some(&GameAction::MoveUp));
    
    // Test ArrowUp for MoveUp (should also be bound)
    let up_key = InputSource::Keyboard(KeyCode::ArrowUp);
    assert_eq!(mapping.get_action(&up_key), Some(&GameAction::MoveUp));
    
    // Test A key for MoveLeft
    let a_key = InputSource::Keyboard(KeyCode::KeyA);
    assert_eq!(mapping.get_action(&a_key), Some(&GameAction::MoveLeft));
    
    // Test D key for MoveRight
    let d_key = InputSource::Keyboard(KeyCode::KeyD);
    assert_eq!(mapping.get_action(&d_key), Some(&GameAction::MoveRight));
    
    // Test S key for MoveDown
    let s_key = InputSource::Keyboard(KeyCode::KeyS);
    assert_eq!(mapping.get_action(&s_key), Some(&GameAction::MoveDown));
}

#[test]
fn test_default_action_bindings() {
    let mapping = InputMapping::new();
    
    // Test Space key for Action1
    let space_key = InputSource::Keyboard(KeyCode::Space);
    assert_eq!(mapping.get_action(&space_key), Some(&GameAction::Action1));
    
    // Test Left mouse button for Action1
    let left_mouse = InputSource::Mouse(MouseButton::Left);
    assert_eq!(mapping.get_action(&left_mouse), Some(&GameAction::Action1));
    
    // Test Enter key for Action2
    let enter_key = InputSource::Keyboard(KeyCode::Enter);
    assert_eq!(mapping.get_action(&enter_key), Some(&GameAction::Action2));
    
    // Test Right mouse button for Action2
    let right_mouse = InputSource::Mouse(MouseButton::Right);
    assert_eq!(mapping.get_action(&right_mouse), Some(&GameAction::Action2));
}

#[test]
fn test_custom_binding() {
    let mut mapping = InputMapping::new();
    
    // Add a custom binding
    let custom_key = InputSource::Keyboard(KeyCode::KeyQ);
    let custom_action = GameAction::Custom(1);
    mapping.bind_input(custom_key, custom_action);
    
    // Verify the binding
    assert_eq!(mapping.get_action(&custom_key), Some(&custom_action));
    assert!(mapping.has_binding(&custom_action));
    
    // Get all bindings for the custom action
    let bindings = mapping.get_bindings(&custom_action);
    assert!(bindings.contains(&custom_key));
}

#[test]
fn test_binding_replacement() {
    let mut mapping = InputMapping::new();
    
    // W key should be bound to MoveUp by default
    let w_key = InputSource::Keyboard(KeyCode::KeyW);
    assert_eq!(mapping.get_action(&w_key), Some(&GameAction::MoveUp));
    
    // Rebind W key to a custom action
    let custom_action = GameAction::Custom(42);
    mapping.bind_input(w_key, custom_action);
    
    // Verify the rebinding
    assert_eq!(mapping.get_action(&w_key), Some(&custom_action));
    
    // MoveUp should still have ArrowUp binding, but not W key
    let move_up_bindings = mapping.get_bindings(&GameAction::MoveUp);
    assert!(!move_up_bindings.contains(&w_key)); // W key should not be in MoveUp bindings
    assert!(move_up_bindings.contains(&InputSource::Keyboard(KeyCode::ArrowUp))); // ArrowUp should still be there
    
    // Custom action should be bound to W key
    assert!(mapping.has_binding(&custom_action));
    let custom_bindings = mapping.get_bindings(&custom_action);
    assert!(custom_bindings.contains(&w_key));
}

#[test]
fn test_unbinding_input() {
    let mut mapping = InputMapping::new();
    
    // W key should be bound to MoveUp by default
    let w_key = InputSource::Keyboard(KeyCode::KeyW);
    assert_eq!(mapping.get_action(&w_key), Some(&GameAction::MoveUp));
    
    // Unbind W key
    mapping.unbind_input(&w_key);
    
    // Verify the unbinding
    assert_eq!(mapping.get_action(&w_key), None);
    
    // MoveUp should still have ArrowUp binding, but not W key
    let move_up_bindings = mapping.get_bindings(&GameAction::MoveUp);
    assert!(!move_up_bindings.contains(&w_key)); // W key should not be in MoveUp bindings
    assert!(move_up_bindings.contains(&InputSource::Keyboard(KeyCode::ArrowUp))); // ArrowUp should still be there
}

#[test]
fn test_unbinding_action() {
    let mut mapping = InputMapping::new();
    
    // MoveUp should have multiple bindings by default
    assert!(mapping.has_binding(&GameAction::MoveUp));
    
    // Unbind all inputs for MoveUp
    mapping.unbind_action(&GameAction::MoveUp);
    
    // Verify the unbinding
    assert!(!mapping.has_binding(&GameAction::MoveUp));
    
    // W and ArrowUp keys should no longer be bound to MoveUp
    let w_key = InputSource::Keyboard(KeyCode::KeyW);
    let up_key = InputSource::Keyboard(KeyCode::ArrowUp);
    assert_eq!(mapping.get_action(&w_key), None);
    assert_eq!(mapping.get_action(&up_key), None);
}

#[test]
fn test_clear_all_bindings() {
    let mut mapping = InputMapping::new();
    
    // Should have default bindings
    assert!(mapping.has_binding(&GameAction::MoveUp));
    assert!(mapping.has_binding(&GameAction::Action1));
    assert!(mapping.has_binding(&GameAction::Menu));
    
    // Clear all bindings
    mapping.clear_bindings();
    
    // Verify all bindings are cleared
    assert!(!mapping.has_binding(&GameAction::MoveUp));
    assert!(!mapping.has_binding(&GameAction::Action1));
    assert!(!mapping.has_binding(&GameAction::Menu));
}

#[test]
fn test_multiple_bindings_per_action() {
    let mut mapping = InputMapping::new();
    
    // MoveUp should have multiple bindings by default (W and ArrowUp)
    let bindings = mapping.get_bindings(&GameAction::MoveUp);
    assert!(bindings.len() >= 2);
    
    // Should contain both W and ArrowUp
    let w_key = InputSource::Keyboard(KeyCode::KeyW);
    let up_key = InputSource::Keyboard(KeyCode::ArrowUp);
    assert!(bindings.contains(&w_key));
    assert!(bindings.contains(&up_key));
}

#[test]
fn test_gamepad_bindings() {
    let mapping = InputMapping::new();
    
    // Test gamepad bindings for Action1 (should be A button on gamepad 0)
    let gamepad_a_button = InputSource::Gamepad(0, GamepadButton::A);
    assert_eq!(mapping.get_action(&gamepad_a_button), Some(&GameAction::Action1));
    
    // Test gamepad bindings for Menu (should be Start button on gamepad 0)
    let gamepad_start_button = InputSource::Gamepad(0, GamepadButton::Start);
    assert_eq!(mapping.get_action(&gamepad_start_button), Some(&GameAction::Menu));
}