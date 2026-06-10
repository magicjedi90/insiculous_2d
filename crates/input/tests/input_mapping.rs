use input::prelude::*;
use winit::event::MouseButton;
use winit::keyboard::KeyCode;

/// Games define their own action types — InputMapping is generic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum TestAction {
    Jump,
    Shoot,
    Pause,
}

#[test]
fn test_new_mapping_is_empty() {
    let mapping: InputMapping<TestAction> = InputMapping::new();

    assert!(mapping.is_empty());
    assert!(!mapping.has_binding(TestAction::Jump));
    assert!(mapping.bindings(TestAction::Jump).is_empty());
}

#[test]
fn test_bind_and_query() {
    let mut mapping = InputMapping::new();
    let space = InputSource::Keyboard(KeyCode::Space);

    mapping.bind(TestAction::Jump, space);

    assert!(mapping.has_binding(TestAction::Jump));
    assert_eq!(mapping.bindings(TestAction::Jump), &[space]);
    assert_eq!(mapping.actions_for(&space), vec![TestAction::Jump]);
}

#[test]
fn test_multiple_sources_per_action() {
    let mut mapping = InputMapping::new();
    let space = InputSource::Keyboard(KeyCode::Space);
    let pad_a = InputSource::Gamepad(0, GamepadButton::A);

    mapping.bind(TestAction::Jump, space);
    mapping.bind(TestAction::Jump, pad_a);

    let bindings = mapping.bindings(TestAction::Jump);
    assert!(bindings.contains(&space));
    assert!(bindings.contains(&pad_a));
}

#[test]
fn test_multiple_actions_per_source() {
    let mut mapping = InputMapping::new();
    let space = InputSource::Keyboard(KeyCode::Space);

    mapping.bind(TestAction::Jump, space);
    mapping.bind(TestAction::Shoot, space);

    assert!(mapping.bindings(TestAction::Jump).contains(&space));
    assert!(mapping.bindings(TestAction::Shoot).contains(&space));

    let actions = mapping.actions_for(&space);
    assert!(actions.contains(&TestAction::Jump));
    assert!(actions.contains(&TestAction::Shoot));
}

#[test]
fn test_duplicate_bind_is_noop() {
    let mut mapping = InputMapping::new();
    let space = InputSource::Keyboard(KeyCode::Space);

    mapping.bind(TestAction::Jump, space);
    mapping.bind(TestAction::Jump, space);

    assert_eq!(mapping.bindings(TestAction::Jump).len(), 1);
}

#[test]
fn test_unbind_single_source() {
    let mut mapping = InputMapping::new();
    let space = InputSource::Keyboard(KeyCode::Space);
    let pad_a = InputSource::Gamepad(0, GamepadButton::A);

    mapping.bind(TestAction::Jump, space);
    mapping.bind(TestAction::Jump, pad_a);

    mapping.unbind(TestAction::Jump, &space);

    assert_eq!(mapping.bindings(TestAction::Jump), &[pad_a]);

    // Removing the last source removes the action entirely
    mapping.unbind(TestAction::Jump, &pad_a);
    assert!(!mapping.has_binding(TestAction::Jump));
}

#[test]
fn test_unbind_action_removes_all_sources() {
    let mut mapping = InputMapping::new();
    mapping.bind(TestAction::Jump, InputSource::Keyboard(KeyCode::Space));
    mapping.bind(TestAction::Jump, InputSource::Keyboard(KeyCode::KeyW));

    mapping.unbind_action(TestAction::Jump);

    assert!(!mapping.has_binding(TestAction::Jump));
}

#[test]
fn test_unbind_source_removes_from_all_actions() {
    // Regression test: removing a source bound to multiple actions must not
    // leave stale bindings on any of them.
    let mut mapping = InputMapping::new();
    let space = InputSource::Keyboard(KeyCode::Space);
    let enter = InputSource::Keyboard(KeyCode::Enter);

    mapping.bind(TestAction::Jump, space);
    mapping.bind(TestAction::Shoot, space);
    mapping.bind(TestAction::Pause, enter);

    mapping.unbind_source(&space);

    assert!(!mapping.has_binding(TestAction::Jump));
    assert!(!mapping.has_binding(TestAction::Shoot));
    assert!(mapping.actions_for(&space).is_empty());
    // Unrelated bindings are untouched
    assert_eq!(mapping.bindings(TestAction::Pause), &[enter]);
}

#[test]
fn test_clear_removes_everything() {
    let mut mapping = InputMapping::new();
    mapping.bind(TestAction::Jump, InputSource::Keyboard(KeyCode::Space));
    mapping.bind(TestAction::Shoot, InputSource::Mouse(MouseButton::Left));

    mapping.clear();

    assert!(mapping.is_empty());
    assert!(!mapping.has_binding(TestAction::Jump));
    assert!(!mapping.has_binding(TestAction::Shoot));
}

// ================== Default GameAction Preset ==================

#[test]
fn test_default_bindings_preset() {
    let mapping = InputMapping::with_default_bindings();

    assert!(mapping.has_binding(GameAction::MoveUp));
    assert!(mapping.has_binding(GameAction::Action1));
    assert!(mapping.has_binding(GameAction::Menu));
}

#[test]
fn test_default_movement_bindings() {
    let mapping = InputMapping::with_default_bindings();

    // WASD + arrows
    assert!(mapping.bindings(GameAction::MoveUp).contains(&InputSource::Keyboard(KeyCode::KeyW)));
    assert!(mapping.bindings(GameAction::MoveUp).contains(&InputSource::Keyboard(KeyCode::ArrowUp)));
    assert!(mapping.bindings(GameAction::MoveDown).contains(&InputSource::Keyboard(KeyCode::KeyS)));
    assert!(mapping.bindings(GameAction::MoveLeft).contains(&InputSource::Keyboard(KeyCode::KeyA)));
    assert!(mapping.bindings(GameAction::MoveRight).contains(&InputSource::Keyboard(KeyCode::KeyD)));
}

#[test]
fn test_default_action_bindings() {
    let mapping = InputMapping::with_default_bindings();

    assert!(mapping.bindings(GameAction::Action1).contains(&InputSource::Keyboard(KeyCode::Space)));
    assert!(mapping.bindings(GameAction::Action1).contains(&InputSource::Mouse(MouseButton::Left)));
    assert!(mapping.bindings(GameAction::Action2).contains(&InputSource::Keyboard(KeyCode::Enter)));
    assert!(mapping.bindings(GameAction::Action2).contains(&InputSource::Mouse(MouseButton::Right)));
}

#[test]
fn test_default_gamepad_bindings() {
    let mapping = InputMapping::with_default_bindings();

    assert!(mapping
        .bindings(GameAction::Action1)
        .contains(&InputSource::Gamepad(0, GamepadButton::A)));
    assert!(mapping
        .bindings(GameAction::Menu)
        .contains(&InputSource::Gamepad(0, GamepadButton::Start)));
}
