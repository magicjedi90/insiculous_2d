//! Test program to debug UI button and slider interaction issues.
//!
//! This test verifies the complete interaction flow:
//! 1. Mouse input is captured by InputHandler
//! 2. Input state is properly passed to UI system via InputState::from_input_handler()
//! 3. UI buttons and sliders detect clicks correctly
//! 4. Input state timing is correct (mouse state not cleared too early)

use glam::Vec2;
use input::prelude::*;
use ui::prelude::*;

/// Test the complete UI interaction flow with proper timing
#[test]
fn test_ui_button_click_detection() {
    println!("\n=== Testing UI Button Click Detection ===");
    
    // Create input handler and UI context
    let mut input_handler = InputHandler::new();
    let mut ui_context = UIContext::new();
    let window_size = Vec2::new(800.0, 600.0);
    
    // Define button bounds
    let button_bounds = Rect::new(100.0, 100.0, 200.0, 50.0);
    
    println!("Button bounds: {:?}", button_bounds);
    
    // === Frame 1: Move mouse over button ===
    println!("\n--- Frame 1: Move mouse over button ---");
    
    // Queue mouse move event over button
    input_handler.queue_event(InputEvent::MouseMoved(200.0, 125.0)); // Center of button
    
    // Process events and begin frame
    input_handler.process_queued_events();
    ui_context.begin_frame(&input_handler, window_size);
    
    // Create button (should be hovered)
    let button_clicked = ui_context.button("test_button", "Click Me!", button_bounds);
    
    println!("Mouse position: {:?}", input_handler.mouse().position());
    println!("Mouse over button: {}", button_bounds.contains(Vec2::new(200.0, 125.0)));
    println!("Button clicked: {}", button_clicked);
    println!("UI mouse pos: {:?}", ui_context.mouse_pos());
    
    // End frame
    ui_context.end_frame();
    input_handler.end_frame();
    
    assert!(!button_clicked, "Button should not be clicked just by hovering");
    
    // === Frame 2: Press mouse button ===
    println!("\n--- Frame 2: Press mouse button ---");
    
    // Queue mouse press event
    input_handler.queue_event(InputEvent::MouseButtonPressed(MouseButton::Left));
    
    // Process events and begin frame
    input_handler.process_queued_events();
    ui_context.begin_frame(&input_handler, window_size);
    
    // Create button (should be active/pressed)
    let button_clicked = ui_context.button("test_button", "Click Me!", button_bounds);
    
    println!("Mouse just pressed: {}", input_handler.mouse().is_button_just_pressed(MouseButton::Left));
    println!("Mouse pressed: {}", input_handler.mouse().is_button_pressed(MouseButton::Left));
    println!("Button clicked: {}", button_clicked);
    
    // End frame
    ui_context.end_frame();
    input_handler.end_frame();
    
    assert!(!button_clicked, "Button should not be clicked on press, only on release");
    
    // === Frame 3: Release mouse button ===
    println!("\n--- Frame 3: Release mouse button (should trigger click) ---");
    
    // Queue mouse release event
    input_handler.queue_event(InputEvent::MouseButtonReleased(MouseButton::Left));
    
    // Process events and begin frame
    input_handler.process_queued_events();
    ui_context.begin_frame(&input_handler, window_size);
    
    // Create button (should be clicked)
    let button_clicked = ui_context.button("test_button", "Click Me!", button_bounds);
    
    println!("Mouse just released: {}", input_handler.mouse().is_button_just_released(MouseButton::Left));
    println!("Mouse pressed: {}", input_handler.mouse().is_button_pressed(MouseButton::Left));
    println!("Button clicked: {}", button_clicked);
    
    // End frame
    ui_context.end_frame();
    input_handler.end_frame();
    
    assert!(button_clicked, "Button should be clicked when mouse is released over it");
    
    println!("\n✅ Button click detection works correctly!");
}

/// Test slider interaction
#[test]
fn test_ui_slider_interaction() {
    println!("\n=== Testing UI Slider Interaction ===");
    
    // Create input handler and UI context
    let mut input_handler = InputHandler::new();
    let mut ui_context = UIContext::new();
    let window_size = Vec2::new(800.0, 600.0);
    
    // Define slider bounds
    let slider_bounds = Rect::new(100.0, 200.0, 200.0, 30.0);
    let initial_value = 0.5;
    
    println!("Slider bounds: {:?}", slider_bounds);
    
    // === Frame 1: Click and drag slider ===
    println!("\n--- Frame 1: Click and drag slider ---");
    
    // Move mouse to slider position (center, should set value to ~0.5)
    let mouse_x = 200.0; // Center of slider
    let mouse_y = 215.0; // Center vertically
    
    input_handler.queue_event(InputEvent::MouseMoved(mouse_x, mouse_y));
    input_handler.queue_event(InputEvent::MouseButtonPressed(MouseButton::Left));
    
    // Process events and begin frame
    input_handler.process_queued_events();
    ui_context.begin_frame(&input_handler, window_size);
    
    // Create slider
    let new_value = ui_context.slider("test_slider", initial_value, slider_bounds);
    
    println!("Mouse position: {:?}", input_handler.mouse().position());
    println!("Initial value: {}", initial_value);
    println!("New value: {}", new_value);
    println!("Mouse just pressed: {}", input_handler.mouse().is_button_just_pressed(MouseButton::Left));
    
    // End frame
    ui_context.end_frame();
    input_handler.end_frame();
    
    // === Frame 2: Drag to new position ===
    println!("\n--- Frame 2: Drag to new position ---");
    
    // Move mouse to new position (75% along slider)
    let new_mouse_x = 250.0; // 75% of slider width
    input_handler.queue_event(InputEvent::MouseMoved(new_mouse_x, mouse_y));
    
    // Process events and begin frame
    input_handler.process_queued_events();
    ui_context.begin_frame(&input_handler, window_size);
    
    // Create slider (should update value while dragging)
    let new_value = ui_context.slider("test_slider", new_value, slider_bounds);
    
    println!("Mouse position: {:?}", input_handler.mouse().position());
    println!("Expected value: ~0.75");
    println!("New value: {}", new_value);
    println!("Mouse pressed: {}", input_handler.mouse().is_button_pressed(MouseButton::Left));
    
    // End frame
    ui_context.end_frame();
    input_handler.end_frame();
    
    // Value should have increased
    assert!(new_value > initial_value, "Slider value should increase when dragged right");
    assert!((new_value - 0.75).abs() < 0.1, "Slider value should be approximately 0.75");
    
    // === Frame 3: Release mouse ===
    println!("\n--- Frame 3: Release mouse ---");
    
    // Release mouse button
    input_handler.queue_event(InputEvent::MouseButtonReleased(MouseButton::Left));
    
    // Process events and begin frame
    input_handler.process_queued_events();
    ui_context.begin_frame(&input_handler, window_size);
    
    // Create slider (should maintain final value)
    let final_value = ui_context.slider("test_slider", new_value, slider_bounds);
    
    println!("Final value: {}", final_value);
    println!("Mouse just released: {}", input_handler.mouse().is_button_just_released(MouseButton::Left));
    
    // End frame
    ui_context.end_frame();
    input_handler.end_frame();
    
    assert!((final_value - new_value).abs() < 0.001, "Slider value should remain stable after release");
    
    println!("\n✅ Slider interaction works correctly!");
}

/// Test input state timing issue - this is the likely bug
#[test]
fn test_input_state_timing_issue() {
    println!("\n=== Testing Input State Timing Issue ===");
    
    // Create input handler
    let mut input_handler = InputHandler::new();
    let window_size = Vec2::new(800.0, 600.0);
    
    // Simulate the problematic game loop order:
    // 1. ui.begin_frame(&input, window_size) - passes InputHandler
    // 2. game.update() - creates UI elements, calls ui.button() and ui.slider()
    // 3. ui.end_frame() - collects draw commands
    // 4. input.end_frame() - clears mouse state
    
    let button_bounds = Rect::new(100.0, 100.0, 200.0, 50.0);
    
    // === Simulate click with potentially wrong timing ===
    println!("\n--- Simulating click with current game loop timing ---");
    
    // Queue mouse events
    input_handler.queue_event(InputEvent::MouseMoved(200.0, 125.0));
    input_handler.queue_event(InputEvent::MouseButtonPressed(MouseButton::Left));
    input_handler.queue_event(InputEvent::MouseButtonReleased(MouseButton::Left));
    
    // Create UI context
    let mut ui_context = UIContext::new();
    
    // Step 1: Process input events (this should happen at start of frame)
    input_handler.process_queued_events();
    println!("After processing events:");
    println!("  Mouse just pressed: {}", input_handler.mouse().is_button_just_pressed(MouseButton::Left));
    println!("  Mouse just released: {}", input_handler.mouse().is_button_just_released(MouseButton::Left));
    
    // Step 2: Begin UI frame (this calls InputState::from_input_handler)
    ui_context.begin_frame(&input_handler, window_size);
    
    // Check what InputState sees
    let input_state = InputState::from_input_handler(&input_handler);
    println!("InputState from InputHandler:");
    println!("  mouse_just_pressed: {}", input_state.mouse_just_pressed);
    println!("  mouse_just_released: {}", input_state.mouse_just_released);
    
    // Step 3: Create UI elements (this calls ui.button() which uses interaction.interact())
    let button_clicked = ui_context.button("test_button", "Click Me!", button_bounds);
    println!("Button clicked: {}", button_clicked);
    
    // Step 4: End UI frame
    ui_context.end_frame();
    
    // Step 5: End input frame (this clears just_pressed/just_released states)
    input_handler.end_frame();
    println!("After input.end_frame():");
    println!("  Mouse just pressed: {}", input_handler.mouse().is_button_just_pressed(MouseButton::Left));
    println!("  Mouse just released: {}", input_handler.mouse().is_button_just_released(MouseButton::Left));
    
    // This should work correctly with the current timing
    assert!(button_clicked, "Button should detect click with correct timing");
    
    println!("\n✅ Input state timing is correct!");
}

/// Test InputState::from_input_handler directly
#[test]
fn test_input_state_from_handler() {
    println!("\n=== Testing InputState::from_input_handler() ===");
    
    let mut input_handler = InputHandler::new();
    
    // Test initial state
    let input_state = InputState::from_input_handler(&input_handler);
    println!("Initial state:");
    println!("  mouse_pos: {:?}", input_state.mouse_pos);
    println!("  mouse_down: {}", input_state.mouse_down);
    println!("  mouse_just_pressed: {}", input_state.mouse_just_pressed);
    println!("  mouse_just_released: {}", input_state.mouse_just_released);
    
    assert_eq!(input_state.mouse_pos, Vec2::ZERO);
    assert!(!input_state.mouse_down);
    assert!(!input_state.mouse_just_pressed);
    assert!(!input_state.mouse_just_released);
    
    // Add some input events
    input_handler.queue_event(InputEvent::MouseMoved(100.0, 200.0));
    input_handler.queue_event(InputEvent::MouseButtonPressed(MouseButton::Left));
    input_handler.queue_event(InputEvent::MouseWheelScrolled(1.5));
    
    // Process events
    input_handler.process_queued_events();
    
    // Test state after events
    let input_state = InputState::from_input_handler(&input_handler);
    println!("\nAfter mouse events:");
    println!("  mouse_pos: {:?}", input_state.mouse_pos);
    println!("  mouse_down: {}", input_state.mouse_down);
    println!("  mouse_just_pressed: {}", input_state.mouse_just_pressed);
    println!("  mouse_just_released: {}", input_state.mouse_just_released);
    println!("  scroll_delta: {}", input_state.scroll_delta);
    
    assert_eq!(input_state.mouse_pos, Vec2::new(100.0, 200.0));
    assert!(input_state.mouse_down);
    assert!(input_state.mouse_just_pressed);
    assert!(!input_state.mouse_just_released);
    assert_eq!(input_state.scroll_delta, 1.5);
    
    // Add release event
    input_handler.queue_event(InputEvent::MouseButtonReleased(MouseButton::Left));
    input_handler.process_queued_events();
    
    let input_state = InputState::from_input_handler(&input_handler);
    println!("\nAfter release:");
    println!("  mouse_down: {}", input_state.mouse_down);
    println!("  mouse_just_pressed: {}", input_state.mouse_just_pressed);
    println!("  mouse_just_released: {}", input_state.mouse_just_released);
    
    assert!(!input_state.mouse_down);
    assert!(!input_state.mouse_just_pressed);
    assert!(input_state.mouse_just_released);
    
    println!("\n✅ InputState::from_input_handler() works correctly!");
}

/// Test interaction manager behavior
#[test]
fn test_interaction_manager_click_logic() {
    println!("\n=== Testing InteractionManager Click Logic ===");
    
    let mut interaction_manager = InteractionManager::new();
    let mut input_handler = InputHandler::new();
    let button_bounds = Rect::new(100.0, 100.0, 200.0, 50.0);
    let button_id = WidgetId::from_str("test_button");
    
    // === Frame 1: Move mouse over button ===
    println!("\n--- Frame 1: Move over button ---");
    
    input_handler.queue_event(InputEvent::MouseMoved(200.0, 125.0));
    input_handler.process_queued_events();
    interaction_manager.begin_frame(&input_handler);
    
    let result = interaction_manager.interact(button_id, button_bounds, true);
    
    println!("Mouse pos: {:?}", interaction_manager.mouse_pos());
    println!("Result state: {:?}", result.state);
    println!("Result clicked: {}", result.clicked);
    
    assert_eq!(result.state, WidgetState::Hovered);
    assert!(!result.clicked);
    
    interaction_manager.end_frame();
    input_handler.end_frame();
    
    // === Frame 2: Press mouse ===
    println!("\n--- Frame 2: Press mouse ---");
    
    input_handler.queue_event(InputEvent::MouseButtonPressed(MouseButton::Left));
    input_handler.process_queued_events();
    interaction_manager.begin_frame(&input_handler);
    
    let result = interaction_manager.interact(button_id, button_bounds, true);
    
    println!("Result state: {:?}", result.state);
    println!("Result clicked: {}", result.clicked);
    
    assert_eq!(result.state, WidgetState::Active);
    assert!(!result.clicked); // Not clicked yet, just pressed
    
    interaction_manager.end_frame();
    input_handler.end_frame();
    
    // === Frame 3: Release mouse (should trigger click) ===
    println!("\n--- Frame 3: Release mouse (should click) ---");
    
    input_handler.queue_event(InputEvent::MouseButtonReleased(MouseButton::Left));
    input_handler.process_queued_events();
    interaction_manager.begin_frame(&input_handler);
    
    let result = interaction_manager.interact(button_id, button_bounds, true);
    
    println!("Result state: {:?}", result.state);
    println!("Result clicked: {}", result.clicked);
    
    assert_eq!(result.state, WidgetState::Hovered); // Back to hovered since released
    assert!(result.clicked); // Should be clicked now!
    
    interaction_manager.end_frame();
    input_handler.end_frame();
    
    println!("\n✅ InteractionManager click logic works correctly!");
}

/// Main test runner
fn main() {
    println!("🧪 Running UI Interaction Debug Tests...\n");
    
    // Run all tests
    test_input_state_from_handler();
    test_interaction_manager_click_logic();
    test_ui_button_click_detection();
    test_ui_slider_interaction();
    test_input_state_timing_issue();
    
    println!("\n🎉 All UI interaction tests completed!");
    println!("\nIf any tests failed, review the output above to identify the issue.");
    println!("Common issues:");
    println!("- Input events not being processed before UI frame begins");
    println!("- Input state being cleared too early (before UI checks it)");
    println!("- Mouse position not being updated correctly");
}