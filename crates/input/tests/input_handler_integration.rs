//! Integration tests: InputMapping action evaluation against InputHandler device state.

use input::prelude::*;
use winit::event::MouseButton;
use winit::keyboard::KeyCode;

#[test]
fn test_action_activation_from_key_press() {
    let mut input = InputHandler::new();
    let actions = InputMapping::with_default_bindings();

    // Simulate pressing the W key (bound to MoveUp in the default preset)
    input.queue_event(InputEvent::KeyPressed(KeyCode::KeyW));
    input.process_queued_events();

    assert!(actions.is_active(GameAction::MoveUp, &input));
    assert!(actions.just_activated(GameAction::MoveUp, &input));
    assert!(!actions.just_deactivated(GameAction::MoveUp, &input));
}

#[test]
fn test_action_states_after_end_frame() {
    let mut input = InputHandler::new();
    let actions = InputMapping::with_default_bindings();

    input.queue_event(InputEvent::KeyPressed(KeyCode::Space));
    input.process_queued_events();

    // Action should be active and just activated
    assert!(actions.is_active(GameAction::Action1, &input));
    assert!(actions.just_activated(GameAction::Action1, &input));

    // End the frame (clears just pressed/released states)
    input.end_frame();

    // Action should still be active but no longer just activated
    assert!(actions.is_active(GameAction::Action1, &input));
    assert!(!actions.just_activated(GameAction::Action1, &input));
}

#[test]
fn test_action_deactivation() {
    let mut input = InputHandler::new();
    let actions = InputMapping::with_default_bindings();

    // Press and settle
    input.queue_event(InputEvent::KeyPressed(KeyCode::Space));
    input.process_queued_events();
    input.end_frame();

    assert!(actions.is_active(GameAction::Action1, &input));

    // Now release the key
    input.queue_event(InputEvent::KeyReleased(KeyCode::Space));
    input.process_queued_events();

    assert!(!actions.is_active(GameAction::Action1, &input));
    assert!(!actions.just_activated(GameAction::Action1, &input));
    assert!(actions.just_deactivated(GameAction::Action1, &input));
}

#[test]
fn test_second_source_does_not_retrigger_activation() {
    // Pressing a second bound source while the action is already held must
    // not report just_activated again.
    let mut input = InputHandler::new();
    let actions = InputMapping::with_default_bindings();

    input.queue_event(InputEvent::KeyPressed(KeyCode::KeyW));
    input.process_queued_events();
    input.end_frame();

    // W is held; now press ArrowUp (also bound to MoveUp)
    input.queue_event(InputEvent::KeyPressed(KeyCode::ArrowUp));
    input.process_queued_events();

    assert!(actions.is_active(GameAction::MoveUp, &input));
    assert!(!actions.just_activated(GameAction::MoveUp, &input));
}

#[test]
fn test_releasing_one_source_keeps_action_active() {
    // Releasing one source while another bound source is still held must not
    // report just_deactivated.
    let mut input = InputHandler::new();
    let actions = InputMapping::with_default_bindings();

    input.queue_event(InputEvent::KeyPressed(KeyCode::KeyW));
    input.queue_event(InputEvent::KeyPressed(KeyCode::ArrowUp));
    input.process_queued_events();
    input.end_frame();

    // Release W; ArrowUp still held
    input.queue_event(InputEvent::KeyReleased(KeyCode::KeyW));
    input.process_queued_events();

    assert!(actions.is_active(GameAction::MoveUp, &input));
    assert!(!actions.just_deactivated(GameAction::MoveUp, &input));

    // Release ArrowUp too — now the action deactivates
    input.end_frame();
    input.queue_event(InputEvent::KeyReleased(KeyCode::ArrowUp));
    input.process_queued_events();

    assert!(!actions.is_active(GameAction::MoveUp, &input));
    assert!(actions.just_deactivated(GameAction::MoveUp, &input));
}

#[test]
fn test_one_source_bound_to_multiple_actions() {
    let mut input = InputHandler::new();

    let mut actions = InputMapping::new();
    let q_key = InputSource::Keyboard(KeyCode::KeyQ);
    actions.bind(GameAction::Custom(1), q_key);
    actions.bind(GameAction::Custom(2), q_key);

    input.queue_event(InputEvent::KeyPressed(KeyCode::KeyQ));
    input.process_queued_events();

    // Both actions respond to the same source
    assert!(actions.is_active(GameAction::Custom(1), &input));
    assert!(actions.is_active(GameAction::Custom(2), &input));
}

#[test]
fn test_mouse_action_integration() {
    let mut input = InputHandler::new();
    let actions = InputMapping::with_default_bindings();

    input.queue_event(InputEvent::MouseButtonPressed(MouseButton::Left));
    input.process_queued_events();

    assert!(actions.is_active(GameAction::Action1, &input));
    assert!(actions.just_activated(GameAction::Action1, &input));
}

#[test]
fn test_gamepad_action_integration_with_auto_registration() {
    let mut input = InputHandler::new();
    let actions = InputMapping::with_default_bindings();

    // No explicit register_gamepad() call — events auto-register the gamepad
    input.queue_event(InputEvent::GamepadButtonPressed(0, GamepadButton::A));
    input.process_queued_events();

    assert!(input.gamepads().get_gamepad(0).is_some());
    assert!(actions.is_active(GameAction::Action1, &input));
    assert!(actions.just_activated(GameAction::Action1, &input));
}

#[test]
fn test_action_with_no_bindings() {
    let mut input = InputHandler::new();
    let actions = InputMapping::with_default_bindings();

    input.queue_event(InputEvent::KeyPressed(KeyCode::KeyA));
    input.process_queued_events();

    let unbound = GameAction::Custom(999);
    assert!(!actions.is_active(unbound, &input));
    assert!(!actions.just_activated(unbound, &input));
    assert!(!actions.just_deactivated(unbound, &input));
}

#[test]
fn test_custom_action_enum() {
    // Games define their own action types; the engine preset is optional.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    enum PongAction {
        PaddleUp,
        PaddleDown,
    }

    let mut input = InputHandler::new();
    let mut actions = InputMapping::new();
    actions.bind(PongAction::PaddleUp, InputSource::Keyboard(KeyCode::KeyI));
    actions.bind(PongAction::PaddleDown, InputSource::Keyboard(KeyCode::KeyK));

    input.queue_event(InputEvent::KeyPressed(KeyCode::KeyI));
    input.process_queued_events();
    assert!(actions.is_active(PongAction::PaddleUp, &input));
    assert!(!actions.is_active(PongAction::PaddleDown, &input));

    input.end_frame();
    input.queue_event(InputEvent::KeyReleased(KeyCode::KeyI));
    input.queue_event(InputEvent::KeyPressed(KeyCode::KeyK));
    input.process_queued_events();
    assert!(!actions.is_active(PongAction::PaddleUp, &input));
    assert!(actions.is_active(PongAction::PaddleDown, &input));
}

#[test]
fn test_source_checks_on_handler() {
    let mut input = InputHandler::new();

    input.queue_event(InputEvent::KeyPressed(KeyCode::Space));
    input.process_queued_events();

    let space = InputSource::Keyboard(KeyCode::Space);
    assert!(input.is_source_pressed(&space));
    assert!(input.is_source_just_pressed(&space));
    assert!(!input.is_source_just_released(&space));

    input.end_frame();
    input.queue_event(InputEvent::KeyReleased(KeyCode::Space));
    input.process_queued_events();

    assert!(!input.is_source_pressed(&space));
    assert!(input.is_source_just_released(&space));
}
