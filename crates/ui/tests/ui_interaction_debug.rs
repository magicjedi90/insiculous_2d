//! UI interaction debugging tests
//!
//! These tests help identify why UI buttons and sliders aren't detecting clicks properly.
//! The tests verify the complete interaction flow from input events to UI detection.

use glam::Vec2;
use input::prelude::*;
use ui::prelude::*;
use ui::{InputState, InteractionManager};

#[test]
fn test_ui_button_click_detection() {
    // Create input handler and UI context
    let mut input_handler = InputHandler::new();
    let mut ui_context = UIContext::new();
    let window_size = Vec2::new(800.0, 600.0);
    
    // Define button bounds
    let button_bounds = Rect::new(100.0, 100.0, 200.0, 50.0);
    
    // === Frame 1: Move mouse over button ===
    // Queue mouse move event over button
    input_handler.queue_event(InputEvent::MouseMoved(200.0, 125.0)); // Center of button
    
    // Process events and begin frame
    input_handler.process_queued_events();
    ui_context.begin_frame(&input_handler, window_size);
    
    // Create button (should be hovered)
    let button_clicked = ui_context.button("test_button", "Click Me!", button_bounds);
    
    // End frame
    ui_context.end_frame();
    input_handler.end_frame();
    
    assert!(!button_clicked, "Button should not be clicked just by hovering");
    
    // === Frame 2: Press mouse button ===
    // Queue mouse press event
    input_handler.queue_event(InputEvent::MouseButtonPressed(MouseButton::Left));
    
    // Process events and begin frame
    input_handler.process_queued_events();
    ui_context.begin_frame(&input_handler, window_size);
    
    // Create button (should be active/pressed)
    let button_clicked = ui_context.button("test_button", "Click Me!", button_bounds);
    
    // End frame
    ui_context.end_frame();
    input_handler.end_frame();
    
    assert!(!button_clicked, "Button should not be clicked on press, only on release");
    
    // === Frame 3: Release mouse button ===
    // Queue mouse release event
    input_handler.queue_event(InputEvent::MouseButtonReleased(MouseButton::Left));
    
    // Process events and begin frame
    input_handler.process_queued_events();
    ui_context.begin_frame(&input_handler, window_size);
    
    // Create button (should be clicked)
    let button_clicked = ui_context.button("test_button", "Click Me!", button_bounds);
    
    // End frame
    ui_context.end_frame();
    input_handler.end_frame();
    
    assert!(button_clicked, "Button should be clicked when mouse is released over it");
}

#[test]
fn test_ui_slider_interaction() {
    // Create input handler and UI context
    let mut input_handler = InputHandler::new();
    let mut ui_context = UIContext::new();
    let window_size = Vec2::new(800.0, 600.0);
    
    // Define slider bounds
    let slider_bounds = Rect::new(100.0, 200.0, 200.0, 30.0);
    let initial_value = 0.5;
    
    // === Frame 1: Click and drag slider ===
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
    
    // End frame
    ui_context.end_frame();
    input_handler.end_frame();
    
    // === Frame 2: Drag to new position ===
    // Move mouse to new position (75% along slider)
    let new_mouse_x = 250.0; // 75% of slider width
    input_handler.queue_event(InputEvent::MouseMoved(new_mouse_x, mouse_y));
    
    // Process events and begin frame
    input_handler.process_queued_events();
    ui_context.begin_frame(&input_handler, window_size);
    
    // Create slider (should update value while dragging)
    let new_value = ui_context.slider("test_slider", new_value, slider_bounds);
    
    // End frame
    ui_context.end_frame();
    input_handler.end_frame();
    
    // Value should have increased
    assert!(new_value > initial_value, "Slider value should increase when dragged right");
    assert!((new_value - 0.75).abs() < 0.1, "Slider value should be approximately 0.75");
    
    // === Frame 3: Release mouse ===
    // Release mouse button
    input_handler.queue_event(InputEvent::MouseButtonReleased(MouseButton::Left));
    input_handler.process_queued_events();
    ui_context.begin_frame(&input_handler, window_size);
    
    // Create slider (should maintain final value)
    let final_value = ui_context.slider("test_slider", new_value, slider_bounds);
    
    // End frame
    ui_context.end_frame();
    input_handler.end_frame();
    
    assert!((final_value - new_value).abs() < 0.001, "Slider value should remain stable after release");
}

#[test]
fn test_input_state_from_input_handler() {
    let mut input_handler = InputHandler::new();
    
    // Test initial state
    let input_state = InputState::from_input_handler(&input_handler);
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
    assert_eq!(input_state.mouse_pos, Vec2::new(100.0, 200.0));
    assert!(input_state.mouse_down);
    assert!(input_state.mouse_just_pressed);
    assert!(!input_state.mouse_just_released);
    assert_eq!(input_state.scroll_delta, 1.5);
    
    // Add release event
    input_handler.queue_event(InputEvent::MouseButtonReleased(MouseButton::Left));
    input_handler.process_queued_events();
    
    let input_state = InputState::from_input_handler(&input_handler);
    assert!(!input_state.mouse_down);
    assert!(input_state.mouse_just_pressed); // Still true until end_frame() is called
    assert!(input_state.mouse_just_released);
    
    // Now call end_frame to clear the just_pressed/just_released states
    input_handler.end_frame();
    
    let input_state = InputState::from_input_handler(&input_handler);
    assert!(!input_state.mouse_down);
    assert!(!input_state.mouse_just_pressed); // Now cleared
    assert!(!input_state.mouse_just_released); // Now cleared
}

#[test]
fn test_interaction_manager_click_logic() {
    let mut interaction_manager = InteractionManager::new();
    let mut input_handler = InputHandler::new();
    let button_bounds = Rect::new(100.0, 100.0, 200.0, 50.0);
    let button_id = WidgetId::from_str("test_button");
    
    // === Frame 1: Move mouse over button ===
    input_handler.queue_event(InputEvent::MouseMoved(200.0, 125.0));
    input_handler.process_queued_events();
    interaction_manager.begin_frame(&input_handler);
    
    let result = interaction_manager.interact(button_id, button_bounds, true);
    
    assert_eq!(result.state, WidgetState::Hovered);
    assert!(!result.clicked);
    
    interaction_manager.end_frame();
    input_handler.end_frame();
    
    // === Frame 2: Press mouse ===
    input_handler.queue_event(InputEvent::MouseButtonPressed(MouseButton::Left));
    input_handler.process_queued_events();
    interaction_manager.begin_frame(&input_handler);
    
    let result = interaction_manager.interact(button_id, button_bounds, true);
    
    assert_eq!(result.state, WidgetState::Active);
    assert!(!result.clicked); // Not clicked yet, just pressed
    
    interaction_manager.end_frame();
    input_handler.end_frame();
    
    // === Frame 3: Release mouse (should trigger click) ===
    input_handler.queue_event(InputEvent::MouseButtonReleased(MouseButton::Left));
    input_handler.process_queued_events();
    interaction_manager.begin_frame(&input_handler);
    
    let result = interaction_manager.interact(button_id, button_bounds, true);
    
    assert_eq!(result.state, WidgetState::Hovered); // Back to hovered since released
    assert!(result.clicked); // Should be clicked now!
    
    interaction_manager.end_frame();
    input_handler.end_frame();
}

#[test]
fn test_input_timing_with_game_loop_order() {
    // This test verifies that the input state timing matches the expected game loop order:
    // 1. input.process_queued_events() - Process window events
    // 2. ui.begin_frame(&input, window_size) - Create InputState from InputHandler
    // 3. game.update() - Create UI elements, call ui.button() and ui.slider()
    // 4. ui.end_frame() - Collect draw commands
    // 5. input.end_frame() - Clear mouse state
    
    let mut input_handler = InputHandler::new();
    let mut ui_context = UIContext::new();
    let window_size = Vec2::new(800.0, 600.0);
    let button_bounds = Rect::new(100.0, 100.0, 200.0, 50.0);
    
    // Queue mouse events for a complete click
    input_handler.queue_event(InputEvent::MouseMoved(200.0, 125.0));
    input_handler.queue_event(InputEvent::MouseButtonPressed(MouseButton::Left));
    input_handler.queue_event(InputEvent::MouseButtonReleased(MouseButton::Left));
    
    // Step 1: Process input events (this should happen at start of frame)
    input_handler.process_queued_events();
    
    // Verify input state is correct after processing
    assert!(input_handler.mouse().is_button_just_pressed(MouseButton::Left));
    assert!(input_handler.mouse().is_button_just_released(MouseButton::Left));
    
    // Step 2: Begin UI frame (this calls InputState::from_input_handler)
    ui_context.begin_frame(&input_handler, window_size);
    
    // Step 3: Create UI elements (this calls ui.button() which uses interaction.interact())
    let button_clicked = ui_context.button("test_button", "Click Me!", button_bounds);
    
    // Button should detect the click
    assert!(button_clicked, "Button should detect click with correct timing");
    
    // Step 4: End UI frame
    ui_context.end_frame();
    
    // Step 5: End input frame (this clears just_pressed/just_released states)
    input_handler.end_frame();
    
    // Verify states are cleared
    assert!(!input_handler.mouse().is_button_just_pressed(MouseButton::Left));
    assert!(!input_handler.mouse().is_button_just_released(MouseButton::Left));
}

#[test]
fn test_click_outside_button() {
    let mut input_handler = InputHandler::new();
    let mut ui_context = UIContext::new();
    let window_size = Vec2::new(800.0, 600.0);
    let button_bounds = Rect::new(100.0, 100.0, 200.0, 50.0);
    
    // Click outside the button
    input_handler.queue_event(InputEvent::MouseMoved(50.0, 50.0)); // Outside button
    input_handler.queue_event(InputEvent::MouseButtonPressed(MouseButton::Left));
    input_handler.queue_event(InputEvent::MouseButtonReleased(MouseButton::Left));
    
    input_handler.process_queued_events();
    ui_context.begin_frame(&input_handler, window_size);
    
    let button_clicked = ui_context.button("test_button", "Click Me!", button_bounds);
    
    ui_context.end_frame();
    input_handler.end_frame();
    
    assert!(!button_clicked, "Button should not be clicked when mouse is outside");
}

#[test]
fn test_click_press_inside_release_outside() {
    let mut input_handler = InputHandler::new();
    let mut ui_context = UIContext::new();
    let window_size = Vec2::new(800.0, 600.0);
    let button_bounds = Rect::new(100.0, 100.0, 200.0, 50.0);
    
    // Press inside button
    input_handler.queue_event(InputEvent::MouseMoved(200.0, 125.0));
    input_handler.queue_event(InputEvent::MouseButtonPressed(MouseButton::Left));
    
    input_handler.process_queued_events();
    ui_context.begin_frame(&input_handler, window_size);
    let _ = ui_context.button("test_button", "Click Me!", button_bounds);
    ui_context.end_frame();
    input_handler.end_frame();
    
    // Release outside button
    input_handler.queue_event(InputEvent::MouseMoved(50.0, 50.0)); // Move outside
    input_handler.queue_event(InputEvent::MouseButtonReleased(MouseButton::Left));
    
    input_handler.process_queued_events();
    ui_context.begin_frame(&input_handler, window_size);
    let button_clicked = ui_context.button("test_button", "Click Me!", button_bounds);
    ui_context.end_frame();
    input_handler.end_frame();
    
    assert!(!button_clicked, "Button should not be clicked when released outside");
}