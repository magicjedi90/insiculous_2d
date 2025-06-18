// crates/builtin_states/src/play_state.rs
//! Primary “game running” state.  Pushes `PauseState` on ⌨️ TogglePause.

use engine_core::prelude::*;
use std::rc::Rc;
use winit::{event::Event, window::Window};

use renderer::core::Renderer;
use input::{PlayerCommand};

/// Active-gameplay state.  Right now it just clears the screen and counts
/// frames, but you can bolt in your ECS systems here.
pub struct PlayState {
    frame_count: u64,
}

impl Default for PlayState {
    fn default() -> Self {
        Self { frame_count: 0 }
    }
}

impl GameState for PlayState {
    /// Advance game logic each frame.
    fn update(&mut self, _delta_seconds: f32) {
        self.frame_count += 1;
        // TODO: run ECS, physics, AI, etc.
    }

    /// Draw the current frame.
    fn render(&mut self, _window: &Window, renderer: Rc<Renderer>) {
        // Simple gray clear for now.
        renderer.clear_screen([0.12, 0.12, 0.12, 1.0]);
    }

    /// Handle OS / keyboard events.
    fn handle_winit_event(&mut self, e: &Event<()>) -> Transition {
        // Map the raw winit event → PlayerCommand using the input crate.
        if matches!(command_from_event(e), Some(PlayerCommand::TogglePause)) {
            // Pause requested: push PauseState on top of the stack.
            Transition::Push(Box::new(super::pause_state::PauseState::default()))
        } else {
            Transition::None
        }
    }
}
